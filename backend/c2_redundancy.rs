// ==============================================================================
// MODULE: C2 Redundancy System
// Purpose: Implements hot-standby C2 server with automatic failover
// ==============================================================================

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

/// C2 Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct C2Server {
    pub id: String,
    pub endpoint: String,
    pub region: String,
    pub priority: u8,  // 0 = hot standby, 1 = primary
    pub health_check_url: String,
    pub status: ServerStatus,
    pub last_health_check: u64,
    pub consecutive_failures: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ServerStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Failed,
}

impl C2Server {
    pub fn new(id: String, endpoint: String, region: String, priority: u8) -> Self {
        Self {
            id,
            endpoint,
            region,
            priority,
            health_check_url: format!("{}/health", endpoint),
            status: ServerStatus::Healthy,
            last_health_check: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            consecutive_failures: 0,
        }
    }

    pub fn is_primary(&self) -> bool {
        self.priority == 1
    }

    pub fn mark_healthy(&mut self) {
        self.status = ServerStatus::Healthy;
        self.consecutive_failures = 0;
        self.last_health_check = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }

    pub fn mark_unhealthy(&mut self) {
        self.consecutive_failures += 1;
        if self.consecutive_failures >= 3 {
            self.status = ServerStatus::Unhealthy;
        } else {
            self.status = ServerStatus::Degraded;
        }
        self.last_health_check = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }
}

/// C2 Redundancy Manager
pub struct C2RedundancyManager {
    servers: Vec<C2Server>,
    active_server_id: Option<String>,
    failover_threshold_ms: u64,
    last_failover: u64,
    failovers_enabled: bool,
}

impl C2RedundancyManager {
    pub fn new(failover_threshold_ms: u64) -> Self {
        Self {
            servers: Vec::new(),
            active_server_id: None,
            failover_threshold_ms,
            last_failover: 0,
            failovers_enabled: true,
        }
    }

    /// Add a C2 server to the pool
    pub fn add_server(&mut self, server: C2Server) {
        if self.active_server_id.is_none() && server.is_primary() {
            self.active_server_id = Some(server.id.clone());
        }
        self.servers.push(server);
    }

    /// Get the currently active server
    pub fn get_active(&self) -> Option<&C2Server> {
        self.servers.iter().find(|s| s.id == self.active_server_id.as_ref().unwrap_or(&String::new()))
    }

    /// Get all healthy servers sorted by priority
    pub fn get_healthy_servers(&self) -> Vec<&C2Server> {
        self.servers
            .iter()
            .filter(|s| s.status == ServerStatus::Healthy || s.status == ServerStatus::Degraded)
            .collect()
    }

    /// Check health of all servers and trigger failover if needed
    pub async fn check_health(&mut self) -> Result<bool, String> {
        let active_id = match &self.active_server_id {
            Some(id) => id,
            None => return Err("No active server configured".to_string()),
        };

        // Find active server
        let active_idx = self.servers
            .iter()
            .position(|s| s.id == *active_id)
            .ok_or("Active server not found")?;

        let needs_failover = match self.servers.get(active_idx) {
            Some(s) => s.status == ServerStatus::Unhealthy || s.status == ServerStatus::Failed,
            None => true,
        };

        if needs_failover && self.failovers_enabled {
            self.trigger_failover().await?;
            return Ok(true);
        }

        Ok(false)
    }

    /// Trigger failover to backup server
    pub async fn trigger_failover(&mut self) -> Result<String, String> {
        if !self.failovers_enabled {
            return Err("Failovers disabled".to_string());
        }

        // Find best backup server (lowest priority that's not current active)
        let backup = self.servers
            .iter()
            .filter(|s| Some(s.id.clone()) != self.active_server_id)
            .filter(|s| s.status == ServerStatus::Healthy || s.status == ServerStatus::Degraded)
            .min_by_key(|s| s.priority);

        match backup {
            Some(b) => {
                let old_active = self.active_server_id.clone();
                self.active_server_id = Some(b.id.clone());
                self.last_failover = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                Ok(format!(
                    "Failover: {} -> {}",
                    old_active.unwrap_or_default(),
                    b.id
                ))
            }
            None => Err("No backup server available".to_string()),
        }
    }

    /// Enable/disable automatic failovers
    pub fn set_failover_enabled(&mut self, enabled: bool) {
        self.failovers_enabled = enabled;
    }

    /// Get time since last failover
    pub fn time_since_failover(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now.saturating_sub(self.last_failover)
    }
}

impl Default for C2RedundancyManager {
    fn default() -> Self {
        Self::new(100)  // 100ms default failover threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_creation() {
        let server = C2Server::new(
            "us-east-1".to_string(),
            "https://c2.useast1.example.com".to_string(),
            "us-east-1".to_string(),
            1,
        );
        assert!(server.is_primary());
        assert_eq!(server.status, ServerStatus::Healthy);
    }

    #[test]
    fn test_failover() {
        let mut manager = C2RedundancyManager::new(100);
        
        let primary = C2Server::new(
            "primary".to_string(),
            "https://primary.example.com".to_string(),
            "us-east".to_string(),
            1,
        );
        let backup = C2Server::new(
            "backup".to_string(),
            "https://backup.example.com".to_string(),
            "us-west".to_string(),
            2,
        );
        
        manager.add_server(primary);
        manager.add_server(backup);
        
        assert_eq!(manager.get_active().unwrap().id, "primary");
    }

    #[test]
    fn test_failover_enabled_toggle() {
        let mut manager = C2RedundancyManager::new(100);
        manager.set_failover_enabled(false);
        assert!(!manager.failovers_enabled);
    }
}
