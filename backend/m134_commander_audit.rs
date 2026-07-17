//! ==============================================================================
//! ALLBRIGHT COMMANDER AUDIT & LEARNING LAYER (M134)
//! ==============================================================================
//! Layer 3 of the Three-Layer Autonomous Audit Framework.
//!
//! It answers the question:
//!   "Is the Commander becoming a better guardian of the autonomous system?"
//!
//! STRICT SEPARATION OF POWERS (see brief §3):
//!   * Layer 3 audits and improves the COMMANDER only.
//!   * It does NOT audit the Copilot — that is DACAM's (M132) exclusive role.
//!     (DACAM restrictions: "must never Audit Commander performance".)
//!   * It does NOT perform macro enterprise governance — that is the Sovereign
//!     Audit Engine's (M133) exclusive role.
//!   * It receives verified outcomes from both layers and adds a *learning* loop
//!     that educates the Commander (explanations, historical comparisons,
//!     scenario simulations, lessons, recommendations).
//!
//! Every Commander intervention is cryptographically recorded and *reviewed by*
//! the Commander Audit layer (brief §6): logged, time-stamped, and scored.

use std::sync::Arc;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use k256::ecdsa::{SigningKey, Signature, RecoveryId};
use sha2::{Sha256, Digest};
use uuid::Uuid;

/// Deterministic internal governance signing key (root-of-trust stand-in) for
/// the Commander Audit layer. Distinct seed from DACAM/Sovereign to keep
/// each audit authority independently verifiable.
const COMMANDER_AUDIT_SEED: [u8; 32] = [9u8; 32];

pub const COMMANDER_AUDIT_SOURCE: &str = "COMMANDER_AUDIT_LEARNING_LAYER";
pub const COMMANDER_AUDIT_SCOPE: &str = "COMMANDER_GOVERNANCE_IMPROVEMENT";

// ------------------------------------------------------------------------------
// Shared status (consistent across all three layers for UI rendering)
// ------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditStatus {
    Green,
    Amber,
    Red,
}

impl AuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Green => "GREEN",
            Self::Amber => "AMBER",
            Self::Red => "RED",
        }
    }
}

impl Default for AuditStatus {
    fn default() -> Self {
        Self::Green
    }
}

// ------------------------------------------------------------------------------
// Inputs & metrics
// ------------------------------------------------------------------------------

/// One observed Commander intervention, fed from the Sovereign intervention pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterventionObservation {
    pub action: String,
    pub commander_id: String,
    pub aligned_with_policy: bool,
    pub response_time_ms: f64,
    pub outcome_positive: bool,
    pub sovereign_status: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InterventionStats {
    pub total: u64,
    pub approvals: u64,
    pub rejections: u64,
    pub profile_switches: u64,
    pub conservative: u64,
    pub pauses: u64,
    pub resumes: u64,
    pub emergencies: u64,
    pub aligned: u64,
    pub scored: u64,
    pub sum_response_ms: f64,
}

/// A learning module the Commander Audit layer uses to educate the Commander.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningModule {
    pub id: String,
    pub title: String,
    pub description: String,
    pub completed: bool,
}

/// The Commander Reflection (Card 3) payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommanderReflection {
    pub status: AuditStatus,
    pub governance_score: f64,
    pub decision_quality: f64,
    pub intervention_efficiency: f64,
    pub policy_alignment: f64,
    pub learning_progress: f64,
    pub strength: String,
    pub improvement: String,
    pub recommendation: String,
    pub intervention_stats: InterventionStats,
    pub learning_modules: Vec<LearningModule>,
    pub generated_at: String,
}

/// Immutable, cryptographically signed Commander Audit record (brief §6, §7).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommanderAuditRecord {
    pub audit_id: String,
    pub created_at: String,
    pub audit_source: String,
    pub audit_scope: String,
    pub status: AuditStatus,
    pub finding: String,
    pub recommended_action: String,
    pub executed_response: String,
    pub authorization_trail: Vec<String>,
    pub payload_hash: String,
    pub signature: String,
    pub reflection: CommanderReflection,
}

