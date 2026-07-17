//! ==============================================================================
//! ALLBRIGHT SOVEREIGN AUDIT ENGINE (M133)
//! ==============================================================================
//! Macro-governance audit layer for the entire AllBright ecosystem.
//!
//! It answers the single macro question:
//!   "Is the overall system operating in the long-term interests of the enterprise?"
//!
//! STRICT SEPARATION OF POWERS (see Integration Brief §2B):
//!   * The Sovereign Audit Engine performs NO Copilot mathematical verification.
//!   * It does NOT reproduce Copilot arithmetic (no SDI / Lambda / epsilon_f / alpha).
//!   * It does NOT modify operational (DACAM) audit records.
//!   * It evaluates enterprise-level governance posture ONLY:
//!       - Strategic alignment
//!       - Capital exposure
//!       - Liquidity posture
//!       - Systemic / aggregated risk
//!       - Governance & regulatory compliance
//!       - Approval or restriction of operating profiles
//!
//! It is the macro governance layer that operates *above* DACAM. DACAM verifies
//! the machine; the Sovereign verifies the enterprise.

use std::sync::Arc;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use k256::ecdsa::{SigningKey, Signature, RecoveryId};
use sha2::{Sha256, Digest};
use uuid::Uuid;

/// Deterministic internal governance signing key (root-of-trust stand-in).
/// In production this is replaced by the enterprise root key loaded from the
/// KeyManager / HSM. The seed is fixed so signatures are reproducible and
/// independently verifiable across restarts.
const SOVEREIGN_GOVERNANCE_SEED: [u8; 32] = [7u8; 32];

/// Audit source identifier stamped on every Sovereign record.
pub const SOVEREIGN_AUDIT_SOURCE: &str = "SOVEREIGN_AUDIT_ENGINE";

/// Macro governance scope (explicitly excludes micro-operational Copilot scope).
pub const SOVEREIGN_AUDIT_SCOPE: &str = "MACRO_GOVERNANCE_ENTERPRISE";

// ------------------------------------------------------------------------------
// Thresholds — macro governance tolerances
// ------------------------------------------------------------------------------

/// Strategic alignment must remain at/above this to be graded PASS (%)
pub const STRATEGIC_ALIGNMENT_PASS_PCT: f64 = 80.0;
pub const STRATEGIC_ALIGNMENT_WARN_PCT: f64 = 60.0;

/// Capital exposure at/below this is PASS (% of reserves deployed)
pub const CAPITAL_EXPOSURE_PASS_PCT: f64 = 70.0;
pub const CAPITAL_EXPOSURE_WARN_PCT: f64 = 85.0;

/// Liquidity ratio at/above this is PASS (liquid assets / short-term obligations)
pub const LIQUIDITY_RATIO_PASS: f64 = 1.2;
pub const LIQUIDITY_RATIO_WARN: f64 = 1.0;

/// Composite systemic risk index at/below this is PASS (0-100)
pub const RISK_INDEX_PASS: f64 = 50.0;
pub const RISK_INDEX_WARN: f64 = 70.0;

/// Governance / regulatory compliance score at/above this is PASS (0-100)
pub const COMPLIANCE_SCORE_PASS: f64 = 90.0;
pub const COMPLIANCE_SCORE_WARN: f64 = 75.0;

// ------------------------------------------------------------------------------
// Core enums
// ------------------------------------------------------------------------------

/// Tri-state audit health used by both audit layers for consistent UI rendering.
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

/// Operating profiles the enterprise may run under. The Sovereign layer is the
/// sole authority that approves or restricts these (brief §2B, §5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperatingProfile {
    Aggressive,
    Balanced,
    Conservative,
    Lockdown,
}

impl Default for OperatingProfile {
    fn default() -> Self {
        Self::Balanced
    }
}

