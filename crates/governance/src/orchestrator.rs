//! Governance Orchestrator (spec §5).
//!
//! The central coordination layer described in §5.2. It sequences the dual-agent
//! workflow and guarantees the handoffs the spec requires:
//!
//! 1. **Engineering Agent (AEA)** produces an observation.
//! 2. **Independent Auditor Agent (IAA)** independently verifies it
//!    (constitutional independence guard from [`crate::agents`]).
//! 3. **Gatekeeper** evaluates the reflection against the 7 §11.2 criteria +
//!    Zero-Trust gate.
//! 4. **Reflection Engine** publishes the card *only* when `Approved`.
//! 5. **Commander** receives the verified reflection (via [`OrchestrationResult`]).
//!
//! Every step is persisted to the append-only audit trail (§8.3).

use crate::agents::{AgentId, EngineeringAgent, IndependenceViolation, IndependentAuditorAgent};
use crate::audit::AuditStore;
use crate::gatekeeper::{Decision, Gatekeeper};
use crate::reflection::ReflectionEngine;

/// Outcome of running one observation through the full governance pipeline.
#[derive(Debug, Clone)]
pub struct OrchestrationResult {
    /// Final Gatekeeper decision for the reflection.
    pub decision: Decision,
    /// Whether the Zero-Trust independent-verification gate passed.
    pub zero_trust_passed: bool,
    /// Set when the IAA independence check failed before publication.
    pub independence_violation: Option<IndependenceViolation>,
    /// Audit-trail sequence number of the Gatekeeper decision.
    pub audit_seq: Option<u64>,
}

/// Coordinates the AEA, IAA, Gatekeeper and Reflection Engine.
pub struct GovernanceOrchestrator {
    engine: ReflectionEngine,
}

impl GovernanceOrchestrator {
    /// Construct an orchestrator backed by the shared audit trail.
    pub fn new(audit: AuditStore) -> Self {
        let engine = ReflectionEngine::new(Gatekeeper::new(audit));
        Self { engine }
    }

    /// Run the full governance pipeline for one AEA-produced observation.
    ///
    /// The card is published to its Reflection Card only when the Gatekeeper
    /// returns `Approved`. If the IAA independence check fails, the pipeline
    /// short-circuits with a rejection (the reflection can never self-validate).
    pub fn process(
        &self,
        card_id: &str,
        content: &str,
        evidence: &str,
        policy_ref: &str,
    ) -> OrchestrationResult {
        // Step 1 — AEA produces the observation.
        let obs = EngineeringAgent::observe(card_id, content, evidence, policy_ref);

        // Step 2 — IAA independently verifies (constitutional guard).
        let verification = IndependentAuditorAgent::verify(&obs.submitted_by);

        // Steps 3-4 — Gatekeeper evaluates; Engine publishes only if Approved.
        let (verdict, violation) = match verification {
            Ok(()) => {
                let v = self.engine.submit_observation(obs, AgentId::Iaa.as_str());
                (v, None)
            }
            Err(violation) => {
                // Independence violated before the Gatekeeper could be satisfied.
                // Submit with the original author as verifier so the constitutional
                // Zero-Trust rejection is recorded in the audit trail.
                let v = self
                    .engine
                    .submit_observation(obs, &AgentId::Aea.as_str());
                (v, Some(violation))
            }
        };

        OrchestrationResult {
            decision: verdict.decision.clone(),
            zero_trust_passed: verdict.zero_trust_passed,
            independence_violation: violation,
            audit_seq: verdict.audit_seq,
        }
    }

    /// Current state of all five Reflection Cards (post Gatekeeper).
    pub fn cards(&self) -> Vec<crate::reflection::ReflectionCard> {
        self.engine.get_all_cards()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::AuditStore;

    fn orchestrator() -> GovernanceOrchestrator {
        let p = std::env::temp_dir().join(format!(
            "aos_orch_{}.log",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        GovernanceOrchestrator::new(AuditStore::open(p).unwrap())
    }

    #[test]
    fn full_pipeline_approves_and_publishes() {
        let o = orchestrator();
        let r = o.process(
            "allbright",
            "System healthy",
            "uptime=99.9;latency_p50_ms=18.2",
            "AOS-GOV-01",
        );
        assert_eq!(r.decision, Decision::Approved);
        assert!(r.zero_trust_passed);
        assert!(r.independence_violation.is_none());
        assert_eq!(o.cards().iter().find(|c| c.id == "allbright").unwrap().status,
                   crate::reflection::CardStatus::Operational);
    }

    #[test]
    fn pipeline_does_not_publish_on_rejected_evidence() {
        let o = orchestrator();
        let r = o.process("copilot", "ok", "", "AOS-GOV-02");
        assert_ne!(r.decision, Decision::Approved);
        assert_eq!(
            o.cards().iter().find(|c| c.id == "copilot").unwrap().status,
            crate::reflection::CardStatus::PendingVerification
        );
    }
}
