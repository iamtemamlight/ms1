#![allow(dead_code, deprecated)]
// ==============================================================================
// ALLBRIGHT SECURE ENV VAULT
// ==============================================================================
// Multi-layer encryption for .env API keys and secrets
// Uses AES-256-GCM with Argon2id key derivation
// Author: External Security Auditor

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2,
};
use secrecy::{SecretString};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::io::Write;
use std::fs::OpenOptions;
use chrono::Utc;
use thiserror::Error;
use tracing::info;

#[derive(Error, Debug)]
pub enum VaultError {
    #[error("Encryption failed: {0}")]
    Encryption(String),
    #[error("Decryption failed: {0}")]
    Decryption(String),
    #[error("Key derivation failed: {0}")]
    KeyDerivation(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Vault not found. Run with --setup-vault first.")]
    NotFound,
    #[error("Invalid password")]
    InvalidPassword,
    #[error("Vault corrupted")]
    Corrupted,
}

/// Encrypted vault structure
#[derive(serde::Serialize, serde::Deserialize)]
struct VaultData {
    salt: String,           // Base64-encoded salt for Argon2
    encrypted_data: String, // Base64-encoded AES-encrypted data
    version: u32,
}

/// Secure environment variable with memory protection
/// Uses SecretString from secrecy crate which automatically zeros memory on drop
#[derive(Clone)]
pub struct SecureEnvVar {
    pub key: String,
    pub value: SecretString,
}

impl SecureEnvVar {
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: SecretString::new(value.into()),
        }
    }
}

/// Main vault manager for encrypted environment loading
pub struct EnvVault {
    cipher: Aes256Gcm,
    vault_path: PathBuf,
}

impl EnvVault {
    /// Create new vault with master password
    pub fn create(vault_path: PathBuf, master_password: &str) -> Result<Self, VaultError> {
        // Generate random salt using argon2's built-in salt generator
        let salt = SaltString::generate(&mut OsRng);
        
        // Derive 256-bit key using Argon2id
        let key = Self::derive_key(master_password, salt.as_str())?;
        
        // Create cipher
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| VaultError::KeyDerivation(e.to_string()))?;
        
        // Create vault file with empty secret map
        let empty_map: HashMap<String, String> = HashMap::new();
        let json = serde_json::to_vec(&empty_map)
            .map_err(|e| VaultError::Encryption(e.to_string()))?;
        
        // Encrypt the data with random nonce
        let mut nonce_bytes = [0u8; 12];
        use rand::RngCore;
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let ciphertext = cipher.encrypt(nonce, json.as_ref())
            .map_err(|e| VaultError::Encryption(e.to_string()))?;
        
        // Combine nonce + ciphertext for storage
        let mut combined = Vec::with_capacity(12 + ciphertext.len());
        combined.extend_from_slice(&nonce_bytes);
        combined.extend_from_slice(&ciphertext);
        
        // Store vault data
        let vault = VaultData {
            salt: salt.to_string(),
            encrypted_data: base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &combined),
            version: 1,
        };
        
        let vault_json = serde_json::to_string_pretty(&vault)
            .map_err(|e| VaultError::Encryption(e.to_string()))?;
        
        fs::write(&vault_path, vault_json)?;
        
        info!("Vault created at: {:?}", vault_path);
        
        Ok(Self {
            cipher,
            vault_path,
        })
    }

    /// Open existing vault with password
    pub fn open(vault_path: PathBuf, master_password: &str) -> Result<Self, VaultError> {
        if !vault_path.exists() {
            return Err(VaultError::NotFound);
        }
        
        let vault_json = fs::read_to_string(&vault_path)?;
        let vault: VaultData = serde_json::from_str(&vault_json)
            .map_err(|_| VaultError::Corrupted)?;
        
        // Derive key from password and stored salt
        let key = Self::derive_key(master_password, &vault.salt)?;
        
        // Create cipher for decryption
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| VaultError::KeyDerivation(e.to_string()))?;
        
        // Decode and decrypt
        let combined = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &vault.encrypted_data)
            .map_err(|_| VaultError::Corrupted)?;
        
        if combined.len() < 12 {
            return Err(VaultError::Corrupted);
        }
        
        let nonce = Nonce::from_slice(&combined[..12]);
        let ciphertext = &combined[12..];
        
        // Test decryption by attempting to decrypt
        // (will fail if wrong password due to auth tag)
        cipher.decrypt(nonce, ciphertext)
            .map_err(|_| VaultError::InvalidPassword)?;
        
        Ok(Self {
            cipher,
            vault_path,
        })
    }

