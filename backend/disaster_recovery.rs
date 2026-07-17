// ==============================================================================
// MODULE: Disaster Recovery Protocol
// Purpose: Implements fleet state checkpointing and instant restore capability
//          Enables full fleet recovery in <5 minutes
// ==============================================================================

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Fleet state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetSnapshot {
    pub id: String,
    pub block_number: u64,
    pub timestamp: u64,
    pub runner_states: HashMap<String, RunnerState>,
    pub total_profit: f64,
    pub fleet_health_score: f64,
    pub snapshot_type: SnapshotType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SnapshotType {
    Full,
    Incremental,
    Checkpoint,
}

/// Individual runner state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunnerState {
    pub runner_id: String,
    pub wallet_address: String,
    pub current_profit: f64,
    pub last_execution_block: u64,
    pub health_score: f64,
    pub active_positions: u32,
}

/// Disaster Recovery Manager
pub struct DisasterRecoveryManager {
    snapshots: Vec<FleetSnapshot>,
    max_snapshots: usize,
    checkpoint_interval_blocks: u64,
    last_checkpoint_block: u64,
    auto_recovery_enabled: bool,
}

impl DisasterRecoveryManager {
    pub fn new() -> Self {
        Self {
            snapshots: Vec::new(),
            max_snapshots: 10,
            checkpoint_interval_blocks: 1000,
            last_checkpoint_block: 0,
            auto_recovery_enabled: true,
        }
    }

    /// Create a new fleet snapshot
    pub fn create_snapshot(
        &mut self,
        block_number: u64,
        runner_states: HashMap<String, RunnerState>,
    ) -> Result<FleetSnapshot, String> {
        // Calculate totals
        let total_profit: f64 = runner_states.values().map(|r| r.current_profit).sum();
        let fleet_health_score: f64 = if runner_states.is_empty() {
            0.0
        } else {
            runner_states.values().map(|r| r.health_score).sum::<f64>() / runner_states.len() as f64
        };

        let snapshot = FleetSnapshot {
            id: format!("snap_{}", block_number),
            block_number,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            runner_states,
            total_profit,
            fleet_health_score,
            snapshot_type: SnapshotType::Checkpoint,
        };

        // Add to snapshots list
        if self.snapshots.len() >= self.max_snapshots {
            self.snapshots.remove(0);
        }
        self.snapshots.push(snapshot.clone());
        self.last_checkpoint_block = block_number;

        Ok(snapshot)
    }

    /// Get the latest snapshot
    pub fn get_latest(&self) -> Option<&FleetSnapshot> {
        self.snapshots.last()
    }

    /// Get snapshot by ID
    pub fn get_by_id(&self, id: &str) -> Option<&FleetSnapshot> {
        self.snapshots.iter().find(|s| s.id == id)
    }

    /// Restore fleet from snapshot
    pub fn restore_from_snapshot(&self, snapshot_id: &str) -> Result<HashMap<String, RunnerState>, String> {
        let snapshot = self.get_by_id(snapshot_id).ok_or("Snapshot not found")?;
        Ok(snapshot.runner_states.clone())
    }

    /// Check if checkpoint is needed based on block number
    pub fn needs_checkpoint(&self, current_block: u64) -> bool {
        current_block >= self.last_checkpoint_block + self.checkpoint_interval_blocks
    }

    /// Get time since last snapshot
    pub fn time_since_last_snapshot(&self) -> u64 {
        let latest = self.get_latest();
        match latest {
            Some(s) => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                now.saturating_sub(s.timestamp)
            }
            None => u64::MAX,
        }
    }

    /// Calculate recovery time estimate (in seconds)
    pub fn estimated_recovery_time(&self, runner_count: u32) -> u64 {
        // Estimate: 1 second per 10 runners, plus 60 seconds base
        // Full recovery target: <5 minutes (300 seconds)
        let estimated = (runner_count as u64 / 10) + 60;
        estimated.min(300)  // Cap at 5 minutes
    }

    /// Enable/disable auto-recovery
    pub fn set_auto_recovery(&mut self, enabled: bool) {
        self.auto_recovery_enabled = enabled;
    }

    /// Get checkpoint history
    pub fn get_checkpoint_history(&self) -> Vec<(String, u64, f64)> {
        self.snapshots
            .iter()
            .map(|s| (s.id.clone(), s.block_number, s.fleet_health_score))
            .collect()
    }
}

impl Default for DisasterRecoveryManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_creation() {
        let mut dr = DisasterRecoveryManager::new();
        let mut runner_states = HashMap::new();
        
        runner_states.insert(
            "runner_001".to_string(),
            RunnerState {
                runner_id: "runner_001".to_string(),
                wallet_address: "0x1234".to_string(),
                current_profit: 1.5,
                last_execution_block: 1000,
                health_score: 0.95,
                active_positions: 2,
            },
        );
        
        let snapshot = dr.create_snapshot(1000, runner_states).unwrap();
        assert_eq!(snapshot.block_number, 1000);
    }

    #[test]
    fn test_restore_from_snapshot() {
        let mut dr = DisasterRecoveryManager::new();
        let mut runner_states = HashMap::new();
        
        runner_states.insert(
            "runner_001".to_string(),
            RunnerState {
                runner_id: "runner_001".to_string(),
                wallet_address: "0x1234".to_string(),
                current_profit: 1.5,
                last_execution_block: 1000,
                health_score: 0.95,
                active_positions: 2,
            },
        );
        
        let snapshot = dr.create_snapshot(1000, runner_states).unwrap();
        let restored = dr.restore_from_snapshot(&snapshot.id).unwrap();
        
        assert_eq!(restored.len(), 1);
    }

    #[test]
    fn test_recovery_time_estimate() {
        let dr = DisasterRecoveryManager::new();
        let time = dr.estimated_recovery_time(100);
        assert_eq!(time, 70);  // 100/10 + 60
    }
}
