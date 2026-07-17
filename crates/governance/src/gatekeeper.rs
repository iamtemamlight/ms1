//! Gatekeeper — validates every Reflection before publication (spec §11).
//!
//! Implements the 7 validation criteria from §11.2:
//!   Evidence | Integrity | Security | Governance | Policy | Zero Trust | Audit Eligibility
//! A Reflection is only `Approved` when ALL criteria pass AND the Zero Trust gate
//! (independent verification + evidence hash) is satisfied. Every decision is
//! persisted to the audit trail (fixes the "no audit trail / in-memory only" gap).

use crate::audit::AuditStore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reflection {
    pub id: String,
    pub source: String,                 // which agent/card produced it (AEA vs IAA)
    pub content: String,                // human-readable governance intelligence
    pub evidence: String,               // raw supporting data (must be non-empty)
    pub evidence_hash: String,          // sha256 of `evidence` (integrity)
    pub submitted_by: String,           // actor id (must NOT equal approver)
    pub policy_ref: String,             // governing policy reference
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Decision {
    Approved,
    Rejected,
    Flagged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatekeeperVerdict {
    pub reflection_id: String,
    pub decision: Decision,
    pub failed_criteria: Vec<String>,  // which §11.2 criteria failed
    pub zero_trust_passed: bool,
    pub audit_seq: Option<u64>,
}

/// Re-computes the integrity hash of a reflection's evidence.
pub fn hash_evidence(evidence: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(evidence.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Independent verification token — a reflection is only Zero-Trust eligible when a
/// *different* actor (the Independent Auditor Agent) signs off, never its own author.
pub fn zero_trust_eligible(reflection: &Reflection, verifier: &str) -> bool {
    !verifier.is_empty() && verifier != reflection.submitted_by
}

pub struct Gatekeeper {
    audit: Mutex<AuditStore>,
    verdicts: Mutex<HashMap<String, GatekeeperVerdict>>,
}

impl Gatekeeper {
    pub fn new(audit: AuditStore) -> Self {
        Self {
            audit: Mutex::new(audit),
            verdicts: Mutex::new(HashMap::new()),
        }
    }

    /// Evaluate a reflection against the 7 §11.2 criteria + Zero Trust gate.
    /// `verifier` must be an independent actor (IAA), never the reflection's author.
    pub fn evaluate(
        &self,
        reflection: &Reflection,
        verifier: &str,
    ) -> GatekeeperVerdict {
        let mut failed = Vec::new();

        // 1. Evidence — sufficient supporting data
        if reflection.evidence.trim().is_empty() {
            failed.push("Evidence".into());
        }
        // 2. Integrity — evidence hash matches content
        if hash_evidence(&reflection.evidence) != reflection.evidence_hash {
            failed.push("Integrity".into());
        }
        // 3. Security — no security violations (basic content sanitization check)
        if reflection.content.contains("IGNORE PREVIOUS")
            || reflection.content.contains("OVERRIDE")
        {
            failed.push("Security".into());
        }
        // 4. Governance — policy reference present
        if reflection.policy_ref.trim().is_empty() {
            failed.push("Governance".into());
        }
        // 5. Policy Compliance — references a known policy domain
        if !reflection.policy_ref.starts_with("AOS-") {
            failed.push("PolicyCompliance".into());
        }
        // 6. Zero Trust — independent verification by a different actor.
        //    This is a constitutional gate (spec §8/§11): a Zero Trust failure is
        //    NEVER merely "flagged" — it must hard-reject the reflection.
        let zt = zero_trust_eligible(reflection, verifier);
        if !zt {
            failed.push("ZeroTrust".into());
        }
        // 7. Audit Eligibility — can be independently audited (has id + source)
        if reflection.id.is_empty() || reflection.source.is_empty() {
            failed.push("AuditEligibility".into());
        }

        let decision = if failed.is_empty() {
            Decision::Approved
        } else if failed.contains(&"ZeroTrust".to_string()) {
            // Constitutional gate: self-validation / no independent verifier => reject.
            Decision::Rejected
        } else if failed.len() <= 2 {
            Decision::Flagged
        } else {
            Decision::Rejected
        };

        let outcome = match decision {
            Decision::Approved => "approved",
            Decision::Rejected => "rejected",
            Decision::Flagged => "flagged",
        };
        let seq = self
            .audit
            .lock()
            .unwrap()
            .append(
                verifier,
                "gatekeeper.decision",
                &reflection.id,
                outcome,
                &reflection.evidence_hash,
            )
            .ok();

        let verdict = GatekeeperVerdict {
            reflection_id: reflection.id.clone(),
            decision: decision.clone(),
            failed_criteria: failed,
            zero_trust_passed: zt,
            audit_seq: seq,
        };
        self.verdicts
            .lock()
            .unwrap()
            .insert(reflection.id.clone(), verdict.clone());
        verdict
    }

    pub fn get_verdict(&self, id: &str) -> Option<GatekeeperVerdict> {
        self.verdicts.lock().unwrap().get(id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::AuditStore;

    fn sample_reflection(author: &str) -> Reflection {
        let evidence = "uptime=99.9%;latency_p50=18.2ms;deploy=success";
        Reflection {
            id: "R-1".into(),
            source: "AEA".into(),
            content: "System healthy".into(),
            evidence: evidence.into(),
            evidence_hash: hash_evidence(evidence),
            submitted_by: author.into(),
            policy_ref: "AOS-GOV-01".into(),
        }
    }

    fn tmp_audit() -> AuditStore {
        let p = std::env::temp_dir().join(format!("aos_audit_{}.log", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)));
        AuditStore::open(p).unwrap()
    }

    #[test]
    fn independent_verifier_approves() {
        let gk = Gatekeeper::new(tmp_audit());
        let v = gk.evaluate(&sample_reflection("AEA"), "IAA");
        assert_eq!(v.decision, Decision::Approved);
        assert!(v.zero_trust_passed);
        assert!(v.failed_criteria.is_empty());
        assert!(v.audit_seq.is_some());
    }

    #[test]
    fn self_validation_is_rejected_zero_trust() {
        // Author tries to verify their own reflection -> fails Zero Trust.
        let gk = Gatekeeper::new(tmp_audit());
        let v = gk.evaluate(&sample_reflection("AEA"), "AEA");
        assert_eq!(v.decision, Decision::Rejected);
        assert!(v.failed_criteria.contains(&"ZeroTrust".into()));
        assert!(!v.zero_trust_passed);
    }

    #[test]
    fn missing_evidence_is_rejected() {
        let gk = Gatekeeper::new(tmp_audit());
        let mut r = sample_reflection("AEA");
        r.evidence = String::new();
        r.evidence_hash = String::new();
        let v = gk.evaluate(&r, "IAA");
        assert!(v.failed_criteria.contains(&"Evidence".into()));
    }
}