/// Derive 256-bit key from password using Argon2id
     fn derive_key(password: &str, salt: &str) -> Result<[u8; 32], VaultError> {
         let argon2 = Argon2::default();
         
         // Use password and salt to generate hash
         let salt = SaltString::from_b64(salt)
             .map_err(|e| VaultError::KeyDerivation(e.to_string()))?;
         
         let hash = argon2
             .hash_password(password.as_bytes(), &salt)
             .map_err(|e| VaultError::KeyDerivation(e.to_string()))?;
         
         // Extract hash output - the hash() method returns Option<Output>
         let hash_output = hash.hash.ok_or_else(|| {
             VaultError::KeyDerivation("No hash output".to_string())
         })?;
         
         // Extract first 32 bytes for AES-256
         let hash_bytes = hash_output.as_bytes();
         let mut key = [0u8; 32];
         let len = hash_bytes.len().min(32);
         key[..len].copy_from_slice(&hash_bytes[..len]);
         
         Ok(key)
     }
     
      /// Rotate encryption keys using a new master password
      pub fn rotate_keys(&self, _new_master_password: &str) -> Result<(), VaultError> {
          Err(VaultError::KeyDerivation("rotate_keys requires current password parameter - implement with explicit current_password argument".into()))
      }
     
     /// Retrieve all secrets from the vault (for migration)
     fn get_all_secrets(&self) -> HashMap<String, String> {
         // Implementation would read current vault JSON and return secret map
         // This is a placeholder for actual logic
         HashMap::new()
     }
     
     /// Append audit entry to immutable audit log
     fn log_audit_action(entry: &str) {
         // Ensure audit log file exists
         let audit_path = PathBuf::from("audit_final.log");
         let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%SZ");
         
         let log_entry = format!("{}|{}\n", timestamp, entry);
         
         // Append securely (flush immediately to avoid buffering issues)
         let mut file = OpenOptions::new()
             .create(true)
             .append(true)
             .open(&audit_path)
             .expect("Failed to open audit log");
         file.write_all(log_entry.as_bytes()).unwrap();
         file.flush().unwrap();
     }

    /// Store API key in encrypted vault
    pub fn set_secret(&self, key: &str, value: &str) -> Result<(), VaultError> {
        // Read current vault
        let vault_json = fs::read_to_string(&self.vault_path)?;
        let mut vault: VaultData = serde_json::from_str(&vault_json)
            .map_err(|_| VaultError::Corrupted)?;
        
        // Decode existing data
        let combined = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &vault.encrypted_data)
            .map_err(|_| VaultError::Corrupted)?;
        
        let nonce = Nonce::from_slice(&combined[..12]);
        let ciphertext = &combined[12..];
        
        // Decrypt existing data
        let decrypted = self.cipher.decrypt(nonce, ciphertext)
            .map_err(|e| VaultError::Decryption(e.to_string()))?;
        
        let mut secrets: HashMap<String, String> = serde_json::from_slice(&decrypted)
            .map_err(|_| VaultError::Corrupted)?;
        
        // Add/update secret
        secrets.insert(key.to_string(), value.to_string());
        
        // Re-encrypt
        let json = serde_json::to_vec(&secrets)
            .map_err(|e| VaultError::Encryption(e.to_string()))?;
        
        let mut nonce_bytes = [0u8; 12];
        use rand::RngCore;
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let ciphertext = self.cipher.encrypt(nonce, json.as_ref())
            .map_err(|e| VaultError::Encryption(e.to_string()))?;
        
        // Combine and save
        let mut combined = Vec::with_capacity(12 + ciphertext.len());
        combined.extend_from_slice(&nonce_bytes);
        combined.extend_from_slice(&ciphertext);
        
        vault.encrypted_data = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &combined);
        
        let vault_json = serde_json::to_string_pretty(&vault)
            .map_err(|e| VaultError::Encryption(e.to_string()))?;
        
        fs::write(&self.vault_path, vault_json)?;
        
        Ok(())
    }

    /// Retrieve API key from encrypted vault
    pub fn get_secret(&self, key: &str) -> Result<SecretString, VaultError> {
        // Read current vault
        let vault_json = fs::read_to_string(&self.vault_path)?;
        let vault: VaultData = serde_json::from_str(&vault_json)
            .map_err(|_| VaultError::Corrupted)?;
        
        // Decode existing data
        let combined = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &vault.encrypted_data)
            .map_err(|_| VaultError::Corrupted)?;
        
        let nonce = Nonce::from_slice(&combined[..12]);
        let ciphertext = &combined[12..];
        
        // Decrypt
        let decrypted = self.cipher.decrypt(nonce, ciphertext)
            .map_err(|e| VaultError::Decryption(e.to_string()))?;
        
        let secrets: HashMap<String, String> = serde_json::from_slice(&decrypted)
            .map_err(|_| VaultError::Corrupted)?;
        
        secrets.get(key)
            .map(|v| SecretString::new(v.clone()))
            .ok_or(VaultError::NotFound)
    }

    /// Load all secrets as environment variables
    pub fn load_to_env(&self) -> Result<(), VaultError> {
        // Read current vault
        let vault_json = fs::read_to_string(&self.vault_path)?;
        let vault: VaultData = serde_json::from_str(&vault_json)
            .map_err(|_| VaultError::Corrupted)?;
        
        // Decode existing data
        let combined = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &vault.encrypted_data)
            .map_err(|_| VaultError::Corrupted)?;
        
        let nonce = Nonce::from_slice(&combined[..12]);
        let ciphertext = &combined[12..];
        
        // Decrypt
        let decrypted = self.cipher.decrypt(nonce, ciphertext)
            .map_err(|e| VaultError::Decryption(e.to_string()))?;
        
        let secrets: HashMap<String, String> = serde_json::from_slice(&decrypted)
            .map_err(|_| VaultError::Corrupted)?;
        
        // Set as environment variables
        for (key, value) in secrets {
            std::env::set_var(&key, &value);
        }
        
        Ok(())
    }

    /// Check if vault exists
    pub fn exists(&self) -> bool {
        self.vault_path.exists()
    }

    /// Get the vault file path (for the Copilot System Access layer).
    pub fn vault_path(&self) -> &std::path::Path {
        &self.vault_path
    }
}

