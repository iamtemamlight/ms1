//! Agent roles for the dual-agent architecture (spec §4 and §6).
//!
//! The AllBright AgentOS operates two *independent* AI agents:
//! - **AEA** (AllBright Engineering Agent): builds, operates, improves.
//! - **IAA** (Independent Auditor Agent): verifies, audits, assures.
//!
//! The constitutional rule (§6, §8) is that an agent MUST NOT validate its own
//! output. [`ensure_independent_verifier`] is the single enforcement point for
//! that rule and is used by the [`crate::orchestrator::GovernanceOrchestrator`].

use crate::reflection::Observation;
use serde::{Deserialize, Serialize};

/// The three governance actor classes referenced throughout the spec.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentRole {
    /// AllBright Engineering Agent (AEA) — builds/operates.
    Engineering,
    /// Independent Auditor Agent (IAA) — verifies/audits.
    Auditor,
    /// Commander — strategic authority (subject to audit).
    Commander,
}

/// Canonical actor identifiers. `as_str()` yields the spec identifiers
/// (`"AEA"`, `"IAA"`, `"Commander"`) used as `submitted_by`/`verifier` in the
/// Reflection + Gatekeeper records.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentId {
    Aea,
    Iaa,
    Commander,
}

impl AgentId {
    pub fn as_str(&self) -> &'static str {
        match self {
            AgentId::Aea => "AEA",
            AgentId::Iaa => "IAA",
            AgentId::Commander => "Commander",
        }
    }

    pub fn role(&self) -> AgentRole {
        match self {
            AgentId::Aea => AgentRole::Engineering,
            AgentId::Iaa => AgentRole::Auditor,
            AgentId::Commander => AgentRole::Commander,
        }
    }
}

impl std::fmt::Display for AgentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Constitutional independence violation: an actor attempted to verify its own work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndependenceViolation {
    pub actor: String,
    pub reason: String,
}

/// Enforce the spec's separation-of-responsibilities rule (§6, §8): a reflection
/// may only be verified by an actor *different* from its author.
///
/// Returns `Ok(())` when `verifier` is a distinct actor from `submitted_by`,
/// otherwise an [`IndependenceViolation`]. This is the same guarantee the
/// Gatekeeper enforces, surfaced earlier so the Orchestrator can short-circuit.
pub fn ensure_independent_verifier(
    submitted_by: &str,
    verifier: &AgentId,
) -> Result<(), IndependenceViolation> {
    if submitted_by == verifier.as_str() {
        return Err(IndependenceViolation {
            actor: verifier.as_str().to_string(),
            reason: "an agent may not verify its own output (Zero Trust; spec §6, §8)".into(),
        });
    }
    Ok(())
}

/// Engineering Agent (AEA) helper: produces a Reflection Engine [`Observation`]
/// authored by the AEA. The AEA never verifies — that is the IAA's role.
pub struct EngineeringAgent;

impl EngineeringAgent {
    pub fn observe(card_id: &str, content: &str, evidence: &str, policy_ref: &str) -> Observation {
        Observation {
            card_id: card_id.to_string(),
            submitted_by: AgentId::Aea.as_str().to_string(),
            content: content.to_string(),
            evidence: evidence.to_string(),
            policy_ref: policy_ref.to_string(),
        }
    }
}

/// Independent Auditor Agent (IAA) helper: verifies an observation on behalf of
/// the IAA, enforcing the independence guard before the Gatekeeper is invoked.
pub struct IndependentAuditorAgent;

impl IndependentAuditorAgent {
    pub fn verify(submitted_by: &str) -> Result<(), IndependenceViolation> {
        ensure_independent_verifier(submitted_by, &AgentId::Iaa)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aea_and_iaa_are_distinct_actors() {
        assert_ne!(AgentId::Aea.as_str(), AgentId::Iaa.as_str());
        assert_eq!(AgentId::Aea.role(), AgentRole::Engineering);
        assert_eq!(AgentId::Iaa.role(), AgentRole::Auditor);
    }

    #[test]
    fn iaa_may_verify_aea_work() {
        assert!(IndependentAuditorAgent::verify(AgentId::Aea.as_str()).is_ok());
    }

    #[test]
    fn self_verification_is_rejected() {
        let v = ensure_independent_verifier(AgentId::Aea.as_str(), &AgentId::Aea);
        assert!(v.is_err());
        let v = IndependentAuditorAgent::verify(AgentId::Iaa.as_str());
        assert!(v.is_err());
    }

    #[test]
    fn aea_observation_is_authored_by_aea() {
        let obs = EngineeringAgent::observe("allbright", "healthy", "uptime=99", "AOS-GOV-01");
        assert_eq!(obs.submitted_by, "AEA");
    }
}