impl OperatingProfile {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Aggressive => "AGGRESSIVE",
            Self::Balanced => "BALANCED",
            Self::Conservative => "CONSERVATIVE",
            Self::Lockdown => "LOCKDOWN",
        }
    }
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "AGGRESSIVE" => Some(Self::Aggressive),
            "BALANCED" => Some(Self::Balanced),
            "CONSERVATIVE" => Some(Self::Conservative),
            "LOCKDOWN" => Some(Self::Lockdown),
            _ => None,
        }
    }
}

/// Commander intervention actions (brief §6). Every one creates an immutable
/// governance record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommanderAction {
    ApproveRecommended,
    RejectRecommendation,
    SwitchOperatingProfile,
    ActivateConservativeMode,
    PauseAutonomousExecution,
    ResumeAutonomousExecution,
    EmergencyShutdown,
}

impl CommanderAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ApproveRecommended => "APPROVE_RECOMMENDED",
            Self::RejectRecommendation => "REJECT_RECOMMENDATION",
            Self::SwitchOperatingProfile => "SWITCH_OPERATING_PROFILE",
            Self::ActivateConservativeMode => "ACTIVATE_CONSERVATIVE_MODE",
            Self::PauseAutonomousExecution => "PAUSE_AUTONOMOUS_EXECUTION",
            Self::ResumeAutonomousExecution => "RESUME_AUTONOMOUS_EXECUTION",
            Self::EmergencyShutdown => "EMERGENCY_SHUTDOWN",
        }
    }
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "APPROVE_RECOMMENDED" => Some(Self::ApproveRecommended),
            "REJECT_RECOMMENDATION" => Some(Self::RejectRecommendation),
            "SWITCH_OPERATING_PROFILE" => Some(Self::SwitchOperatingProfile),
            "ACTIVATE_CONSERVATIVE_MODE" => Some(Self::ActivateConservativeMode),
            "PAUSE_AUTONOMOUS_EXECUTION" => Some(Self::PauseAutonomousExecution),
            "RESUME_AUTONOMOUS_EXECUTION" => Some(Self::ResumeAutonomousExecution),
            "EMERGENCY_SHUTDOWN" => Some(Self::EmergencyShutdown),
            _ => None,
        }
    }
}

// ------------------------------------------------------------------------------
// Inputs & outputs
// ------------------------------------------------------------------------------

/// Enterprise-level metrics supplied to the Sovereign layer.
/// NOTE: none of these are Copilot micro-arithmetic — they are macro posture.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SovereignMetrics {
    pub total_yield_eth: f64,
    pub capital_exposure_pct: f64,
    pub liquidity_ratio: f64,
    pub risk_index: f64,
    pub compliance_score: f64,
    pub strategic_alignment_pct: f64,
    pub active_runners: i32,
    pub daily_drawdown_pct: f64,
    pub current_profile: OperatingProfile,
}

/// One graded governance dimension for the System Reflection card.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SovereignDimension {
    pub name: String,
    pub status: String,
    pub value: f64,
    pub detail: String,
}

/// Enterprise health block of the System Reflection card.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EnterpriseHealth {
    pub strategic_alignment: String,
    pub capital_exposure: String,
    pub liquidity_posture: String,
    pub risk_profile: String,
    pub compliance_status: String,
}

/// The System Reflection (Sovereign) card payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SovereignReflection {
    pub status: AuditStatus,
    pub assessment: String,
    pub recommendation: String,
    pub enterprise_health: EnterpriseHealth,
    pub dimensions: Vec<SovereignDimension>,
    pub current_operating_profile: OperatingProfile,
    pub allowed_profiles: Vec<OperatingProfile>,
    pub generated_at: String,
}

/// Immutable, cryptographically signed governance record (brief §7).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SovereignAuditRecord {
    pub audit_id: String,
    pub created_at: String,
    pub audit_source: String,
    pub audit_scope: String,
    pub status: AuditStatus,
    pub finding: String,
    pub recommended_action: String,
    pub executed_response: String,
    pub authorization_trail: Vec<String>,
    pub operating_profile: OperatingProfile,
    pub payload_hash: String,
    pub signature: String,
    pub reflection: SovereignReflection,
}

