//! Reflection Engine (spec §9).
//!
//! Converts *verified* system observations into governance intelligence. Per §9.3,
//! every reflection must first pass the Gatekeeper before it reaches the Commander
//! (i.e. before it is published to a Reflection Card). This implementation:
//!   - seeds the 5 cards (§10) with NO fake metrics (status "pending-verification");
//!   - only publishes a card update when the Gatekeeper returns `Approved`;
//!   - persists publication to the audit trail.
//! Hardcoded constant metrics from the prior version have been removed (fixes F3).

use crate::gatekeeper::{Decision, Gatekeeper, Reflection as GovReflection, hash_evidence};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

pub const CARD_IDS: [&str; 5] = [
    "allbright",  // Card 1: System
    "copilot",    // Card 2: Copilot
    "intelligence", // Card 3: Intelligence
    "commander",  // Card 4: Commander
    "zerotrust",  // Card 5: Zero Trust
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CardStatus {
    PendingVerification,
    Operational,
    Degraded,
    Critical,
}

impl Default for CardStatus {
    fn default() -> Self {
        CardStatus::PendingVerification
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub name: String,
    pub value: f64,
    pub unit: String,
    /// Where this metric came from (must be a verified signal, never a constant).
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflectionCard {
    pub id: String,
    pub name: String,
    pub status: CardStatus,
    pub last_update: i64,
    pub metrics: Vec<Metric>,
}

/// A raw observation supplied by an agent; must be verified before publication.
#[derive(Debug, Clone)]
pub struct Observation {
    pub card_id: String,
    pub submitted_by: String, // actor id (e.g. "AEA")
    pub content: String,
    pub evidence: String,      // raw signal data
    pub policy_ref: String,
}

pub struct ReflectionEngine {
    gatekeeper: Mutex<Gatekeeper>,
    cards: Mutex<Vec<ReflectionCard>>,
}

impl ReflectionEngine {
    /// `gatekeeper` is constructed with the shared audit store so all decisions persist.
    pub fn new(gatekeeper: Gatekeeper) -> Self {
        let cards = CARD_IDS
            .iter()
            .map(|id| ReflectionCard {
                id: id.to_string(),
                name: card_name(id),
                status: CardStatus::PendingVerification,
                last_update: chrono::Utc::now().timestamp(),
                metrics: Vec::new(),
            })
            .collect();
        Self {
            gatekeeper: Mutex::new(gatekeeper),
            cards: Mutex::new(cards),
        }
    }

    pub fn get_all_cards(&self) -> Vec<ReflectionCard> {
        self.cards.lock().unwrap().clone()
    }

    pub fn get_card(&self, card_id: &str) -> Option<ReflectionCard> {
        self.cards
            .lock()
            .unwrap()
            .iter()
            .find(|c| c.id == card_id)
            .cloned()
    }

    /// Submit a raw observation. It is converted to a `Reflection`, evaluated by the
    /// Gatekeeper (independent IAA verifier), and ONLY published if `Approved`.
    /// Returns the Gatekeeper verdict.
    pub fn submit_observation(
        &self,
        obs: Observation,
        independent_verifier: &str, // must be a different actor (IAA), never obs.submitted_by
    ) -> crate::gatekeeper::GatekeeperVerdict {
        let evidence_hash = hash_evidence(&obs.evidence);
        let gov_reflection = GovReflection {
            id: format!("REF-{}", obs.card_id),
            source: obs.card_id.clone(),
            content: obs.content.clone(),
            evidence: obs.evidence.clone(),
            evidence_hash,
            submitted_by: obs.submitted_by.clone(),
            policy_ref: obs.policy_ref.clone(),
        };

        let verdict = self
            .gatekeeper
            .lock()
            .unwrap()
            .evaluate(&gov_reflection, independent_verifier);

        if verdict.decision == Decision::Approved {
            // Publish: update the card with the verified signal.
            if let Some(card) = self.cards.lock().unwrap().iter_mut().find(|c| c.id == obs.card_id) {
                card.status = CardStatus::Operational;
                card.last_update = chrono::Utc::now().timestamp();
                card.metrics = parse_evidence_metrics(&obs.evidence);
            }
        }
        verdict
    }
}

fn card_name(id: &str) -> String {
    match id {
        "allbright" => "AllBright System",
        "copilot" => "Copilot",
        "intelligence" => "Intelligence",
        "commander" => "Commander",
        "zerotrust" => "Zero Trust",
        _ => id,
    }
    .to_string()
}

/// Parse `key=value;key=value` evidence into metrics with a "verified-signal" source.
fn parse_evidence_metrics(evidence: &str) -> Vec<Metric> {
    evidence
        .split(';')
        .filter_map(|kv| {
            let mut it = kv.splitn(2, '=');
            let key = it.next()?.trim().to_string();
            let val = it.next()?.trim().parse::<f64>().ok()?;
            Some(Metric {
                name: key,
                value: val,
                unit: "raw".into(),
                source: "verified-signal".into(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::AuditStore;

    fn engine() -> ReflectionEngine {
            let audit = AuditStore::open(
            std::env::temp_dir().join(format!("aos_audit_re_{}.log", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0))),
        )
        .unwrap();
        ReflectionEngine::new(Gatekeeper::new(audit))
    }

    #[test]
    fn cards_start_unverified_no_fake_metrics() {
        let e = engine();
        for c in e.get_all_cards() {
            assert_eq!(c.status, CardStatus::PendingVerification);
            assert!(c.metrics.is_empty(), "card {} had fake metrics", c.id);
        }
    }

    #[test]
    fn verified_observation_publishes_card() {
        let e = engine();
        let v = e.submit_observation(
            Observation {
                card_id: "allbright".into(),
                submitted_by: "AEA".into(),
                content: "System healthy".into(),
                evidence: "uptime=99.9;latency=18.2".into(),
                policy_ref: "AOS-GOV-01".into(),
            },
            "IAA",
        );
        assert_eq!(v.decision, Decision::Approved);
        let card = e.get_card("allbright").unwrap();
        assert_eq!(card.status, CardStatus::Operational);
        assert_eq!(card.metrics.len(), 2);
    }

    #[test]
    fn self_validation_blocks_publication() {
        let e = engine();
        let v = e.submit_observation(
            Observation {
                card_id: "copilot".into(),
                submitted_by: "AEA".into(),
                content: "ok".into(),
                evidence: "x=1".into(),
                policy_ref: "AOS-GOV-01".into(),
            },
            "AEA", // same actor -> Zero Trust fails
        );
        assert_eq!(v.decision, Decision::Rejected);
        assert_eq!(e.get_card("copilot").unwrap().status, CardStatus::PendingVerification);
    }
}