// ------------------------------------------------------------------------------
// Main Commander Audit & Learning Layer
// ------------------------------------------------------------------------------

pub struct CommanderAuditor {
    pub enabled: bool,
    pub records: DashMap<String, CommanderAuditRecord>,
    pub governance_key: Arc<SigningKey>,
    pub stats: InterventionStats,
    pub last_recommendation: String,
    pub last_sovereign_status: String,
    pub learning_modules: Vec<LearningModule>,
    pub recommendations_log: Vec<String>,
    pub last_audit_block: u64,
}

/// Internal scoring snapshot used by the learning loop.
struct Scores {
    governance_score: f64,
    decision_quality: f64,
    intervention_efficiency: f64,
    policy_alignment: f64,
    learning_progress: f64,
}

impl CommanderAuditor {
    pub fn new() -> Self {
        let governance_key = Arc::new(
            SigningKey::from_slice(&COMMANDER_AUDIT_SEED)
                .expect("Commander audit seed must be a valid 32-byte scalar"),
        );
        Self {
            enabled: true,
            records: DashMap::new(),
            governance_key,
            stats: InterventionStats::default(),
            last_recommendation: String::new(),
            last_sovereign_status: "GREEN".into(),
            learning_modules: vec![
                LearningModule {
                    id: "scenario-strategic".into(),
                    title: "Strategic Scenario Simulation".to_string(),
                    description: "Run what-if governance simulations against historical interventions.".to_string(),
                    completed: false,
                },
                LearningModule {
                    id: "liquidity-concentration".into(),
                    title: "Liquidity Concentration Recognition".to_string(),
                    description: "Earlier identification of liquidity concentration risk before Sovereign escalation.".to_string(),
                    completed: false,
                },
                LearningModule {
                    id: "risk-response-timing".into(),
                    title: "Risk Response Timing".to_string(),
                    description: "Tighten decision latency during RED sovereign posture.".to_string(),
                    completed: false,
                },
                LearningModule {
                    id: "policy-alignment".into(),
                    title: "Policy Alignment Drills".to_string(),
                    description: "Align Commander overrides with Sovereign recommended boundaries.".to_string(),
                    completed: false,
                },
                LearningModule {
                    id: "intervention-discipline".into(),
                    title: "Intervention Discipline".to_string(),
                    description: "Avoid over-intervention during GREEN autonomous operation.".to_string(),
                    completed: false,
                },
            ],
            recommendations_log: Vec::new(),
            last_audit_block: 0,
        }
    }

    pub fn set_enabled(&mut self, e: bool) { self.enabled = e; }
    pub fn is_enabled(&self) -> bool { self.enabled }

    /// Compare a Commander action against the current Sovereign posture to decide
    /// whether the intervention was *aligned* with governance policy.
    pub fn action_alignment(action: &str, target_profile: Option<&str>, sovereign_status: &str) -> bool {
        match sovereign_status {
            "GREEN" => matches!(
                action,
                "APPROVE_RECOMMENDED" | "RESUME_AUTONOMOUS_EXECUTION" | "REJECT_RECOMMENDATION"
            ),
            "AMBER" => matches!(action, "ACTIVATE_CONSERVATIVE_MODE" | "PAUSE_AUTONOMOUS_EXECUTION")
                || (action == "SWITCH_OPERATING_PROFILE" && target_profile == Some("CONSERVATIVE")),
            "RED" => matches!(
                action,
                "EMERGENCY_SHUTDOWN" | "PAUSE_AUTONOMOUS_EXECUTION" | "ACTIVATE_CONSERVATIVE_MODE"
            ) || (action == "SWITCH_OPERATING_PROFILE" && target_profile == Some("LOCKDOWN")),
            _ => true,
        }
    }

