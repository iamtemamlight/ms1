// ==============================================================================
// CONSTITUTIONAL GOVERNANCE MODULE (CGM) — Constitution Guard
// Purpose: Enforce AIGUIDE constitutional laws as pre-action gates
// ==============================================================================

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::relationship_matrix::{RelationshipMatrix, Subsystem};
use crate::shield_guardrails::{EthicsEngine, TradeAuthorization};

/// Constitutional verdict for an action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceVerdict {
    pub allowed: bool,
    pub violations: Vec<GovernanceViolation>,
    pub cross_subsystem_impact: HashMap<Subsystem, f64>,
    pub risk_level: RiskLevel,
}

/// Individual constitutional violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceViolation {
    pub law_id: u8,
    pub law: String,
    pub description: String,
    pub severity: RiskLevel,
}

/// Risk level matching shield_guardrails
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
    Halted,
}

/// Constitution Guard — validates actions against AIGUIDE laws
#[derive(Clone)]
pub struct ConstitutionGuard {
    pub relationship_matrix: std::sync::Arc<tokio::sync::Mutex<RelationshipMatrix>>,
}

impl ConstitutionGuard {
    pub fn new(matrix: std::sync::Arc<tokio::sync::Mutex<RelationshipMatrix>>) -> Self {
        Self { relationship_matrix: matrix }
    }

    /// Evaluate an action against all constitutional laws
    pub async fn evaluate(&self, action: &SystemAction) -> GovernanceVerdict {
        let mut violations = Vec::new();

        // Law 1: Profit Growth is the only explicit user-defined objective
        if action.objective != Some("profit_growth") {
            violations.push(GovernanceViolation {
                law_id: 1,
                law: "Profit Growth is the only explicit user-defined objective".to_string(),
                description: format!("Action attempts to set objective: {:?}", action.objective),
                severity: RiskLevel::Critical,
            });
        }

        // Law 3: Subsystems must never be optimized independently
        if action.affected_subsystems.len() == 1 {
            violations.push(GovernanceViolation {
                law_id: 3,
                law: "Subsystems must never be optimized independently".to_string(),
                description: format!("Action targets only subsystem: {:?}", action.affected_subsystems),
                severity: RiskLevel::High,
            });
        }

        // Law 7: All optimization must consider enterprise-wide impact
        let matrix = self.relationship_matrix.lock().await;
        let impact = matrix.evaluate_impact(&action.expected_changes);
        drop(matrix);

        // Check if any subsystem would be negatively impacted beyond threshold
        for (subsystem, delta) in &impact {
            if *delta < -0.3 {
                violations.push(GovernanceViolation {
                    law_id: 7,
                    law: "All optimization must consider enterprise-wide impact".to_string(),
                    description: format!("Subsystem {:?} would be negatively impacted by {:.2}", subsystem, delta),
                    severity: RiskLevel::High,
                });
            }
        }

        let allowed = violations.is_empty() || violations.iter().all(|v| v.severity <= RiskLevel::Medium);
        let risk_level = violations.iter().map(|v| v.severity).min().unwrap_or(RiskLevel::Low);

        GovernanceVerdict {
            allowed,
            violations,
            cross_subsystem_impact: impact,
            risk_level,
        }
    }

    /// Validate trade against constitutional guardrails
    pub async fn validate_trade(&self, position_size_eth: f64, expected_profit_eth: f64, expected_loss_eth: f64, ethics: &mut EthicsEngine) -> TradeAuthorization {
        // Law 8: All agents must be orchestrated by the Copilot
        // Law 9: Every action must be logged and auditable
        // These are enforced by the Copilot loop itself
        
        // Delegate to EthicsEngine for P&L guardrails
        let mut auth = ethics.authorize_trade(position_size_eth, expected_profit_eth, expected_loss_eth);
        
        // Additional constitutional checks
        if !auth.approved {
            return auth;
        }

        // Check cross-subsystem impact
        let matrix = self.relationship_matrix.lock().await;
        let impact = matrix.evaluate_impact(&[
            (Subsystem::Profit, expected_profit_eth / 100.0),
            (Subsystem::Security, -expected_loss_eth / 100.0),
        ]);
        drop(matrix);

        // If Security subsystem impact is too negative, block trade
        if let Some(security_impact) = impact.get(&Subsystem::Security) {
            if *security_impact < -0.4 {
                auth.approved = false;
                auth.risk_level = crate::shield_guardrails::RiskLevel::Critical;
                auth.reason = format!("CONSTITUTIONAL_VIOLATION: Security subsystem impact {:.2} exceeds threshold", security_impact);
            }
        }

        auth
    }
}

/// Action representation for constitutional validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemAction {
    pub action_type: ActionType,
    pub objective: Option<&'static str>,
    pub affected_subsystems: Vec<Subsystem>,
    pub expected_changes: Vec<(Subsystem, f64)>,
    pub initiated_by: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionType {
    Optimization,
    Trade,
    ConfigurationChange,
    AgentDeployment,
    ParameterUpdate,
}