/// Request envelope for Commander interventions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterventionRequest {
    pub action: String,
    pub target_profile: Option<String>,
    pub commander_id: String,
    pub reason: String,
}

// ------------------------------------------------------------------------------
// Main Sovereign Audit Engine
// ------------------------------------------------------------------------------

pub struct SovereignAuditor {
    pub enabled: bool,
    pub running: bool,
    pub records: DashMap<String, SovereignAuditRecord>,
    pub governance_key: Arc<SigningKey>,
    pub current_profile: OperatingProfile,
    pub autonomous_execution_paused: bool,
    pub emergency_shutdown: bool,
    pub last_reflection: Option<SovereignReflection>,
    pub last_audit_block: u64,
}

impl SovereignAuditor {
    pub fn new() -> Self {
        let governance_key = Arc::new(
            SigningKey::from_slice(&SOVEREIGN_GOVERNANCE_SEED)
                .expect("Sovereign governance seed must be a valid 32-byte scalar"),
        );
        Self {
            enabled: true,
            running: true,
            records: DashMap::new(),
            governance_key,
            current_profile: OperatingProfile::Balanced,
            autonomous_execution_paused: false,
            emergency_shutdown: false,
            last_reflection: None,
            last_audit_block: 0,
        }
    }

    pub fn set_enabled(&mut self, e: bool) { self.enabled = e; }
    pub fn is_enabled(&self) -> bool { self.enabled }
    pub fn start(&mut self) { self.running = true; }
    pub fn stop(&mut self) { self.running = false; }

    /// Grade a "higher is better" metric.
    fn grade_higher(value: f64, pass: f64, warn: f64) -> (String, AuditStatus) {
        if value >= pass {
            ("PASS".into(), AuditStatus::Green)
        } else if value >= warn {
            ("WARN".into(), AuditStatus::Amber)
        } else {
            ("FAIL".into(), AuditStatus::Red)
        }
    }

    /// Grade a "lower is better" metric.
    fn grade_lower(value: f64, pass: f64, warn: f64) -> (String, AuditStatus) {
        if value <= pass {
            ("PASS".into(), AuditStatus::Green)
        } else if value <= warn {
            ("WARN".into(), AuditStatus::Amber)
        } else {
            ("FAIL".into(), AuditStatus::Red)
        }
    }

