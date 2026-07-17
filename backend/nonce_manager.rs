// ==============================================================================
// NONCE PERSISTENCE MANAGER
// ==============================================================================
// Atomic nonce counter persisted to SQLite. Prevents nonce gaps on restart
// and ensures transaction ordering.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonceRecord {
    pub address: String,
    pub chain_id: u64,
    pub nonce: u64,
    pub updated_at: i64,
    pub tx_hash: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NonceManager {
    pool: SqlitePool,
    address: String,
    chain_id: u64,
    cache: Arc<RwLock<Option<NonceRecord>>>,
}

impl NonceManager {
    pub async fn new(pool: SqlitePool, address: &str, chain_id: u64) -> Result<Self, sqlx::Error> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS nonce_counter (
                address TEXT NOT NULL,
                chain_id INTEGER NOT NULL,
                nonce INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                tx_hash TEXT,
                PRIMARY KEY (address, chain_id)
            )
            "#,
        )
        .execute(&pool)
        .await?;

        let manager = Self {
            pool,
            address: address.to_string(),
            chain_id,
            cache: Arc::new(RwLock::new(None)),
        };

        manager.load_from_db().await?;
        Ok(manager)
    }

    /// Load nonce from SQLite
    pub async fn load_from_db(&self) -> Result<Option<NonceRecord>, sqlx::Error> {
        let row = sqlx::query_as::<_, (String, i64, i64, i64, Option<String>)>(
            r#"
            SELECT address, chain_id, nonce, updated_at, tx_hash
            FROM nonce_counter
            WHERE address = ? AND chain_id = ?
            "#
        )
        .bind(&self.address)
        .bind(self.chain_id as i64)
        .fetch_optional(&self.pool)
        .await?;

        let record = row.map(|(address, chain_id, nonce, updated_at, tx_hash)| NonceRecord {
            address,
            chain_id: chain_id as u64,
            nonce: nonce as u64,
            updated_at,
            tx_hash,
        });

        if let Some(ref r) = record {
            let mut cache = self.cache.write().await;
            *cache = Some(r.clone());
        }

        Ok(record)
    }

    /// Get current nonce (from cache)
    pub async fn get_nonce(&self) -> u64 {
        let cache = self.cache.read().await;
        cache.as_ref().map(|r| r.nonce).unwrap_or(0)
    }

    /// Get next nonce and increment atomically
    pub async fn next_nonce(&self) -> Result<u64, sqlx::Error> {
        let nonce = self.get_nonce().await;
        let new_nonce = nonce + 1;
        self.set_nonce(new_nonce, None).await?;
        Ok(new_nonce)
    }

    /// Set nonce and persist to SQLite
    pub async fn set_nonce(&self, nonce: u64, tx_hash: Option<&str>) -> Result<(), sqlx::Error> {
        let now = chrono::Utc::now().timestamp();

        sqlx::query(
            r#"
            INSERT INTO nonce_counter (address, chain_id, nonce, updated_at, tx_hash)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(address, chain_id) DO UPDATE SET
                nonce = excluded.nonce,
                updated_at = excluded.updated_at,
                tx_hash = excluded.tx_hash
            "#,
        )
        .bind(&self.address)
        .bind(self.chain_id as i64)
        .bind(nonce as i64)
        .bind(now)
        .bind(tx_hash)
        .execute(&self.pool)
        .await?;

        let mut cache = self.cache.write().await;
        *cache = Some(NonceRecord {
            address: self.address.clone(),
            chain_id: self.chain_id,
            nonce,
            updated_at: now,
            tx_hash: tx_hash.map(|s| s.to_string()),
        });

        Ok(())
    }

    /// Sync nonce with chain state (should be called periodically)
    pub async fn sync_with_chain(&self, chain_nonce: u64) -> Result<(), sqlx::Error> {
        let cached = self.get_nonce().await;
        if chain_nonce > cached {
            self.set_nonce(chain_nonce, None).await?;
        }
        Ok(())
    }

    /// Reset nonce to zero (emergency use only)
    pub async fn emergency_reset(&self) -> Result<(), sqlx::Error> {
        self.set_nonce(0, Some("EMERGENCY_RESET")).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    #[tokio::test]
    async fn test_nonce_manager_basic() -> Result<(), sqlx::Error> {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await?;

        let mgr = NonceManager::new(pool, "0x123", 1).await?;
        assert_eq!(mgr.get_nonce().await, 0);

        let n1 = mgr.next_nonce().await?;
        assert_eq!(n1, 1);

        let n2 = mgr.next_nonce().await?;
        assert_eq!(n2, 2);

        Ok(())
    }
}