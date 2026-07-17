//! Deployment Governance (spec §17).
//!
//! A deployment is permitted only when **all seven** pre-deployment conditions
//! from §17.1 are satisfied. The Zero-Trust condition is a *constitutional hard
//! gate*: if it is unmet the deployment is rejected (never merely flagged),
//! consistent with the Gatekeeper's behaviour in `gatekeeper.rs`.
//!
//! This module is the canonical, dependency-free policy engine for deployment
//! governance. The backend `deployment.rs` and any deployment endpoint should
//! evaluate these conditions through this gate so the policy is enforced in one
//! place rather than re-implemented (and drifted) across services.

use serde::{Deserialize, Serialize};

/// The seven pre-deployment conditions (spec §17.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeploymentCondition {
    /// 1. Engineering Agent (AEA) verification passes.
    EngineeringVerified,
    /// 2. Independent Auditor Agent (IAA) verification passes.
    AuditorVerified,
    /// 3. Zero Trust verification passes (constitutional hard gate).
    ZeroTrustVerified,
    /// 4. Reflection Engine is operational.
    ReflectionEngineOperational,
    /// 5. Gatekeeper validation succeeds.
    GatekeeperValidated,
    /// 6. Compliance Dashboard reports healthy status.
    ComplianceHealthy,
    /// 7. Commander approval is recorded.
    CommanderApproved,
}

impl DeploymentCondition {
    /// All seven conditions in canonical order (§17.1).
    pub fn all() -> [DeploymentCondition; 7] {
        [
            DeploymentCondition::EngineeringVerified,
            DeploymentCondition::AuditorVerified,
            DeploymentCondition::ZeroTrustVerified,
            DeploymentCondition::ReflectionEngineOperational,
            DeploymentCondition::GatekeeperValidated,
            DeploymentCondition::ComplianceHealthy,
            DeploymentCondition::CommanderApproved,
        ]
    }

    /// Human-readable label used in readiness reports and audit records.
    pub fn label(&self) -> &'static str {
        match self {
            DeploymentCondition::EngineeringVerified => "Engineering Agent verification",
            DeploymentCondition::AuditorVerified => "Independent Auditor verification",
            DeploymentCondition::ZeroTrustVerified => "Zero Trust verification",
            DeploymentCondition::ReflectionEngineOperational => "Reflection Engine operational",
            DeploymentCondition::GatekeeperValidated => "Gatekeeper validation",
            DeploymentCondition::ComplianceHealthy => "Compliance Dashboard healthy",
            DeploymentCondition::CommanderApproved => "Commander approval recorded",
        }
    }

    /// Whether failing this condition is a constitutional (Zero-Trust) hard gate
    /// that must *reject* rather than merely block a deployment.
    pub fn is_zero_trust(&self) -> bool {
        matches!(self, DeploymentCondition::ZeroTrustVerified)
    }
}

/// Snapshot of the seven deployment conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DeploymentState {
    pub engineering_verified: bool,
    pub auditor_verified: bool,
    pub zero_trust_verified: bool,
    pub reflection_engine_operational: bool,
    pub gatekeeper_validated: bool,
    pub compliance_healthy: bool,
    pub commander_approved: bool,
}

impl DeploymentState {
    /// All conditions satisfied.
    pub fn all_satisfied(&self) -> bool {
        self.engineering_verified
            && self.auditor_verified
            && self.zero_trust_verified
            && self.reflection_engine_operational
            && self.gatekeeper_validated
            && self.compliance_healthy
            && self.commander_approved
    }

    /// The condition corresponding to each boolean field.
    pub fn condition_for(&self, cond: DeploymentCondition) -> bool {
        match cond {
            DeploymentCondition::EngineeringVerified => self.engineering_verified,
            DeploymentCondition::AuditorVerified => self.auditor_verified,
            DeploymentCondition::ZeroTrustVerified => self.zero_trust_verified,
            DeploymentCondition::ReflectionEngineOperational => self.reflection_engine_operational,
            DeploymentCondition::GatekeeperValidated => self.gatekeeper_validated,
            DeploymentCondition::ComplianceHealthy => self.compliance_healthy,
            DeploymentCondition::CommanderApproved => self.commander_approved,
        }
    }
}

/// Outcome of evaluating the deployment gate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeploymentReadiness {
    /// Whether deployment is permitted.
    pub ready: bool,
    /// Whether the Zero-Trust constitutional gate passed.
    pub zero_trust_passed: bool,
    /// Conditions that failed evaluation (empty when `ready`).
    pub failed_conditions: Vec<DeploymentCondition>,
}

/// Evaluates the seven §17.1 deployment conditions.
pub struct DeploymentGate;

impl DeploymentGate {
    /// Evaluate a [`DeploymentState`] against the seven conditions.
    ///
    /// Returns `ready: true` only when every condition passes. The Zero-Trust
    /// condition is treated as a constitutional hard gate: when unmet the
    /// deployment is rejected regardless of the other conditions.
    pub fn evaluate(state: &DeploymentState) -> DeploymentReadiness {
        let mut failed = Vec::new();
        for cond in DeploymentCondition::all() {
            if !state.condition_for(cond) {
                failed.push(cond);
            }
        }

        let zero_trust_passed = state.zero_trust_verified;
        let ready = failed.is_empty();

        DeploymentReadiness {
            ready,
            zero_trust_passed,
            failed_conditions: failed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ready_state() -> DeploymentState {
        DeploymentState {
            engineering_verified: true,
            auditor_verified: true,
            zero_trust_verified: true,
            reflection_engine_operational: true,
            gatekeeper_validated: true,
            compliance_healthy: true,
            commander_approved: true,
        }
    }

    #[test]
    fn seven_conditions_defined() {
        assert_eq!(DeploymentCondition::all().len(), 7);
        assert!(DeploymentCondition::ZeroTrustVerified.is_zero_trust());
        assert!(!DeploymentCondition::EngineeringVerified.is_zero_trust());
    }

    #[test]
    fn all_conditions_pass_is_ready() {
        let r = DeploymentGate::evaluate(&ready_state());
        assert!(r.ready);
        assert!(r.zero_trust_passed);
        assert!(r.failed_conditions.is_empty());
    }

    #[test]
    fn zero_trust_failure_is_hard_reject() {
        let mut s = ready_state();
        s.zero_trust_verified = false;
        let r = DeploymentGate::evaluate(&s);
        assert!(!r.ready);
        assert!(!r.zero_trust_passed);
        assert!(r.failed_conditions.contains(&DeploymentCondition::ZeroTrustVerified));
    }

    #[test]
    fn missing_commander_approval_blocks() {
        let mut s = ready_state();
        s.commander_approved = false;
        let r = DeploymentGate::evaluate(&s);
        assert!(!r.ready);
        assert!(r.zero_trust_passed); // Zero Trust still passed
        assert!(r.failed_conditions.contains(&DeploymentCondition::CommanderApproved));
    }

    #[test]
    fn missing_engineering_verification_blocks() {
        let mut s = ready_state();
        s.engineering_verified = false;
        let r = DeploymentGate::evaluate(&s);
        assert!(!r.ready);
        assert!(r.failed_conditions.contains(&DeploymentCondition::EngineeringVerified));
    }
}