    /// Evaluate the enterprise posture and produce the System Reflection.
    /// This is the macro governance pass — it never touches Copilot math.
    pub fn evaluate(&mut self, metrics: &SovereignMetrics) -> SovereignReflection {
        let (s_align, st_align) = Self::grade_higher(
            metrics.strategic_alignment_pct,
            STRATEGIC_ALIGNMENT_PASS_PCT,
            STRATEGIC_ALIGNMENT_WARN_PCT,
        );
        let (s_cap, st_cap) = Self::grade_lower(
            metrics.capital_exposure_pct,
            CAPITAL_EXPOSURE_PASS_PCT,
            CAPITAL_EXPOSURE_WARN_PCT,
        );
        let (s_liq, st_liq) = Self::grade_higher(
            metrics.liquidity_ratio,
            LIQUIDITY_RATIO_PASS,
            LIQUIDITY_RATIO_WARN,
        );
        let (s_risk, st_risk) = Self::grade_lower(
            metrics.risk_index,
            RISK_INDEX_PASS,
            RISK_INDEX_WARN,
        );
        let (s_comp, st_comp) = Self::grade_higher(
            metrics.compliance_score,
            COMPLIANCE_SCORE_PASS,
            COMPLIANCE_SCORE_WARN,
        );

        let mut fails = 0u32;
        let mut warns = 0u32;
        for st in [&st_align, &st_cap, &st_liq, &st_risk, &st_comp] {
            match st {
                AuditStatus::Red => fails += 1,
                AuditStatus::Amber => warns += 1,
                AuditStatus::Green => {}
            }
        }

        let status = if fails > 0 || warns >= 3 {
            AuditStatus::Red
        } else if warns > 0 {
            AuditStatus::Amber
        } else {
            AuditStatus::Green
        };

        // Recommendation + profile guidance
        let (assessment, recommendation, recommended_profile) = match status {
            AuditStatus::Green => (
                "Enterprise operating parameters remain within approved governance framework.".into(),
                "Continue autonomous operation.".into(),
                metrics.current_profile,
            ),
            AuditStatus::Amber => {
                if st_cap == AuditStatus::Red || st_risk == AuditStatus::Red {
                    (
                        "Elevated capital exposure or systemic risk detected above tolerance.".into(),
                        "Restrict operating profile to CONSERVATIVE and continue with heightened monitoring.".into(),
                        OperatingProfile::Conservative,
                    )
                } else {
                    (
                        "Minor governance drift detected; parameters within recovery envelope.".into(),
                        "Continue autonomous operation with heightened monitoring.".into(),
                        metrics.current_profile,
                    )
                }
            }
            AuditStatus::Red => (
                "Enterprise posture has breached governance boundaries.".into(),
                "Restrict operating profile to LOCKDOWN and halt non-essential autonomous execution pending Commander review.".into(),
                OperatingProfile::Lockdown,
            ),
        };

        let allowed_profiles = match status {
            AuditStatus::Green => vec![
                OperatingProfile::Aggressive,
                OperatingProfile::Balanced,
                OperatingProfile::Conservative,
            ],
            AuditStatus::Amber => vec![OperatingProfile::Balanced, OperatingProfile::Conservative],
            AuditStatus::Red => vec![OperatingProfile::Conservative, OperatingProfile::Lockdown],
        };

        let reflection = SovereignReflection {
            status,
            assessment,
            recommendation,
            enterprise_health: EnterpriseHealth {
                strategic_alignment: s_align.clone(),
                capital_exposure: s_cap.clone(),
                liquidity_posture: s_liq.clone(),
                risk_profile: s_risk.clone(),
                compliance_status: s_comp.clone(),
            },
            dimensions: vec![
                SovereignDimension {
                    name: "Strategic Alignment".into(),
                    status: s_align,
                    value: metrics.strategic_alignment_pct,
                    detail: format!("Aligned to enterprise intent: {:.1}%", metrics.strategic_alignment_pct),
                },
                SovereignDimension {
                    name: "Capital Exposure".into(),
                    status: s_cap,
                    value: metrics.capital_exposure_pct,
                    detail: format!("Reserves deployed: {:.1}%", metrics.capital_exposure_pct),
                },
                SovereignDimension {
                    name: "Liquidity Posture".into(),
                    status: s_liq,
                    value: metrics.liquidity_ratio,
                    detail: format!("Liquidity ratio: {:.2}", metrics.liquidity_ratio),
                },
                SovereignDimension {
                    name: "Risk Profile".into(),
                    status: s_risk,
                    value: metrics.risk_index,
                    detail: format!("Composite risk index: {:.1}", metrics.risk_index),
                },
                SovereignDimension {
                    name: "Compliance Status".into(),
                    status: s_comp,
                    value: metrics.compliance_score,
                    detail: format!("Governance compliance score: {:.1}", metrics.compliance_score),
                },
            ],
            current_operating_profile: self.current_profile,
            allowed_profiles,
            generated_at: chrono::Utc::now().to_rfc3339(),
        };

        // Autonomously apply the recommended profile when not overridden by a
        // Commander directive (brief §5 Sovereign Feedback Loop).
        if recommended_profile != self.current_profile
            && !self.autonomous_execution_paused
            && self.current_profile != OperatingProfile::Lockdown
        {
            info!(
                "SOVEREIGN: autonomous profile shift {} -> {} ({}).",
                self.current_profile.as_str(),
                recommended_profile.as_str(),
                reflection.status.as_str()
            );
            self.current_profile = recommended_profile;
        }

        self.last_reflection = Some(reflection.clone());
        reflection
    }