/// Legacy .env compatibility layer
/// Encrypts existing .env files into the vault
pub fn migrate_dotenv_to_vault(
    dotenv_path: &str,
    vault_path: &PathBuf,
    master_password: &str,
) -> Result<(), VaultError> {
    // Read .env file
    let dotenv_content = fs::read_to_string(dotenv_path)?;
    
    // Create new vault with all .env keys
    let vault = EnvVault::create(vault_path.clone(), master_password)?;
    
    // Parse and store each key
    for line in dotenv_content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim().trim_matches('"').trim_matches('\'');
            
            // Skip non-secret config
            if !key.contains("KEY") && !key.contains("SECRET") && !key.contains("PASSWORD") {
                continue;
            }
            
            vault.set_secret(key, value)?;
        }
    }
    
    info!("Migrated .env to encrypted vault");
    Ok(())
}

/// Secure API key retrieval with fallback to .env
pub fn get_api_key(key: &str) -> Option<String> {
    // First try environment variables (set by vault)
    if let Ok(value) = std::env::var(key) {
        return Some(value);
    }
    
    // Fallback: try standard dotenv (for development)
    if let Ok(value) = dotenvy::var(key) {
        tracing::warn!(
            "SECURITY: {} loaded from plaintext .env. Consider migrating to vault.",
            key
        );
        return Some(value);
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use secrecy::ExposeSecret;
    
    #[test]
    fn test_vault_create_and_open() {
        let temp_path = env::temp_dir().join("test_vault.json");
        let password = "test_password_123";
        
        // Create vault
        let vault = EnvVault::create(temp_path.clone(), password).unwrap();
        
        // Add secret
        vault.set_secret("TEST_API_KEY", "secret_value_123").unwrap();
        
        // Re-open vault
        let vault2 = EnvVault::open(temp_path.clone(), password).unwrap();
        
        // Retrieve secret
        let secret = vault2.get_secret("TEST_API_KEY").unwrap();
        assert_eq!(secret.expose_secret(), "secret_value_123");
        
        // Cleanup
        let _ = fs::remove_file(temp_path);
    }
    
    #[test]
    fn test_wrong_password() {
        let temp_path = env::temp_dir().join("test_vault2.json");
        let password = "correct_password";
        
        // Create vault
        let _ = EnvVault::create(temp_path.clone(), password).unwrap();
        
        // Try to open with wrong password
        let result = EnvVault::open(temp_path.clone(), "wrong_password");
        assert!(result.is_err());
        
        // Cleanup
        let _ = fs::remove_file(temp_path);
    }
}
