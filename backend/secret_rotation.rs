// Secret Rotation System for Production Security
// Automatically rotates API keys and secrets at configurable intervals
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::{interval, Instant};
use chrono::{Duration as ChronoDuration, Timelike, Utc};
use tracing::{info, warn, error};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretMetadata {
    pub name: String,
    pub value: String,
    pub created_at: String,
    pub last_rotated: String,
    pub rotation_interval_hours: u64,
    pub next_rotation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationResult {
    pub secret_name: String,
    pub rotated_at: String,
    pub previous_value_hash: String,
    pub status: String,
    pub message: String,
}

pub struct SecretRotationManager {
    secrets: Arc<RwLock<HashMap<String, SecretMetadata>>>,
    rotation_enabled: bool,
    default_interval_hours: u64,
}

impl SecretRotationManager {
    pub fn new(default_interval_hours: u64) -> Self {
        Self {
            secrets: Arc::new(RwLock::new(HashMap::new())),
            rotation_enabled: std::env::var("SECRET_ROTATION_ENABLED")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false),
            default_interval_hours,
        }
    }

    /// Add a secret to the rotation manager
    pub async fn add_secret(&self, name: String, value: String, interval_hours: Option<u64>) {
        let now = Utc::now();
        let interval = interval_hours.unwrap_or(self.default_interval_hours);
        let next_rotation = now + ChronoDuration::hours(interval as i64);

        let metadata = SecretMetadata {
            name: name.clone(),
            value,
            created_at: now.to_rfc3339(),
            last_rotated: now.to_rfc3339(),
            rotation_interval_hours: interval,
            next_rotation: next_rotation.to_rfc3339(),
        };

        let mut secrets = self.secrets.write().await;
        secrets.insert(name, metadata);
        info!("Added secret to rotation manager: {}", name);
    }

    /// Check if a secret needs rotation
    pub async fn needs_rotation(&self, name: &str) -> bool {
        let secrets = self.secrets.read().await;
        if let Some(metadata) = secrets.get(name) {
            let next_rotation = metadata.next_rotation.parse::<chrono::DateTime<chrono::Utc>>().unwrap_or_else(|_| Utc::now());
            Utc::now() >= next_rotation
        } else {
            false
        }
    }

    /// Rotate a specific secret
    pub async fn rotate_secret(&self, name: &str) -> Result<RotationResult, String> {
        if !self.rotation_enabled {
            return Ok(RotationResult {
                secret_name: name.to_string(),
                rotated_at: Utc::now().to_rfc3339(),
                previous_value_hash: "skipped".to_string(),
                status: "skipped".to_string(),
                message: "Secret rotation is disabled".to_string(),
            });
        }

        let mut secrets = self.secrets.write().await;
        
        if let Some(metadata) = secrets.get_mut(name) {
            let previous_hash = self.hash_value(&metadata.value);
            let now = Utc::now();
            
            // In production, this would call the actual secret provider API
            // For now, we simulate rotation by appending a timestamp
            let new_value = format!("{}_rotated_{}", metadata.value, now.timestamp());
            
            metadata.value = new_value.clone();
            metadata.last_rotated = now.to_rfc3339();
            metadata.next_rotation = (now + ChronoDuration::hours(metadata.rotation_interval_hours as i64)).to_rfc3339();
            
            info!("Rotated secret: {}", name);
            
            Ok(RotationResult {
                secret_name: name.to_string(),
                rotated_at: now.to_rfc3339(),
                previous_value_hash,
                status: "success".to_string(),
                message: "Secret rotated successfully".to_string(),
            })
        } else {
            Err(format!("Secret not found: {}", name))
        }
    }

    /// Get a secret value
    pub async fn get_secret(&self, name: &str) -> Option<String> {
        let secrets = self.secrets.read().await;
        secrets.get(name).map(|m| m.value.clone())
    }

    /// Check all secrets and rotate those that need it
    pub async fn check_and_rotate_all(&self) -> Vec<RotationResult> {
        let mut results = Vec::new();
        let secrets_to_rotate: Vec<String> = {
            let secrets = self.secrets.read().await;
            secrets.iter()
                .filter(|(name, _)| {
                    let next_rotation = _.1.next_rotation.parse::<chrono::DateTime<chrono::Utc>>().unwrap_or_else(|_| Utc::now());
                    Utc::now() >= next_rotation
                })
                .map(|(name, _)| name.clone())
                .collect()
        };

        for name in secrets_to_rotate {
            match self.rotate_secret(&name).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    error!("Failed to rotate secret {}: {}", name, e);
                    results.push(RotationResult {
                        secret_name: name.clone(),
                        rotated_at: Utc::now().to_rfc3339(),
                        previous_value_hash: "error".to_string(),
                        status: "error".to_string(),
                        message: e,
                    });
                }
            }
        }

        results
    }

    /// Start automatic rotation background task
    pub async fn start_rotation_task(&self, check_interval_hours: u64) {
        if !self.rotation_enabled {
            info!("Secret rotation is disabled, background task not started");
            return;
        }

        let manager = self.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(check_interval_hours * 3600));
            loop {
                interval.tick().await;
                info!("Running automatic secret rotation check...");
                let results = manager.check_and_rotate_all().await;
                
                let success_count = results.iter().filter(|r| r.status == "success").count();
                let error_count = results.iter().filter(|r| r.status == "error").count();
                
                info!("Secret rotation complete: {} rotated, {} errors", success_count, error_count);
            }
        });
    }

    /// Get rotation status for all secrets
    pub async fn get_rotation_status(&self) -> HashMap<String, SecretMetadata> {
        let secrets = self.secrets.read().await;
        secrets.clone()
    }

    fn hash_value(&self, value: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(value.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

impl Clone for SecretRotationManager {
    fn clone(&self) -> Self {
        Self {
            secrets: self.secrets.clone(),
            rotation_enabled: self.rotation_enabled,
            default_interval_hours: self.default_interval_hours,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_secret() {
        let manager = SecretRotationManager::new(24);
        manager.add_secret("test_key".to_string(), "test_value".to_string(), None).await;
        
        let value = manager.get_secret("test_key").await;
        assert!(value.is_some());
        assert_eq!(value.unwrap(), "test_value");
    }

    #[tokio::test]
    async fn test_needs_rotation() {
        let manager = SecretRotationManager::new(24);
        manager.add_secret("test_key".to_string(), "test_value".to_string(), Some(0)).await;
        
        // With 0 hour interval, should need rotation immediately
        assert!(manager.needs_rotation("test_key").await);
    }

    #[tokio::test]
    async fn test_rotate_secret() {
        let manager = SecretRotationManager::new(24);
        manager.rotation_enabled = true;
        manager.add_secret("test_key".to_string(), "test_value".to_string(), Some(0)).await;
        
        let result = manager.rotate_secret("test_key").await.unwrap();
        assert_eq!(result.status, "success");
        
        let new_value = manager.get_secret("test_key").await.unwrap();
        assert!(new_value.contains("_rotated_"));
    }
}