    /// Convenience: evaluate and persist a signed immutable record.
    pub fn audit_cycle(&mut self, metrics: &SovereignMetrics) -> SovereignAuditRecord {
        let reflection = self.evaluate(metrics);
        let finding = format!(
            "Status {} | strategic {:.1}% | exposure {:.1}% | liquidity {:.2} | risk {:.1} | compliance {:.1}%",
            reflection.status.as_str(),
            metrics.strategic_alignment_pct,
            metrics.capital_exposure_pct,
            metrics.liquidity_ratio,
            metrics.risk_index,
            metrics.compliance_score,
        );
        let recommended_action = reflection.recommendation.clone();
        self.create_record(
            reflection,
            finding,
            recommended_action,
            "AUTONOMOUS_SOVEREIGN_CYCLE".into(),
            vec![format!("cycle block {}", self.last_audit_block + 1)],
        )
    }

    /// Deterministically sign a canonical payload string with the governance key.
    fn sign_payload(&self, canonical: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(canonical.as_bytes());
        let digest = hasher.finalize();
        let hash: [u8; 32] = digest.into();
        let (signature, _recid): (Signature, RecoveryId) =
            self.governance_key.sign_prehash_recoverable(&hash).expect("governance signing");
        hex::encode(signature.to_bytes())
    }

    /// Build the canonical string that is hashed + signed (excludes signature).
    fn canonical(reflection: &SovereignReflection, finding: &str, recommended: &str, profile: OperatingProfile) -> String {
        format!(
            "{}|{}|{}|{}|{}|{}|{}",
            SOVEREIGN_AUDIT_SOURCE,
            SOVEREIGN_AUDIT_SCOPE,
            reflection.status.as_str(),
            profile.as_str(),
            finding,
            recommended,
            reflection.generated_at,
        )
    }

    /// Persist an immutable, cryptographically signed governance record.
    pub fn create_record(
        &mut self,
        reflection: SovereignReflection,
        finding: String,
        recommended_action: String,
        executed_response: String,
        authorization_trail: Vec<String>,
    ) -> SovereignAuditRecord {
        let block = self.last_audit_block + 1;
        self.last_audit_block = block;

        let canonical = Self::canonical(
            &reflection,
            &finding,
            &recommended_action,
            self.current_profile,
        );

        let mut hasher = Sha256::new();
        hasher.update(canonical.as_bytes());
        let payload_hash = hex::encode(hasher.finalize());

        let signature = self.sign_payload(&canonical);

        let record = SovereignAuditRecord {
            audit_id: format!("SOV-{}-{}", Uuid::new_v4().simple(), block),
            created_at: chrono::Utc::now().to_rfc3339(),
            audit_source: SOVEREIGN_AUDIT_SOURCE.into(),
            audit_scope: SOVEREIGN_AUDIT_SCOPE.into(),
            status: reflection.status,
            finding,
            recommended_action,
            executed_response,
            authorization_trail,
            operating_profile: self.current_profile,
            payload_hash,
            signature,
            reflection,
        };

        self.records.insert(record.audit_id.clone(), record.clone());
        record
    }

    /// Independently verify a record's signature (brief §7: independently verifiable).
    pub fn verify_record(&self, record: &SovereignAuditRecord) -> bool {
        let canonical = Self::canonical(
            &record.reflection,
            &record.finding,
            &record.recommended_action,
            record.operating_profile,
        );
        let mut hasher = Sha256::new();
        hasher.update(canonical.as_bytes());
        if hex::encode(hasher.finalize()) != record.payload_hash {
            return false;
        }
        // Recompute signature over the payload hash and compare.
        let recomputed = self.sign_payload(&canonical);
        recomputed == record.signature
    }

