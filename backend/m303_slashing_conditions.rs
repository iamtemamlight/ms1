// ==============================================================================
// M303: Governance Slashing Conditions
// Purpose: Automated slashing for governance violations
//          Enforces economic deterrence for malicious/negligent behavior
// CGM Subsystem: Governance / Security
// ==============================================================================

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// Slashing offense types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SlashingOffense {
    DoubleVoting,
    VoteManipulation,
    ProposalSpam,
    TimelockExploit,
    GovernanceAttack,
    ValidatorMisbehavior,
    EmergencyAbuse,
    UnauthorizedExecution,
}

impl SlashingOffense {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DoubleVoting => "DOUBLE_VOTING",
            Self::VoteManipulation => "VOTE_MANIPULATION",
            Self::ProposalSpam => "PROPOSAL_SPAM",
            Self::TimelockExploit => "TIMELOCK_EXPLOIT",
            Self::GovernanceAttack => "GOVERNANCE_ATTACK",
            Self::ValidatorMisbehavior => "VALIDATOR_MISBEHAVIOR",
            Self::EmergencyAbuse => "EMERGENCY_ABUSE",
            Self::UnauthorizedExecution => "UNAUTHORIZED_EXECUTION",
        }
    }

    pub fn default_penalty_pct(&self) -> f64 {
        match self {
            Self::DoubleVoting => 0.05,
            Self::VoteManipulation => 0.10,
            Self::ProposalSpam => 0.02,
            Self::TimelockExploit => 0.50,
            Self::GovernanceAttack => 1.0,
            Self::ValidatorMisbehavior => 0.10,
            Self::EmergencyAbuse => 0.25,
            Self::UnauthorizedExecution => 0.15,
        }
    }

    pub fn jail_duration_blocks(&self) -> u64 {
        match self {
            Self::DoubleVoting => 100,
            Self::VoteManipulation => 500,
            Self::ProposalSpam => 50,
            Self::TimelockExploit => 1000,
            Self::GovernanceAttack => 5000,
            Self::ValidatorMisbehavior => 300,
            Self::EmergencyAbuse => 500,
            Self::UnauthorizedExecution => 300,
        }
    }
}

/// Slashing record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlashingRecord {
    pub record_id: String,
    pub offender: String,
    pub offense: SlashingOffense,
    pub penalty_percentage: f64,
    pub slashed_amount: f64,
    pub block_number: u64,
    pub timestamp_ms: u64,
    pub evidence: String,
    pub processed: bool,
    pub appealed: bool,
    pub appeal_deadline_ms: u64,
}

/// Slashing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlashingConfig {
    pub enabled: bool,
    pub auto_slash: bool,
    pub penalty_multiplier: f64,
    pub jail_duration_multiplier: f64,
    pub appeal_period_blocks: u64,
    pub min_slash_amount: f64,
    pub max_slash_percentage: f64,
    pub governance_whitelist: Vec<String>,
}

impl Default for SlashingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_slash: true,
            penalty_multiplier: 1.0,
            jail_duration_multiplier: 1.0,
            appeal_period_blocks: 100,
            min_slash_amount: 0.01,
            max_slash_percentage: 1.0,
            governance_whitelist: Vec::new(),
        }
    }
}

/// Slashing Conditions Engine
#[derive(Debug, Clone)]
pub struct SlashingEngine {
    pub config: SlashingConfig,
    pub slashing_records: Arc<RwLock<HashMap<String, SlashingRecord>>>,
    pub jailed_addresses: Arc<RwLock<HashMap<String, u64>>>, // address -> unjail block
    pub offense_counts: Arc<RwLock<HashMap<String, HashMap<SlashingOffense, u32>>>>,
    pub total_slashed: Arc<RwLock<f64>>,
    pub total_offenses: u64,
    pub appeal_queue: Arc<RwLock<Vec<SlashingRecord>>>,
}

