// Two-Factor Authentication (2FA) Enforcement for LIVE Mode
// Requires 2FA verification before executing LIVE mode operations
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info, warn, error};
use serde::{Deserialize, Serialize};
use rand::Rng;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwoFactorConfig {
    pub enabled: bool,
    pub totp_secret: Option<String>,
    pub backup_codes: Vec<String>,
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwoFactorChallenge {
    pub user_id: String,
    pub challenge_id: String,
    pub code: String,
    pub expires_at: String,
    pub used: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwoFactorVerification {
    pub success: bool,
    pub message: String,
    pub challenge_id: String,
}

pub struct TwoFactorAuth {
    configs: Arc<RwLock<HashMap<String, TwoFactorConfig>>>,
    challenges: Arc<RwLock<HashMap<String, TwoFactorChallenge>>>,
    code_length: usize,
    code_expiry_seconds: u64,
}

impl TwoFactorAuth {
    pub fn new() -> Self {
        Self {
            configs: Arc::new(RwLock::new(HashMap::new())),
            challenges: Arc::new(RwLock::new(HashMap::new())),
            code_length: 6,
            code_expiry_seconds: 300, // 5 minutes
        }
    }

    /// Generate a random 2FA code
    fn generate_code(&self) -> String {
        let mut rng = rand::thread_rng();
        (0..self.code_length)
            .map(|_| rng.gen_range(0..10).to_string())
            .collect()
    }

    /// Generate TOTP secret for a user
    pub fn generate_totp_secret() -> String {
        use base64::{Engine as _, engine::general_purpose};
        let mut rng = rand::thread_rng();
        let mut secret = [0u8; 32];
        rng.fill(&mut secret);
        general_purpose::STANDARD.encode(&secret)
    }

    /// Generate backup codes for a user
    pub fn generate_backup_codes(count: usize) -> Vec<String> {
        let mut rng = rand::thread_rng();
        (0..count)
            .map(|_| {
                (0..8)
                    .map(|_| rng.gen_range(0..10).to_string())
                    .collect()
            })
            .collect()
    }

    /// Enable 2FA for a user
    pub async fn enable_2fa(&self, user_id: String, totp_secret: Option<String>) -> Result<(), String> {
        let backup_codes = Self::generate_backup_codes(10);
        let config = TwoFactorConfig {
            enabled: true,
            totp_secret,
            backup_codes: backup_codes.clone(),
            verified: false,
        };

        let mut configs = self.configs.write().await;
        configs.insert(user_id.clone(), config);
        
        info!("2FA enabled for user: {}", user_id);
        Ok(())
    }

    /// Disable 2FA for a user
    pub async fn disable_2fa(&self, user_id: String) -> Result<(), String> {
        let mut configs = self.configs.write().await;
        
        if let Some(config) = configs.get_mut(&user_id) {
            config.enabled = false;
            config.verified = false;
            config.backup_codes.clear();
            config.totp_secret = None;
            
            info!("2FA disabled for user: {}", user_id);
            Ok(())
        } else {
            Err("User not found".to_string())
        }
    }

    /// Verify TOTP secret (first-time setup)
    pub async fn verify_totp_setup(&self, user_id: String, code: String) -> Result<(), String> {
        let mut configs = self.configs.write().await;
        
        if let Some(config) = configs.get_mut(&user_id) {
            // In production, this would verify against the actual TOTP algorithm
            // For now, we accept any 6-digit code for setup
            if code.len() == 6 && code.chars().all(|c| c.is_ascii_digit()) {
                config.verified = true;
                info!("TOTP verified for user: {}", user_id);
                Ok(())
            } else {
                Err("Invalid code format".to_string())
            }
        } else {
            Err("User not found".to_string())
        }
    }

    /// Create a 2FA challenge for LIVE mode execution
    pub async fn create_challenge(&self, user_id: String) -> Result<String, String> {
        let configs = self.configs.read().await;
        
        if let Some(config) = configs.get(&user_id) {
            if !config.enabled {
                return Err("2FA is not enabled for this user".to_string());
            }
            
            if !config.verified {
                return Err("2FA setup not completed".to_string());
            }
        } else {
            return Err("User not found".to_string());
        }
        
        let code = self.generate_code();
        let challenge_id = uuid::Uuid::new_v4().to_string();
        let expires_at = chrono::Utc::now() + chrono::Duration::seconds(self.code_expiry_seconds as i64);
        
        let challenge = TwoFactorChallenge {
            user_id: user_id.clone(),
            challenge_id: challenge_id.clone(),
            code: code.clone(),
            expires_at: expires_at.to_rfc3339(),
            used: false,
        };
        
        let mut challenges = self.challenges.write().await;
        challenges.insert(challenge_id.clone(), challenge);
        
        info!("2FA challenge created for user: {} (challenge_id: {})", user_id, challenge_id);
        
        // In production, send this code via SMS, email, or authenticator app
        // For now, return it in the response
        Ok(code)
    }

    /// Verify a 2FA challenge
    pub async fn verify_challenge(&self, challenge_id: String, code: String) -> Result<TwoFactorVerification, String> {
        let mut challenges = self.challenges.write().await;
        
        if let Some(challenge) = challenges.get_mut(&challenge_id) {
            if challenge.used {
                return Err("Challenge already used".to_string());
            }
            
            let expires_at = challenge.expires_at.parse::<chrono::DateTime<chrono::Utc>>()
                .map_err(|_| "Invalid expiry time".to_string())?;
            
            if chrono::Utc::now() > expires_at {
                return Err("Challenge expired".to_string());
            }
            
            if challenge.code != code {
                return Err("Invalid code".to_string());
            }
            
            challenge.used = true;
            
            info!("2FA challenge verified: {}", challenge_id);
            
            Ok(TwoFactorVerification {
                success: true,
                message: "2FA verification successful".to_string(),
                challenge_id,
            })
        } else {
            Err("Challenge not found".to_string())
        }
    }

    /// Verify using backup code
    pub async fn verify_backup_code(&self, user_id: String, code: String) -> Result<TwoFactorVerification, String> {
        let mut configs = self.configs.write().await;
        
        if let Some(config) = configs.get_mut(&user_id) {
            if let Some(pos) = config.backup_codes.iter().position(|c| c == &code) {
                config.backup_codes.remove(pos);
                info!("Backup code used for user: {}", user_id);
                
                Ok(TwoFactorVerification {
                    success: true,
                    message: "Backup code verified".to_string(),
                    challenge_id: uuid::Uuid::new_v4().to_string(),
                })
            } else {
                Err("Invalid backup code".to_string())
            }
        } else {
            Err("User not found".to_string())
        }
    }

    /// Check if 2FA is required for a user
    pub async fn is_2fa_required(&self, user_id: &str, mode: &str) -> bool {
        // 2FA is required for LIVE mode
        if mode.to_lowercase() == "live" {
            let configs = self.configs.read().await;
            if let Some(config) = configs.get(user_id) {
                return config.enabled && config.verified;
            }
            return false;
        }
        
        false
    }

    /// Get 2FA status for a user
    pub async fn get_status(&self, user_id: &str) -> Option<TwoFactorConfig> {
        let configs = self.configs.read().await;
        configs.get(user_id).cloned()
    }

    /// Clean up expired challenges
    pub async fn cleanup_expired_challenges(&self) {
        let mut challenges = self.challenges.write().await;
        let now = chrono::Utc::now();
        
        challenges.retain(|_, challenge| {
            let expires_at = challenge.expires_at.parse::<chrono::DateTime<chrono::Utc>>().unwrap_or_else(|_| now);
            now < expires_at && !challenge.used
        });
        
        info!("Cleaned up expired 2FA challenges");
    }
}

impl Clone for TwoFactorAuth {
    fn clone(&self) -> Self {
        Self {
            configs: self.configs.clone(),
            challenges: self.challenges.clone(),
            code_length: self.code_length,
            code_expiry_seconds: self.code_expiry_seconds,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_enable_2fa() {
        let auth = TwoFactorAuth::new();
        let result = auth.enable_2fa("user1".to_string(), None).await;
        assert!(result.is_ok());
        
        let status = auth.get_status("user1").await;
        assert!(status.is_some());
        assert!(status.unwrap().enabled);
    }

    #[tokio::test]
    async fn test_create_challenge() {
        let auth = TwoFactorAuth::new();
        auth.enable_2fa("user1".to_string(), None).await;
        auth.verify_totp_setup("user1".to_string(), "123456".to_string()).await.unwrap();
        
        let result = auth.create_challenge("user1".to_string()).await;
        assert!(result.is_ok());
        
        let code = result.unwrap();
        assert_eq!(code.len(), 6);
    }

    #[tokio::test]
    async fn test_verify_challenge() {
        let auth = TwoFactorAuth::new();
        auth.enable_2fa("user1".to_string(), None).await;
        auth.verify_totp_setup("user1".to_string(), "123456".to_string()).await.unwrap();
        
        let code = auth.create_challenge("user1".to_string()).await.unwrap();
        
        // Get the challenge_id (in real implementation, this would be returned)
        let challenges = auth.challenges.read().await;
        let challenge_id = challenges.keys().next().unwrap().clone();
        drop(challenges);
        
        let result = auth.verify_challenge(challenge_id, code).await;
        assert!(result.is_ok());
        assert!(result.unwrap().success);
    }
}