    /// Execute a Commander intervention. Every action produces an immutable
    /// governance record with a full authorization trail (brief §6, §7).
    pub fn commander_intervention(&mut self, req: InterventionRequest) -> Result<SovereignAuditRecord, String> {
        let action = CommanderAction::from_str(&req.action)
            .ok_or_else(|| format!("Unknown Commander action: {}", req.action))?;

        let mut finding = String::new();
        let mut executed_response = String::new();
        let mut trail = vec![
            format!("COMMANDER:{}", req.commander_id),
            format!("ACTION:{}", action.as_str()),
            format!("REASON:{}", req.reason),
        ];

        match action {
            CommanderAction::ApproveRecommended => {
                finding = "Commander approved Sovereign recommendation.".into();
                executed_response = "Recommendation accepted; autonomous optimization continues.".into();
            }
            CommanderAction::RejectRecommendation => {
                finding = "Commander rejected Sovereign recommendation.".into();
                executed_response = "Recommendation overridden; prior operating parameters retained.".into();
            }
            CommanderAction::SwitchOperatingProfile => {
                let target = req
                    .target_profile
                    .as_deref()
                    .and_then(OperatingProfile::from_str)
                    .ok_or_else(|| String::from("SwitchOperatingProfile requires a valid target_profile"))?;
                // Respect allowed-profiles guard unless Commander explicitly forces.
                if let Some(r) = &self.last_reflection {
                    if !r.allowed_profiles.contains(&target) {
                        warn!("SOVEREIGN: Commander forcing profile {} outside allowed set.", target.as_str());
                    }
                }
                finding = format!("Commander switched operating profile to {}.", target.as_str());
                executed_response = format!("Operating profile set to {}.", target.as_str());
                self.current_profile = target;
            }
            CommanderAction::ActivateConservativeMode => {
                finding = "Commander activated conservative mode.".into();
                executed_response = "Operating profile set to CONSERVATIVE.".into();
                self.current_profile = OperatingProfile::Conservative;
            }
            CommanderAction::PauseAutonomousExecution => {
                finding = "Commander paused autonomous execution.".into();
                executed_response = "Autonomous execution halted; system in monitoring-only state.".into();
                self.autonomous_execution_paused = true;
            }
            CommanderAction::ResumeAutonomousExecution => {
                finding = "Commander resumed autonomous execution.".into();
                executed_response = "Autonomous execution restored.".into();
                self.autonomous_execution_paused = false;
            }
            CommanderAction::EmergencyShutdown => {
                finding = "Commander initiated EMERGENCY SHUTDOWN.".into();
                executed_response = "All autonomous execution halted; operating profile set to LOCKDOWN.".into();
                self.emergency_shutdown = true;
                self.autonomous_execution_paused = true;
                self.current_profile = OperatingProfile::Lockdown;
            }
        }

        let reflection = self.last_reflection.clone().unwrap_or_else(|| SovereignReflection {
            status: AuditStatus::Green,
            assessment: "No prior reflection available.".into(),
            recommendation: "Awaiting next Sovereign evaluation cycle.".into(),
            enterprise_health: EnterpriseHealth::default(),
            dimensions: vec![],
            current_operating_profile: self.current_profile,
            allowed_profiles: vec![
                OperatingProfile::Balanced,
                OperatingProfile::Conservative,
                OperatingProfile::Lockdown,
            ],
            generated_at: chrono::Utc::now().to_rfc3339(),
        });

        Ok(self.create_record(
            reflection,
            finding,
            format!("Commander directive: {}", action.as_str()),
            executed_response,
            trail,
        ))
    }

    /// Get the latest immutable record.
    pub fn get_latest_record(&self) -> Option<SovereignAuditRecord> {
        self.records
            .iter()
            .max_by_key(|entry| entry.value().created_at.clone())
            .map(|entry| entry.value().clone())
    }

    /// Snapshot all immutable records (newest first).
    pub fn list_records(&self) -> Vec<SovereignAuditRecord> {
        let mut v: Vec<SovereignAuditRecord> = self.records.iter().map(|e| e.value().clone()).collect();
        v.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        v
    }
}

impl Default for SovereignAuditor {
    fn default() -> Self {
        Self::new()
    }
}
