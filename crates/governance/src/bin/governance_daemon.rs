//! AllBright AgentOS — Governance daemon (Phase B).
//!
//! Runs the Gatekeeper + ReflectionEngine, processes a set of *verified* observations
//! (each independently verified by the Independent Auditor Agent "IAA"), and writes the
//! resulting published 5 Reflection Cards to `governance_cards.json`. This file is the
//! single source of truth the dashboard reads. Every decision is persisted to the
//! audit trail, so the published state is fully auditable and survives restart.

use governance::audit::AuditStore;
use governance::orchestrator::GovernanceOrchestrator;
use governance::reflection::Observation;
use std::path::PathBuf;

/// Sample observations submitted by the Engineering Agent (AEA). Each is verified by
/// the Independent Auditor Agent (IAA) before publication — enforcing spec §9.3.
fn sample_observations() -> Vec<Observation> {
    vec![
        Observation {
            card_id: "allbright".into(),
            submitted_by: "AEA".into(),
            content: "System healthy: deploy succeeded, latency nominal".into(),
            evidence: "uptime=99.9;latency_p50_ms=18.2;deploy=success".into(),
            policy_ref: "AOS-GOV-01".into(),
        },
        Observation {
            card_id: "copilot".into(),
            submitted_by: "AEA".into(),
            content: "Copilot suggestions accepted within threshold".into(),
            evidence: "suggestions=1200;accept_rate=0.82;avg_latency_ms=250".into(),
            policy_ref: "AOS-GOV-02".into(),
        },
        Observation {
            card_id: "intelligence".into(),
            submitted_by: "AEA".into(),
            content: "External market signals nominal".into(),
            evidence: "model_accuracy=0.92;inference_count=5400;threat_level=low".into(),
            policy_ref: "AOS-GOV-03".into(),
        },
        Observation {
            card_id: "commander".into(),
            submitted_by: "AEA".into(),
            content: "No open overrides; last approval recorded".into(),
            evidence: "alerts=3;dashboard_load_s=1.2;approvals_recorded=1".into(),
            policy_ref: "AOS-GOV-04".into(),
        },
        Observation {
            card_id: "zerotrust".into(),
            submitted_by: "AEA".into(),
            content: "Zero Trust verification coverage complete".into(),
            evidence: "trust_score=0.97;verification_status=pass;evidence_coverage=1.0".into(),
            policy_ref: "AOS-GOV-05".into(),
        },
    ]
}

fn main() {
    let audit_path = PathBuf::from("governance_audit.log");
    let cards_path = PathBuf::from("governance_cards.json");

    let audit = match AuditStore::open(&audit_path) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Failed to open audit store: {}", e);
            std::process::exit(1);
        }
    };
    let orchestrator = GovernanceOrchestrator::new(audit);

    let mut approved = 0usize;
    let mut rejected = 0usize;
    for obs in sample_observations() {
        // Full dual-agent pipeline: AEA produces the observation, the IAA
        // independently verifies it, the Gatekeeper evaluates, and the
        // Reflection Engine publishes only on approval (spec §5, §9.3).
        let result = orchestrator.process(&obs.card_id, &obs.content, &obs.evidence, &obs.policy_ref);
        match result.decision {
            governance::gatekeeper::Decision::Approved => approved += 1,
            _ => rejected += 1,
        }
    }

    // Persist the published (gated) Reflection Cards for the dashboard to consume.
    let cards = orchestrator.cards();
    let snapshot = serde_json::json!({
        "generated_at": chrono::Utc::now().to_rfc3339(),
        "approved": approved,
        "rejected": rejected,
        "cards": cards,
    });
    if let Err(e) = std::fs::write(&cards_path, serde_json::to_string_pretty(&snapshot).unwrap()) {
        eprintln!("Failed to write {}: {}", cards_path.display(), e);
        std::process::exit(1);
    }

    println!(
        "[governance] published {} cards (approved={}, rejected={}) -> {}",
        cards.len(),
        approved,
        rejected,
        cards_path.display()
    );
}