    /// Feed one Commander intervention into the audit + learning loop.
    /// Produces an immutable, signed, Commander-audited record.
    pub fn record_intervention(
        &mut self,
        obs: InterventionObservation,
    ) -> CommanderAuditRecord {
        // Update running statistics.
        self.stats.total += 1;
        match obs.action.as_str() {
            "APPROVE_RECOMMENDED" => self.stats.approvals += 1,
            "REJECT_RECOMMENDATION" => self.stats.rejections += 1,
            "SWITCH_OPERATING_PROFILE" => self.stats.profile_switches += 1,
            "ACTIVATE_CONSERVATIVE_MODE" => self.stats.conservative += 1,
            "PAUSE_AUTONOMOUS_EXECUTION" => self.stats.pauses += 1,
            "RESUME_AUTONOMOUS_EXECUTION" => self.stats.resumes += 1,
            "EMERGENCY_SHUTDOWN" => self.stats.emergencies += 1,
            _ => {}
        }
        if obs.aligned_with_policy {
            self.stats.aligned += 1;
        }
        self.stats.scored += 1;
        self.stats.sum_response_ms += obs.response_time_ms;

        // Keep the latest Sovereign posture for alignment context.
        self.last_sovereign_status = obs.sovereign_status.clone();

        let scores = self.compute_scores();
        let reflection = self.build_reflection(&scores);

        let finding = format!(
            "Commander {} — action {} | aligned={} | response {}ms | sovereign={}",
            obs.commander_id,
            obs.action,
            obs.aligned_with_policy,
            obs.response_time_ms as u64,
            obs.sovereign_status,
        );
        let recommended_action = format!(
            "Commander Audit review: governance match {:.0}% — {}",
            scores.governance_score,
            reflection.recommendation,
        );

        self.create_record(
            reflection,
            finding,
            recommended_action,
            format!("Reviewed & recorded by Commander Audit Layer: {}", obs.action),
            vec![
                format!("COMMANDER:{}", obs.commander_id),
                format!("ACTION:{}", obs.action),
                format!("ALIGNED:{}", obs.aligned_with_policy),
                format!("SOVEREIGN:{}", obs.sovereign_status),
                "REVIEWED_BY:COMMANDER_AUDIT".into(),
            ],
        )
    }

    fn compute_scores(&self) -> Scores {
        let decision_quality = if self.stats.scored > 0 {
            (self.stats.aligned as f64 / self.stats.scored as f64) * 100.0
        } else {
            100.0
        };

        let avg_response = if self.stats.scored > 0 {
            self.stats.sum_response_ms / self.stats.scored as f64
        } else {
            0.0
        };
        let intervention_efficiency = if avg_response > 0.0 {
            (100.0 - (avg_response / 50.0).min(100.0)).max(0.0)
        } else {
            100.0
        };

        // Policy alignment: reward aligned actions, penalize repeated emergencies.
        let penalty = (self.stats.emergencies as f64 * 5.0).min(40.0);
        let policy_alignment = (decision_quality - penalty).max(0.0);

        let completed = self.learning_modules.iter().filter(|m| m.completed).count() as f64;
        let total = self.learning_modules.len().max(1) as f64;
        let base_progress = (completed / total) * 60.0;
        let earned = (self.stats.aligned as f64).min(40.0);
        let learning_progress = (base_progress + earned).min(100.0);

        let governance_score =
            0.30 * decision_quality
                + 0.25 * intervention_efficiency
                + 0.25 * policy_alignment
                + 0.20 * learning_progress;

        Scores {
            governance_score,
            decision_quality,
            intervention_efficiency,
            policy_alignment,
            learning_progress,
        }
    }

