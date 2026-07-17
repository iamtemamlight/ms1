// ==============================================================================
// M301: Timelock Controller
// Purpose: Time-delayed execution of governance proposals
//          Enforces minimum delay between approval and execution
// CGM Subsystem: Governance / Security
// ==============================================================================

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use crate::m300_governance_executor::{GovernanceProposal, ProposalType, ProposalStatus};

/// Timelock entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelockEntry {
    pub proposal_id: u64,
    pub queued_at_ms: u64,
    pub execution_at_ms: u64,
    pub delay_ms: u64,
    pub executed: bool,
    pub cancelled: bool,
    pub tx_hash: Option<String>,
}

/// Timelock Controller
#[derive(Debug, Clone)]
pub struct TimelockController {
    pub min_delay_ms: u64,
    pub max_delay_ms: u64,
    pub grace_period_ms: u64,
    pub timelocks: Arc<RwLock<HashMap<u64, TimelockEntry>>>,
    pub pending_executions: Arc<RwLock<Vec<u64>>>,
    pub executed_count: u64,
    pub cancelled_count: u64,
}

impl TimelockController {
    pub fn new(min_delay_ms: u64, max_delay_ms: u64) -> Self {
        Self {
            min_delay_ms,
            max_delay_ms,
            grace_period_ms: 14 * 24 * 60 * 60 * 1000, // 14 days
            timelocks: Arc::new(RwLock::new(HashMap::new())),
            pending_executions: Arc::new(RwLock::new(Vec::new())),
            executed_count: 0,
            cancelled_count: 0,
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(24 * 60 * 60 * 1000, 30 * 24 * 60 * 60 * 1000) // 24h min, 30d max
    }

    /// Queue a proposal for timelock
    pub async fn queue_proposal(
        &self,
        proposal: &GovernanceProposal,
        custom_delay_ms: Option<u64>,
    ) -> Result<TimelockEntry, String> {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        let delay = custom_delay_ms.unwrap_or_else(|| {
            match proposal.proposal_type {
                ProposalType::EmergencyAction => self.min_delay_ms / 24, // 1 hour for emergency
                ProposalType::ParameterChange => self.min_delay_ms,
                ProposalType::PolicyUpdate => self.min_delay_ms * 2,
                ProposalType::CapitalAllocation => self.min_delay_ms * 3,
                ProposalType::ModuleUpgrade => self.max_delay_ms / 2,
            }
        }).clamp(self.min_delay_ms, self.max_delay_ms);

        let execution_at = now_ms + delay;

        let entry = TimelockEntry {
            proposal_id: proposal.proposal_id,
            queued_at_ms: now_ms,
            execution_at_ms: execution_at,
            delay_ms: delay,
            executed: false,
            cancelled: false,
            tx_hash: None,
        };

        self.timelocks.write().await.insert(proposal.proposal_id, entry.clone());
        self.pending_executions.write().await.push(proposal.proposal_id);

        Ok(entry)
    }

    /// Execute a queued proposal after timelock expires
    pub async fn execute_queued(&mut self, proposal_id: u64) -> Result<String, String> {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        let mut timelocks = self.timelocks.write().await;
        let entry = timelocks.get_mut(&proposal_id).ok_or("Proposal not in timelock")?;

        if entry.cancelled {
            return Err("Proposal has been cancelled".to_string());
        }

        if now_ms < entry.execution_at_ms {
            let remaining = entry.execution_at_ms - now_ms;
            return Err(format!(
                "Timelock not expired: {}ms remaining",
                remaining
            ));
        }

        // Check grace period
        if now_ms > entry.execution_at_ms + self.grace_period_ms {
            entry.cancelled = true;
            self.cancelled_count += 1;
            return Err("Timelock grace period expired".to_string());
        }

        entry.executed = true;
        entry.tx_hash = Some(format!("0x{:x}", rand::random::<u64>()));
        self.executed_count += 1;

        self.pending_executions.write().await.retain(|&id| id != proposal_id);

        Ok(entry.tx_hash.clone().unwrap())
    }

    /// Cancel a queued proposal
    pub async fn cancel_queued(&mut self, proposal_id: u64, canceller: &str) -> Result<(), String> {
        let mut timelocks = self.timelocks.write().await;
        let entry = timelocks.get_mut(&proposal_id).ok_or("Proposal not in timelock")?;

        if entry.executed {
            return Err("Proposal already executed".to_string());
        }

        entry.cancelled = true;
        self.cancelled_count += 1;

        self.pending_executions.write().await.retain(|&id| id != proposal_id);

        tracing::warn!("Timelock proposal {} cancelled by {}", proposal_id, canceller);
        Ok(())
    }

    /// Get remaining timelock time
    pub async fn get_remaining_delay(&self, proposal_id: u64) -> Result<u64, String> {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        let timelocks = self.timelocks.read().await;
        let entry = timelocks.get(&proposal_id).ok_or("Proposal not in timelock")?;

        if entry.executed || entry.cancelled {
            return Ok(0);
        }

        if now_ms >= entry.execution_at_ms {
            Ok(0)
        } else {
            Ok(entry.execution_at_ms - now_ms)
        }
    }

    /// Check if proposal is ready for execution
    pub async fn is_ready_for_execution(&self, proposal_id: u64) -> Result<bool, String> {
        let remaining = self.get_remaining_delay(proposal_id).await?;
        Ok(remaining == 0)
    }

    /// Process pending timelocks
    pub async fn process_pending(&self) -> Vec<u64> {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        let mut ready = Vec::new();
        let timelocks = self.timelocks.read().await;
        
        for (&proposal_id, entry) in timelocks.iter() {
            if !entry.executed && !entry.cancelled && now_ms >= entry.execution_at_ms {
                ready.push(proposal_id);
            }
        }

        ready
    }

    /// Get timelock statistics
    pub async fn get_stats(&self) -> String {
        format!(
            r#"{{"executed":{},"cancelled":{},"pending":{}}}"#,
            self.executed_count,
            self.cancelled_count,
            self.pending_executions.read().await.len()
        )
    }
}

impl Default for TimelockController {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_timelock_queue() {
        let controller = TimelockController::with_defaults();
        let proposal = GovernanceProposal {
            proposal_id: 1,
            proposal_type: ProposalType::ParameterChange,
            title: "Test".to_string(),
            description: "Test".to_string(),
            proposer: "0xproposer".to_string(),
            parameters: HashMap::new(),
            status: ProposalStatus::Timelocked,
            created_at_ms: 0,
            voting_start_ms: 0,
            voting_end_ms: 0,
            timelock_end_ms: Some(0),
            execution_deadline_ms: 0,
            votes_for: 0,
            votes_against: 0,
            votes_abstain: 0,
            total_voting_power: 0,
            quorum_reached: true,
            approval_reached: true,
            executed_at_ms: None,
            transaction_hash: None,
            cancellation_hash: None,
        };
        
        let result = controller.queue_proposal(&proposal, Some(1000)).await;
        assert!(result.is_ok());
    }
}
