// ==============================================================================
// M302: Cross-Chain Governance Sync
// Purpose: Synchronize governance state across EVM and SVM chains
//          Ensures unified governance across multi-chain deployments
// CGM Subsystem: Governance / Continuity
// ==============================================================================

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use sha2::Digest;

/// Supported chain types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChainType {
    Ethereum,
    Arbitrum,
    Base,
    Optimism,
    Polygon,
    BSC,
    Solana,
    Avalanche,
    Fantom,
    Gnosis,
    Linea,
    Zora,
    Mantle,
    Metis,
}

impl ChainType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ethereum => "ethereum",
            Self::Arbitrum => "arbitrum",
            Self::Base => "base",
            Self::Optimism => "optimism",
            Self::Polygon => "polygon",
            Self::BSC => "bsc",
            Self::Solana => "solana",
            Self::Avalanche => "avalanche",
            Self::Fantom => "fantom",
            Self::Gnosis => "gnosis",
            Self::Linea => "linea",
            Self::Zora => "zora",
            Self::Mantle => "mantle",
            Self::Metis => "metis",
        }
    }

    pub fn is_evm(&self) -> bool {
        !matches!(self, Self::Solana)
    }
}

/// Governance state snapshot for a chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainGovernanceState {
    pub chain: ChainType,
    pub last_sync_block: u64,
    pub last_sync_timestamp_ms: u64,
    pub active_proposals: Vec<u64>,
    pub pending_executions: Vec<u64>,
    pub voting_power_snapshot: HashMap<String, u64>,
    pub timelock_delay_ms: u64,
    pub governance_paused: bool,
    pub checksum: String,
}

/// Cross-chain governance sync configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossChainSyncConfig {
    pub primary_chain: ChainType,
    pub sync_interval_ms: u64,
    pub confirmation_blocks: u32,
    pub max_clock_drift_ms: u64,
    pub enabled_chains: Vec<ChainType>,
    pub relay_enabled: bool,
}

impl Default for CrossChainSyncConfig {
    fn default() -> Self {
        Self {
            primary_chain: ChainType::Ethereum,
            sync_interval_ms: 300_000, // 5 minutes
            confirmation_blocks: 12,
            max_clock_drift_ms: 5000, // 5 seconds
            enabled_chains: vec![
                ChainType::Ethereum,
                ChainType::Arbitrum,
                ChainType::Base,
                ChainType::Optimism,
                ChainType::Polygon,
            ],
            relay_enabled: true,
        }
    }
}

/// Cross-chain governance message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceSyncMessage {
    pub message_id: String,
    pub source_chain: ChainType,
    pub target_chain: ChainType,
    pub message_type: SyncMessageType,
    pub payload: serde_json::Value,
    pub timestamp_ms: u64,
    pub block_number: u64,
    pub signature: String,
    pub processed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncMessageType {
    ProposalCreated,
    VoteCast,
    ProposalPassed,
    ProposalExecuted,
    ProposalCancelled,
    TimelockUpdate,
    GovernancePaused,
    GovernanceResumed,
    VotingPowerUpdate,
}

impl SyncMessageType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ProposalCreated => "PROPOSAL_CREATED",
            Self::VoteCast => "VOTE_CAST",
            Self::ProposalPassed => "PROPOSAL_PASSED",
            Self::ProposalExecuted => "PROPOSAL_EXECUTED",
            Self::ProposalCancelled => "PROPOSAL_CANCELLED",
            Self::TimelockUpdate => "TIMELOCK_UPDATE",
            Self::GovernancePaused => "GOVERNANCE_PAUSED",
            Self::GovernanceResumed => "GOVERNANCE_RESUMED",
            Self::VotingPowerUpdate => "VOTING_POWER_UPDATE",
        }
    }
}

