#![allow(dead_code)]
use dashmap::DashMap;
use std::sync::Arc;
use keyring::Entry;
use k256::ecdsa::{SigningKey, Signature, RecoveryId};

#[derive(Debug, Clone)]
pub struct KeyManager {
    cache: Arc<DashMap<String, SigningKey>>,
}

impl KeyManager {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
        }
    }

    /// Loads a private key directly from a hex string (e.g., from .env PRIVATE_KEY).
    pub fn load_private_key_hex(&self, label: &str, hex_key: &str) -> Result<(), String> {
        let key_bytes = hex::decode(hex_key.trim_start_matches("0x"))
            .map_err(|e| format!("Hex decode error: {}", e))?;
        let signing_key = SigningKey::from_slice(&key_bytes)
            .map_err(|e| format!("Invalid signing key: {}", e))?;
        self.cache.insert(label.to_string(), signing_key);
        Ok(())
    }

    /// Fetches a key from the OS Keychain and stores it in the encrypted RAM cache.
    /// This should be called when the fleet starts or a new wallet is activated.
    pub async fn unlock_key(&self, label: &str) -> Result<(), String> {
        let entry = Entry::new("allbright-fleet", label)
            .map_err(|e| format!("Keychain access error: {}", e))?;
        
        let key_data = entry.get_password()
            .map_err(|e| format!("Failed to retrieve key '{}': {}", label, e))?;
            
        let key_bytes = hex::decode(&key_data).map_err(|e| format!("Hex decode error: {}", e))?;
        let signing_key = SigningKey::from_slice(&key_bytes)
            .map_err(|e| format!("Invalid signing key: {}", e))?;

        self.cache.insert(label.to_string(), signing_key);
        Ok(())
    }

    /// Signs a 32-byte message hash using the cached private key.
    pub fn sign_hash(&self, label: &str, hash: &[u8; 32]) -> Result<String, String> {
        let signing_key = self.cache.get(label)
            .ok_or_else(|| format!("Key '{}' not found in cache. Ensure wallet is unlocked.", label))?;
        
        let (signature, _recid): (Signature, RecoveryId) = signing_key.sign_prehash_recoverable(hash)
            .map_err(|e| format!("Signing failed: {}", e))?;
            
        Ok(hex::encode(signature.to_bytes()))
    }

    /// Purges RAM cache. Use this for emergency "Lock" or app shutdown.
    pub fn clear_cache(&self) {
        self.cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_manager_new() {
        let km = KeyManager::new();
        assert!(km.cache.is_empty());
    }

    #[test]
    fn test_key_manager_clear_cache() {
        let km = KeyManager::new();
        km.cache.insert("test".to_string(), SigningKey::from_slice(&[1u8; 32]).unwrap());
        assert_eq!(km.cache.len(), 1);
        km.clear_cache();
        assert!(km.cache.is_empty());
    }

    #[test]
    fn test_sign_hash_missing_key() {
        let km = KeyManager::new();
        let hash = [0u8; 32];
        let result = km.sign_hash("nonexistent", &hash);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found in cache"));
    }

    #[tokio::test]
    async fn test_unlock_key_invalid_hex() {
        let km = KeyManager::new();
        let result = km.unlock_key("invalid-hex-key").await;
        assert!(result.is_err());
    }
}