impl SlashingEngine {
    pub fn new(config: SlashingConfig) -> Self {
        Self {
            config,
            slashing_records: Arc::new(RwLock::new(HashMap::new())),
            jailed_addresses: Arc::new(RwLock::new(HashMap::new())),
            offense_counts: Arc::new(RwLock::new(HashMap::new())),
            total_slashed: Arc::new(RwLock::new(0.0)),
            total_offenses: 0,
            appeal_queue: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(SlashingConfig::default())
    }

    /// Record an offense and calculate penalty
    pub async fn record_offense(
        &mut self,
        offender: &str,
        offense: SlashingOffense,
        evidence: &str,
        current_block: u64,
        stake_amount: f64,
    ) -> SlashingRecord {
        self.total_offenses += 1;

        // Check if address is whitelisted
        if self.config.governance_whitelist.contains(&offender.to_string()) {
            tracing::warn!("Slashing skipped for whitelisted address: {}", offender);
            return SlashingRecord {
                record_id: format!("slash-{}-{}", offender, current_block),
                offender: offender.to_string(),
                offense,
                penalty_percentage: 0.0,
                slashed_amount: 0.0,
                block_number: current_block,
                timestamp_ms: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0),
                evidence: evidence.to_string(),
                processed: true,
                appealed: false,
                appeal_deadline_ms: 0,
            };
        }

        // Calculate penalty with escalation for repeat offenses
        let base_penalty = offense.default_penalty_pct();
        let repeat_multiplier = self.get_repeat_offense_multiplier(offender, offense).await;
        let penalty_pct = (base_penalty * self.config.penalty_multiplier * repeat_multiplier)
            .min(self.config.max_slash_percentage);

        let slashed_amount = (stake_amount * penalty_pct).max(self.config.min_slash_amount);
        let jail_blocks = (offense.jail_duration_blocks() as f64 * self.config.jail_duration_multiplier) as u64;
        let unjail_block = current_block + jail_blocks;

        // Record slashing
        let record = SlashingRecord {
            record_id: format!("slash-{}-{}", offender, current_block),
            offender: offender.to_string(),
            offense,
            penalty_percentage: penalty_pct,
            slashed_amount,
            block_number: current_block,
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
            evidence: evidence.to_string(),
            processed: self.config.auto_slash,
            appealed: false,
            appeal_deadline_ms: if self.config.appeal_period_blocks > 0 {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as u64 + self.config.appeal_period_blocks * 12000)
                    .unwrap_or(0)
            } else {
                0
            },
        };

        // Update state
        if self.config.auto_slash {
            self.slashing_records.write().await.insert(record.record_id.clone(), record.clone());
            self.jailed_addresses.write().await.insert(offender.to_string(), unjail_block);
            *self.total_slashed.write().await += slashed_amount;

            // Update offense counts
            let mut counts = self.offense_counts.write().await;
            counts.entry(offender.to_string())
                .or_default()
                .entry(offense)
                .and_modify(|c| *c += 1)
                .or_insert(1);
        }

        tracing::warn!(
            "SLASHING: {} | {} | penalty={:.2}% | amount={:.4} | jail={} blocks",
            offender,
            offense.as_str(),
            penalty_pct * 100.0,
            slashed_amount,
            jail_blocks
        );