    fn build_reflection(&self, s: &Scores) -> CommanderReflection {
        let status = if s.governance_score >= 85.0 {
            AuditStatus::Green
        } else if s.governance_score >= 65.0 {
            AuditStatus::Amber
        } else {
            AuditStatus::Red
        };

        let strength = if self.stats.sum_response_ms / (self.stats.scored as f64).max(1.0) <= 1200.0 {
            "Excellent risk response timing.".into()
        } else if s.policy_alignment >= 85.0 {
            "Strong policy alignment with Sovereign boundaries.".into()
        } else {
            "Developing judgment — continue aligned governance.".into()
        };

        let improvement = if !self
            .learning_modules
            .iter()
            .any(|m| m.id == "liquidity-concentration" && m.completed)
        {
            "Earlier recognition of liquidity concentration.".into()
        } else {
            "Maintain disciplined intervention cadence during GREEN operation.".into()
        };

        let recommendation = if let Some(m) = self.learning_modules.iter().find(|m| !m.completed) {
            format!("Complete learning module: {}.", m.title)
        } else {
            "Continue aligned governance; Commander mastery sustained.".into()
        };

        CommanderReflection {
            status,
            governance_score: s.governance_score,
            decision_quality: s.decision_quality,
            intervention_efficiency: s.intervention_efficiency,
            policy_alignment: s.policy_alignment,
            learning_progress: s.learning_progress,
            strength,
            improvement,
            recommendation,
            intervention_stats: self.stats.clone(),
            learning_modules: self.learning_modules.clone(),
            generated_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Mark a learning module complete (Commander education progress).
    pub fn complete_module(&mut self, id: &str) {
        if let Some(m) = self.learning_modules.iter_mut().find(|m| m.id == id) {
            m.completed = true;
        }
    }

    /// Produce the Commander Reflection (Card 3) from current state.
    pub fn reflect(&self) -> CommanderReflection {
        let s = self.compute_scores();
        self.build_reflection(&s)
    }

    fn canonical(reflection: &CommanderReflection, finding: &str, recommended: &str) -> String {
        format!(
            "{}|{}|{}|{:.1}|{:.1}|{:.1}|{:.1}|{:.1}|{}",
            COMMANDER_AUDIT_SOURCE,
            COMMANDER_AUDIT_SCOPE,
            reflection.status.as_str(),
            reflection.governance_score,
            reflection.decision_quality,
            reflection.intervention_efficiency,
            reflection.policy_alignment,
            reflection.learning_progress,
            reflection.generated_at,
        )
    }

    fn sign_payload(&self, canonical: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(canonical.as_bytes());
        let digest = hasher.finalize();
        let hash: [u8; 32] = digest.into();
        let (signature, _recid): (Signature, RecoveryId) = self
            .governance_key
            .sign_prehash_recoverable(&hash)
            .expect("commander audit signing");
        hex::encode(signature.to_bytes())
    }

    pub fn create_record(
        &mut self,
        reflection: CommanderReflection,
        finding: String,
        recommended_action: String,
        executed_response: String,
        authorization_trail: Vec<String>,
    ) -> CommanderAuditRecord {
        let block = self.last_audit_block + 1;
        self.last_audit_block = block;

        let canonical = Self::canonical(&reflection, &finding, &recommended_action);
        let mut hasher = Sha256::new();
        hasher.update(canonical.as_bytes());
        let payload_hash = hex::encode(hasher.finalize());

        let signature = self.sign_payload(&canonical);

        let record = CommanderAuditRecord {
            audit_id: format!("CMD-{}-{}", Uuid::new_v4().simple(), block),
            created_at: chrono::Utc::now().to_rfc3339(),
            audit_source: COMMANDER_AUDIT_SOURCE.into(),
            audit_scope: COMMANDER_AUDIT_SCOPE.into(),
            status: reflection.status,
            finding,
            recommended_action,
            executed_response,
            authorization_trail,
            payload_hash,
            signature,
            reflection,
        };

        self.records.insert(record.audit_id.clone(), record.clone());
        record
    }

    /// Independently verify a Commander Audit record's signature.
    pub fn verify_record(&self, record: &CommanderAuditRecord) -> bool {
        let canonical = Self::canonical(
            &record.reflection,
            &record.finding,
            &record.recommended_action,
        );
        let mut hasher = Sha256::new();
        hasher.update(canonical.as_bytes());
        if hex::encode(hasher.finalize()) != record.payload_hash {
            return false;
        }
        self.sign_payload(&canonical) == record.signature
    }

    pub fn get_latest_record(&self) -> Option<CommanderAuditRecord> {
        self.records
            .iter()
            .max_by_key(|entry| entry.value().created_at.clone())
            .map(|entry| entry.value().clone())
    }

    pub fn list_records(&self) -> Vec<CommanderAuditRecord> {
        let mut v: Vec<CommanderAuditRecord> =
            self.records.iter().map(|e| e.value().clone()).collect();
        v.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        v
    }
}

impl Default for CommanderAuditor {
    fn default() -> Self {
        Self::new()
    }
}
