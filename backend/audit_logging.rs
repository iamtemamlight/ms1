// Comprehensive Audit Logging for Production Security
// Logs all critical operations for security monitoring and compliance
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{Utc, DateTime};
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub timestamp: DateTime<Utc>,
    pub event_id: String,
    pub user_id: Option<String>,
    pub action: String,
    pub resource: Option<String>,
    pub result: AuditResult,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditResult {
    Success,
    Failure,
    Warning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogConfig {
    pub enabled: bool,
    pub log_to_file: bool,
    pub log_to_database: bool,
    pub log_to_external: bool,
    pub retention_days: u32,
}

pub struct AuditLogger {
    events: Arc<RwLock<Vec<AuditEvent>>>,
    config: AuditLogConfig,
}

impl AuditLogger {
    pub fn new() -> Self {
        let config = AuditLogConfig {
            enabled: std::env::var("AUDIT_LOGGING_ENABLED")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(true),
            log_to_file: std::env::var("AUDIT_LOG_TO_FILE")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false),
            log_to_database: std::env::var("AUDIT_LOG_TO_DATABASE")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(true),
            log_to_external: std::env::var("AUDIT_LOG_TO_EXTERNAL")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false),
            retention_days: std::env::var("AUDIT_LOG_RETENTION_DAYS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(90),
        };

        Self {
            events: Arc::new(RwLock::new(Vec::new())),
            config,
        }
    }

    /// Log an audit event
    pub async fn log(&self, event: AuditEvent) {
        if !self.config.enabled {
            return;
        }

        let mut events = self.events.write().await;
        events.push(event.clone());

        // Log to tracing
        match &event.result {
            AuditResult::Success => info!("AUDIT: {} - {} by {:?}", event.action, event.resource, event.user_id),
            AuditResult::Failure => error!("AUDIT FAILED: {} - {} by {:?}", event.action, event.resource, event.user_id),
            AuditResult::Warning => warn!("AUDIT WARNING: {} - {} by {:?}", event.action, event.resource, event.user_id),
        }

        // Log to file if enabled
        if self.config.log_to_file {
            self.log_to_file(&event).await;
        }

        // Log to database if enabled
        if self.config.log_to_database {
            self.log_to_database(&event).await;
        }

        // Log to external service if enabled
        if self.config.log_to_external {
            self.log_to_external(&event).await;
        }
    }

    /// Create a new audit event
    pub fn create_event(
        action: String,
        resource: Option<String>,
        result: AuditResult,
        user_id: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
        metadata: serde_json::Value,
    ) -> AuditEvent {
        AuditEvent {
            timestamp: Utc::now(),
            event_id: uuid::Uuid::new_v4().to_string(),
            user_id,
            action,
            resource,
            result,
            ip_address,
            user_agent,
            metadata,
        }
    }

    /// Get audit events
    pub async fn get_events(&self, limit: usize) -> Vec<AuditEvent> {
        let events = self.events.read().await;
        events.iter().rev().take(limit).cloned().collect()
    }

    /// Get audit events by user
    pub async fn get_events_by_user(&self, user_id: &str, limit: usize) -> Vec<AuditEvent> {
        let events = self.events.read().await;
        events.iter()
            .filter(|e| e.user_id.as_deref() == Some(user_id))
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    /// Get audit events by action
    pub async fn get_events_by_action(&self, action: &str, limit: usize) -> Vec<AuditEvent> {
        let events = self.events.read().await;
        events.iter()
            .filter(|e| e.action == action)
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    /// Clean up old events based on retention policy
    pub async fn cleanup_old_events(&self) {
        let mut events = self.events.write().await;
        let cutoff = Utc::now() - chrono::Duration::days(self.config.retention_days as i64);
        
        let before_count = events.len();
        events.retain(|e| e.timestamp > cutoff);
        let after_count = events.len();
        
        info!("Audit log cleanup: removed {} events (retention: {} days)", before_count - after_count, self.config.retention_days);
    }

    async fn log_to_file(&self, event: &AuditEvent) {
        // In production, write to a log file
        // For now, we just log via tracing
        info!("AUDIT FILE: {}", serde_json::to_string(event).unwrap_or_default());
    }

    async fn log_to_database(&self, event: &AuditEvent) {
        // In production, write to database
        // For now, we just log via tracing
        info!("AUDIT DB: {}", serde_json::to_string(event).unwrap_or_default());
    }

    async fn log_to_external(&self, event: &AuditEvent) {
        // In production, send to external service (Datadog, Splunk, etc.)
        // For now, we just log via tracing
        info!("AUDIT EXTERNAL: {}", serde_json::to_string(event).unwrap_or_default());
    }

    /// Get audit statistics
    pub async fn get_statistics(&self) -> AuditStatistics {
        let events = self.events.read().await;
        
        let total = events.len();
        let success = events.iter().filter(|e| matches!(e.result, AuditResult::Success)).count();
        let failure = events.iter().filter(|e| matches!(e.result, AuditResult::Failure)).count();
        let warning = events.iter().filter(|e| matches!(e.result, AuditResult::Warning)).count();
        
        let unique_users = events.iter()
            .filter_map(|e| e.user_id.clone())
            .collect::<std::collections::HashSet<_>>()
            .len();
        
        let unique_actions = events.iter()
            .map(|e| e.action.clone())
            .collect::<std::collections::HashSet<_>>()
            .len();
        
        AuditStatistics {
            total_events: total,
            success_events: success,
            failure_events: failure,
            warning_events: warning,
            unique_users,
            unique_actions,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStatistics {
    pub total_events: usize,
    pub success_events: usize,
    pub failure_events: usize,
    pub warning_events: usize,
    pub unique_users: usize,
    pub unique_actions: usize,
}

impl Clone for AuditLogger {
    fn clone(&self) -> Self {
        Self {
            events: self.events.clone(),
            config: AuditLogConfig {
                enabled: self.config.enabled,
                log_to_file: self.config.log_to_file,
                log_to_database: self.config.log_to_database,
                log_to_external: self.config.log_to_external,
                retention_days: self.config.retention_days,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_log_event() {
        let logger = AuditLogger::new();
        let event = AuditLogger::create_event(
            "TEST_ACTION".to_string(),
            Some("test_resource".to_string()),
            AuditResult::Success,
            Some("user123".to_string()),
            Some("127.0.0.1".to_string()),
            Some("test-agent".to_string()),
            serde_json::json!({"test": "data"}),
        );
        
        logger.log(event).await;
        
        let events = logger.get_events(10).await;
        assert_eq!(events.len(), 1);
    }

    #[tokio::test]
    async fn test_get_events_by_user() {
        let logger = AuditLogger::new();
        
        let event1 = AuditLogger::create_event(
            "ACTION1".to_string(),
            None,
            AuditResult::Success,
            Some("user123".to_string()),
            None,
            None,
            serde_json::json!({}),
        );
        
        let event2 = AuditLogger::create_event(
            "ACTION2".to_string(),
            None,
            AuditResult::Success,
            Some("user456".to_string()),
            None,
            None,
            serde_json::json!({}),
        );
        
        logger.log(event1).await;
        logger.log(event2).await;
        
        let user_events = logger.get_events_by_user("user123", 10).await;
        assert_eq!(user_events.len(), 1);
    }

    #[tokio::test]
    async fn test_statistics() {
        let logger = AuditLogger::new();
        
        logger.log(AuditLogger::create_event(
            "ACTION1".to_string(),
            None,
            AuditResult::Success,
            None,
            None,
            None,
            serde_json::json!({}),
        )).await;
        
        logger.log(AuditLogger::create_event(
            "ACTION2".to_string(),
            None,
            AuditResult::Failure,
            None,
            None,
            None,
            serde_json::json!({}),
        )).await;
        
        let stats = logger.get_statistics().await;
        assert_eq!(stats.total_events, 2);
        assert_eq!(stats.success_events, 1);
        assert_eq!(stats.failure_events, 1);
    }
}