        record
    }

    /// Get repeat offense multiplier
    async fn get_repeat_offense_multiplier(&self, offender: &str, offense: SlashingOffense) -> f64 {
        let counts = self.offense_counts.read().await;
        if let Some(offender_counts) = counts.get(offender) {
            let total_offenses: u32 = offender_counts.values().sum();
            let same_offense_count = offender_counts.get(&offense).copied().unwrap_or(0);
            
            // Escalating penalties: 1.0x, 1.5x, 2.0x, 3.0x, 5.0x
            match total_offenses {
                0 => 1.0,
                1 => 1.0,
                2 => 1.5,
                3 => 2.0,
                4 => 3.0,
                _ => 5.0,
            }
        } else {
            1.0
        }
    }

    /// Check if address is jailed
    pub async fn is_jailed(&self, address: &str, current_block: u64) -> bool {
        let jailed = self.jailed_addresses.read().await;
        if let Some(&unjail_block) = jailed.get(address) {
            if current_block >= unjail_block {
                // Auto-unjail
                drop(jailed);
                let mut j = self.jailed_addresses.write().await;
                j.remove(address);
                return false;
            }
            return true;
        }
        false
    }

    /// Appeal a slashing decision
    pub async fn appeal_slashing(&self, record_id: &str, appellant: &str) -> Result<(), String> {
        let mut records = self.slashing_records.write().await;
        let record = records.get_mut(record_id).ok_or("Slashing record not found")?;

        if record.offender != appellant {
            return Err("Only the offender can appeal".to_string());
        }

        if record.appealed {
            return Err("Already appealed".to_string());
        }

        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        if now_ms > record.appeal_deadline_ms && record.appeal_deadline_ms > 0 {
            return Err("Appeal period expired".to_string());
        }

        record.appealed = true;
        self.appeal_queue.write().await.push(record.clone());

        tracing::info!("SLASHING APPEAL: {} by {}", record_id, appellant);
        Ok(())
    }

    /// Process appeal queue
    pub async fn process_appeals(&self, approval_threshold: f64) -> Vec<String> {
        let mut resolved = Vec::new();
        let mut appeals = self.appeal_queue.write().await;
        
        for appeal in appeals.iter() {
            // In production, this would involve governance voting
            // For now, approve appeals with >50% chance (simulated)
            if rand::random::<f64>() > approval_threshold {
                tracing::info!("SLASHING APPEAL APPROVED: {}", appeal.record_id);
                resolved.push(appeal.record_id.clone());
            } else {
                tracing::warn!("SLASHING APPEAL DENIED: {}", appeal.record_id);
            }
        }

        appeals.clear();
        resolved
    }

    /// Get slashing statistics
    pub async fn get_statistics(&self) -> SlashingStatistics {
        let total_records = self.slashing_records.read().await.len();
        let total_jailed = self.jailed_addresses.read().await.len();
        let total_appeals = self.appeal_queue.read().await.len();

        SlashingStatistics {
            total_offenses: self.total_offenses,
            total_slashed: *self.total_slashed.read().await,
            total_records,
            total_jailed,
            pending_appeals: total_appeals,
        }
    }

    /// Get offense history for an address
    pub async fn get_offense_history(&self, address: &str) -> Vec<SlashingRecord> {
        let records = self.slashing_records.read().await;
        records.values()
            .filter(|r| r.offender == address)
            .cloned()
            .collect()
    }

    /// Clear jail for an address (admin function)
    pub async fn clear_jail(&self, address: &str) -> Result<(), String> {
        self.jailed_addresses.write().await.remove(address);
        Ok(())
    }
}

/// Slashing statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SlashingStatistics {
    pub total_offenses: u64,
    pub total_slashed: f64,
    pub total_records: usize,
    pub total_jailed: usize,
    pub pending_appeals: usize,
}

impl Default for SlashingEngine {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_slashing_offense() {
        let mut engine = SlashingEngine::with_defaults();
        let record = engine.record_offense(
            "0xmalicious",
            SlashingOffense::DoubleVoting,
            "Voted twice on proposal 123",
            18000000,
            1000.0,
        ).await;
        assert!(record.slashed_amount > 0.0);
    }

    #[tokio::test]
    async fn test_jail_status() {
        let mut engine = SlashingEngine::with_defaults();
        let _ = engine.record_offense(
            "0xmalicious",
            SlashingOffense::DoubleVoting,
            "Evidence",
            18000000,
            1000.0,
        ).await;
        assert!(engine.is_jailed("0xmalicious", 18000000).await);
        assert!(!engine.is_jailed("0xmalicious", 18000100).await);
    }
}