/// Cross-chain governance synchronizer
#[derive(Debug, Clone)]
pub struct CrossChainGovernanceSync {
    pub config: CrossChainSyncConfig,
    pub chain_states: Arc<RwLock<HashMap<ChainType, ChainGovernanceState>>>,
    pub message_queue: Arc<RwLock<Vec<GovernanceSyncMessage>>>,
    pub processed_messages: Arc<RwLock<HashMap<String, GovernanceSyncMessage>>>,
    pub last_sync_timestamp: Arc<RwLock<HashMap<ChainType, u64>>>,
    pub sync_statistics: Arc<RwLock<SyncStatistics>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SyncStatistics {
    pub messages_sent: u64,
    pub messages_received: u64,
    pub messages_processed: u64,
    pub messages_failed: u64,
    pub last_sync_ms: u64,
    pub avg_sync_latency_ms: f64,
    pub chain_health: HashMap<ChainType, ChainHealthStatus>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChainHealthStatus {
    Healthy,
    Degraded,
    Offline,
    Syncing,
}

impl ChainHealthStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Healthy => "HEALTHY",
            Self::Degraded => "DEGRADED",
            Self::Offline => "OFFLINE",
            Self::Syncing => "SYNCING",
        }
    }
}

impl CrossChainGovernanceSync {
    pub fn new(config: CrossChainSyncConfig) -> Self {
        let mut chain_states = HashMap::new();
        for &chain in &config.enabled_chains {
            chain_states.insert(chain, ChainGovernanceState {
                chain,
                last_sync_block: 0,
                last_sync_timestamp_ms: 0,
                active_proposals: Vec::new(),
                pending_executions: Vec::new(),
                voting_power_snapshot: HashMap::new(),
                timelock_delay_ms: 24 * 60 * 60 * 1000,
                governance_paused: false,
                checksum: String::new(),
            });
        }

        Self {
            config,
            chain_states: Arc::new(RwLock::new(chain_states)),
            message_queue: Arc::new(RwLock::new(Vec::new())),
            processed_messages: Arc::new(RwLock::new(HashMap::new())),
            last_sync_timestamp: Arc::new(RwLock::new(HashMap::new())),
            sync_statistics: Arc::new(RwLock::new(SyncStatistics::default())),
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(CrossChainSyncConfig::default())
    }

    /// Create a sync message for cross-chain propagation
    pub fn create_sync_message(
        &self,
        source_chain: ChainType,
        target_chain: ChainType,
        message_type: SyncMessageType,
        payload: serde_json::Value,
        block_number: u64,
    ) -> GovernanceSyncMessage {
        let message_id = format!(
            "{}-{}-{}-{}",
            source_chain.as_str(),
            target_chain.as_str(),
            message_type.as_str(),
            block_number
        );

        let timestamp_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        // In production, this would be a real signature
        let signature = format!("0x{}", hex::encode(sha2::Sha256::digest(message_id.as_bytes())));

        GovernanceSyncMessage {
            message_id,
            source_chain,
            target_chain,
            message_type,
            payload,
            timestamp_ms,
            block_number,
            signature,
            processed: false,
        }
    }

    /// Queue a message for cross-chain sync
    pub async fn queue_message(&self, message: GovernanceSyncMessage) {
        self.message_queue.write().await.push(message.clone());
        self.sync_statistics.write().await.messages_sent += 1;
        tracing::debug!(
            "CrossChainSync: Queued message {} from {} to {}",
            message.message_id,
            message.source_chain.as_str(),
            message.target_chain.as_str()
        );
    }

    /// Process incoming sync messages
    pub async fn process_message(&self, message: GovernanceSyncMessage) -> Result<(), String> {
        // Check for duplicates
        if self.processed_messages.read().await.contains_key(&message.message_id) {
            return Ok(());
        }

        // Verify signature (simplified)
        if !self.verify_signature(&message) {
            return Err("Invalid signature".to_string());
        }

        // Check clock drift
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        if now_ms > message.timestamp_ms + self.config.max_clock_drift_ms {
            return Err("Message too old".to_string());
        }

        // Apply message to local state
        self.apply_sync_message(&message).await?;

        // Mark as processed
        self.processed_messages.write().await.insert(message.message_id.clone(), message);
        self.sync_statistics.write().await.messages_processed += 1;

        Ok(())
    }

    /// Apply sync message to local chain state
    async fn apply_sync_message(&self, message: &GovernanceSyncMessage) -> Result<(), String> {
        let mut states = self.chain_states.write().await;
        let state = states.get_mut(&message.target_chain).ok_or("Target chain not configured")?;

        match message.message_type {
            SyncMessageType::ProposalCreated => {
                if let Some(proposal_id) = message.payload["proposal_id"].as_u64() {
                    state.active_proposals.push(proposal_id);
                }
            }
            SyncMessageType::ProposalExecuted => {
                if let Some(proposal_id) = message.payload["proposal_id"].as_u64() {
                    state.active_proposals.retain(|&id| id != proposal_id);
                    state.pending_executions.push(proposal_id);
                }
            }
            SyncMessageType::GovernancePaused => {
                state.governance_paused = true;
            }
            SyncMessageType::GovernanceResumed => {
                state.governance_paused = false;
            }
            SyncMessageType::TimelockUpdate => {
                if let Some(delay) = message.payload["delay_ms"].as_u64() {
                    state.timelock_delay_ms = delay;
                }
            }
            SyncMessageType::VotingPowerUpdate => {
                if let (Some(address), Some(power)) = (
                    message.payload["address"].as_str(),
                    message.payload["voting_power"].as_u64(),
                ) {
                    state.voting_power_snapshot.insert(address.to_string(), power);
                }
            }
            _ => {}
        }

        state.last_sync_timestamp_ms = message.timestamp_ms;
        state.last_sync_block = message.block_number;
        state.checksum = self.compute_state_checksum(state);

        Ok(())
    }

    /// Verify message signature
    fn verify_signature(&self, message: &GovernanceSyncMessage) -> bool {
        // In production, verify against known relayers
        // For now, accept all messages with valid format
        message.signature.len() == 66 && message.signature.starts_with("0x")
    }

    /// Compute state checksum for integrity verification
    fn compute_state_checksum(&self, state: &ChainGovernanceState) -> String {
        let data = format!(
            "{:?}:{}:{}",
            state.chain,
            state.last_sync_block,
            state.active_proposals.len()
        );
        hex::encode(sha2::Sha256::digest(data.as_bytes()))
    }

    /// Get synchronized state for a chain
    pub async fn get_chain_state(&self, chain: ChainType) -> Option<ChainGovernanceState> {
        self.chain_states.read().await.get(&chain).cloned()
    }

    /// Get all chain states
    pub async fn get_all_states(&self) -> HashMap<ChainType, ChainGovernanceState> {
        self.chain_states.read().await.clone()
    }

    /// Check if all chains are in sync
    pub async fn is_fully_synced(&self) -> bool {
        let states = self.chain_states.read().await;
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        states.values().all(|state| {
            now_ms - state.last_sync_timestamp_ms < self.config.sync_interval_ms * 2
        })
    }

    /// Get sync statistics
    pub async fn get_statistics(&self) -> SyncStatistics {
        self.sync_statistics.read().await.clone()
    }

    /// Force sync a specific chain
    pub async fn force_sync_chain(&self, chain: ChainType) -> Result<(), String> {
        let mut states = self.chain_states.write().await;
        if let Some(state) = states.get_mut(&chain) {
            let now_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            state.last_sync_timestamp_ms = now_ms;
            state.checksum = self.compute_state_checksum(state);
            Ok(())
        } else {
            Err("Chain not configured".to_string())
        }
    }
}

impl Default for CrossChainGovernanceSync {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cross_chain_sync() {
        let sync = CrossChainGovernanceSync::with_defaults();
        let message = sync.create_sync_message(
            ChainType::Ethereum,
            ChainType::Arbitrum,
            SyncMessageType::ProposalCreated,
            serde_json::json!({"proposal_id": 1}),
            18000000,
        );
        assert!(!message.message_id.is_empty());
    }

    #[tokio::test]
    async fn test_message_processing() {
        let sync = CrossChainGovernanceSync::with_defaults();
        let message = sync.create_sync_message(
            ChainType::Ethereum,
            ChainType::Arbitrum,
            SyncMessageType::GovernancePaused,
            serde_json::json!({}),
            18000000,
        );
        let result = sync.process_message(message).await;
        assert!(result.is_ok());
    }
}
