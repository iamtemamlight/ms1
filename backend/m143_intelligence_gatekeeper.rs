//! ==============================================================================
//! ALLBRIGHT INTELLIGENCE GATEKEEPER (M143)
//! ==============================================================================
//! Mandatory safety layer between REIM (M142) and the AllBright execution system
//! (Brief §3, §5). Its purpose is to ensure that ONLY verified, relevant, and
//! strategically safe intelligence influences AllBright.
//!
//! No external information may bypass validation. The correct flow is:
//!   External World -> REIM -> Intelligence Gatekeeper -> Approved Intelligence
//!   Feed -> Governance Decision Engine -> Autonomous Copilot -> Execution
//!
//! SCOPE BOUNDARY (no duplication):
//!   * `constitution_guard` (M-ConstitutionGuard) is a PRE-TRADE validation gate
//!     for execution proposals. The Intelligence Gatekeeper validates INTELLIGENCE
//!     FEEDS, not trade proposals — a distinct responsibility.
//!   * `security_gate` is network/security oriented. This module is intelligence
//!     provenance + strategic-safety oriented.

use crate::m142_reim::{
    AffectedArea, ImpactMagnitude, IntelligenceItem, IntelligenceLevel,
};

/// Confidence (0-100) at/above which intelligence is admitted to the system.
pub const APPROVAL_THRESHOLD: f64 = 50.0;

/// Recency (hours) at/under which the recency factor is full weight.
const RECENCY_FULL_HOURS: f64 = 24.0;
/// Recency (hours) at/over which the recency factor drops to floor.
const RECENCY_FLOOR_HOURS: f64 = 168.0;
const RECENCY_FLOOR: f64 = 0.3;

/// Full Gatekeeper verdict for an item.
#[derive(Debug, Clone, Serialize)]
pub struct GatekeeperVerdict {
    pub item_id: String,
    pub confidence: f64,
    pub approved: bool,
    pub level: IntelligenceLevel,
    pub risk_flags: Vec<String>,
    pub affected_area: AffectedArea,
    pub expected_impact: ImpactMagnitude,
    pub routed_to: AffectedArea,
    pub evaluated_at: String,
}

pub struct IntelligenceGatekeeper {
    pub enabled: bool,
    pub evaluations: u64,
    pub approved: u64,
    pub rejected: u64,
}

impl IntelligenceGatekeeper {
    pub fn new() -> Self {
        Self {
            enabled: true,
            evaluations: 0,
            approved: 0,
            rejected: 0,
        }
    }

    pub fn set_enabled(&mut self, e: bool) { self.enabled = e; }
    pub fn is_enabled(&self) -> bool { self.enabled }

    /// §5.1 Source Validation — compute a 0-100 confidence score from credibility,
    /// independence, historical accuracy, recency, and cross-source confirmation.
    pub fn validate_source(&self, item: &IntelligenceItem) -> f64 {
        let s = &item.source;
        let recency = if item.recency_hours <= RECENCY_FULL_HOURS {
            1.0
        } else if item.recency_hours >= RECENCY_FLOOR_HOURS {
            RECENCY_FLOOR
        } else {
            let t = (item.recency_hours - RECENCY_FULL_HOURS)
                / (RECENCY_FLOOR_HOURS - RECENCY_FULL_HOURS);
            1.0 + t * (RECENCY_FLOOR - 1.0)
        };
        let cross = (s.cross_source_confirmations.min(3) as f64) / 3.0;

        let score = s.credibility * 0.30
            + s.independence * 0.15
            + s.historical_accuracy * 0.30
            + recency * 0.10
            + cross * 0.15;

        (score * 100.0).clamp(0.0, 100.0)
    }

    /// §5.2 Intelligence Quality Assessment — classify by proposed level.
    /// (The item carries a proposed level; the gatekeeper may escalate to L4 when
    /// a hard risk flag is present.)
    fn assess_quality(&self, level: IntelligenceLevel) -> IntelligenceLevel {
        level
    }

    /// §5.3 Risk Assessment — detect unsafe / exposure / sovereign-conflict /
    /// biased-optimization risks and produce human-readable flags.
    fn assess_risk(&self, item: &IntelligenceItem) -> Vec<String> {
        let mut flags = Vec::new();
        let blob = format!("{} {} {}", item.headline, item.detail, item.tags.join(" ")).to_lowercase();

        if blob.contains("exploit") || blob.contains("bypass") || blob.contains("unsafe") {
            flags.push("UNSAFE_ACTION_RISK".into());
        }
        if blob.contains("leverage") || blob.contains("exposure") || blob.contains("over-allocate") {
            flags.push("INCREASED_EXPOSURE_RISK".into());
        }
        if blob.contains("regulat") && item.affected_area == AffectedArea::Sovereign {
            flags.push("SOVEREIGN_CONFLICT_RISK".into());
        }
        if blob.contains("only strategy") || blob.contains("always better") || blob.contains("ignore") {
            flags.push("BIASED_OPTIMIZATION_RISK".into());
        }
        flags
    }

    /// §5.4 Impact Analysis — confirm affected area and expected impact and route.
    fn assess_impact(&self, item: &IntelligenceItem, risk_flags: &[String]) -> (AffectedArea, ImpactMagnitude, AffectedArea) {
        // Hard sovereign-conflict items are forced to the Sovereign review path.
        if risk_flags.iter().any(|f| f == "SOVEREIGN_CONFLICT_RISK") {
            return (AffectedArea::Sovereign, ImpactMagnitude::High, AffectedArea::Sovereign);
        }
        let routed = if risk_flags.iter().any(|f| f == "UNSAFE_ACTION_RISK" || f == "INCREASED_EXPOSURE_RISK") {
            AffectedArea::Sovereign
        } else {
            item.affected_area
        };
        (item.affected_area, item.expected_impact, routed)
    }

    /// Run the full gate over an item, mutating its gatekeeper-controlled fields.
    /// Returns true if approved (admitted to the approved feed).
    pub fn gate(&mut self, item: &mut IntelligenceItem) -> bool {
        self.evaluations += 1;
        let confidence = self.validate_source(item);
        item.confidence = confidence;
        item.level = self.assess_quality(item.level);

        let risk_flags = self.assess_risk(item);
        item.risk_flags = risk_flags.clone();

        let (affected, impact, routed) = self.assess_impact(item, &risk_flags);
        item.affected_area = affected;
        item.expected_impact = impact;
        item.routed_to = Some(routed);

        // Escalate to critical alert when a hard risk flag is present.
        if !risk_flags.is_empty() && risk_flags.iter().any(|f| f.starts_with("UNSAFE") || f.starts_with("SOVEREIGN_CONFLICT")) {
            item.level = IntelligenceLevel::L4CriticalAlert;
        }

        if confidence < APPROVAL_THRESHOLD {
            item.approved = false;
            self.rejected += 1;
            return false;
        }
        item.approved = true;
        self.approved += 1;
        true
    }

    /// Produce a standalone verdict snapshot (for logging / records).
    pub fn verdict(&self, item: &IntelligenceItem) -> GatekeeperVerdict {
        GatekeeperVerdict {
            item_id: item.id.clone(),
            confidence: item.confidence,
            approved: item.approved,
            level: item.level,
            risk_flags: item.risk_flags.clone(),
            affected_area: item.affected_area,
            expected_impact: item.expected_impact,
            routed_to: item.routed_to.unwrap_or(item.affected_area),
            evaluated_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

impl Default for IntelligenceGatekeeper {
    fn default() -> Self { Self::new() }
}
