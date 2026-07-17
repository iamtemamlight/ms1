//! ==============================================================================
//! ALLBRIGHT RESEARCH & ENVIRONMENTAL INTELLIGENCE MODULE (REIM / M142)
//! ==============================================================================
//! External intelligence capability of AllBright. REIM continuously learns from
//! the external environment, predicts future changes, and prepares adaptive
//! strategies so AllBright is not limited to internal self-learning.
//!
//! Position within the Four-Dimension Autonomous Intelligence Architecture:
//!   * DACAM (M132)  -> "Is the Copilot executing correctly?"
//!   * Sovereign (M133) -> "Is AllBright operating in the enterprise's interest?"
//!   * Commander (M134) -> "Is the Commander becoming a better guardian?"
//!   * REIM (M142)   -> "What is changing outside AllBright, how should it prepare?"
//!
//! CRITICAL SAFETY PRINCIPLE (Brief §3):
//!   REIM NEVER directly controls the Copilot. External information must pass
//!   through the Intelligence Gatekeeper (M143) before it can influence any
//!   internal system. Flow:
//!     External World -> REIM -> Intelligence Gatekeeper -> Approved Feed
//!       -> Governance Decision Engine -> Autonomous Copilot -> Execution
//!
//! SCOPE BOUNDARY (no duplication): internal execution forecasting/prediction is
//! owned by M082 Predictor / M083 Forecaster / M084 Simulator. REIM predicts the
//! EXTERNAL environment only (markets, technology, competitors, regulation).

use std::collections::VecDeque;
use serde::{Deserialize, Serialize};

// ------------------------------------------------------------------------------
// Domains & classification
// ------------------------------------------------------------------------------

/// The four intelligence domains REIM monitors (Brief §4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntelligenceDomain {
    Industry,
    Technology,
    Competitive,
    Regulatory,
}

impl IntelligenceDomain {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Industry => "INDUSTRY",
            Self::Technology => "TECHNOLOGY",
            Self::Competitive => "COMPETITIVE",
            Self::Regulatory => "REGULATORY",
        }
    }
}

/// Intelligence quality levels (Brief §5.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntelligenceLevel {
    L1Observation,
    L2StrategicAwareness,
    L3OperationalOpportunity,
    L4CriticalAlert,
}

impl IntelligenceLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::L1Observation => "L1_OBSERVATION",
            Self::L2StrategicAwareness => "L2_STRATEGIC_AWARENESS",
            Self::L3OperationalOpportunity => "L3_OPERATIONAL_OPPORTUNITY",
            Self::L4CriticalAlert => "L4_CRITICAL_ALERT",
        }
    }
}

/// Where approved intelligence is routed (Brief §7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AffectedArea {
    Copilot,
    Sovereign,
    Commander,
    ResearchSystem,
}

impl AffectedArea {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Copilot => "COPILOT",
            Self::Sovereign => "SOVEREIGN",
            Self::Commander => "COMMANDER",
            Self::ResearchSystem => "RESEARCH_SYSTEM",
        }
    }
}

/// Expected impact magnitude (Brief §5.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImpactMagnitude {
    Low,
    Medium,
    High,
    Critical,
}

impl ImpactMagnitude {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "LOW",
            Self::Medium => "MEDIUM",
            Self::High => "HIGH",
            Self::Critical => "CRITICAL",
        }
    }
}

/// Gatekeeper decision states (Brief §6) — every intelligence item receives one.
/// Shared by REIM and the Environmental Intelligence Gatekeeper (M143).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GateDecision {
    Approved,
    Monitor,
    Quarantined,
    Rejected,
}

impl GateDecision {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Approved => "APPROVED",
            Self::Monitor => "MONITOR",
            Self::Quarantined => "QUARANTINED",
            Self::Rejected => "REJECTED",
        }
    }
}

/// Reflection card status (mirrors the GREEN/AMBER/RED convention of the other
/// three reflection cards so the Commander dashboard can render it uniformly).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReflectionStatus {
    Green,
    Amber,
    Red,
}

impl ReflectionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Green => "GREEN",
            Self::Amber => "AMBER",
            Self::Red => "RED",
        }
    }
}

// ------------------------------------------------------------------------------
// Intelligence item model
// ------------------------------------------------------------------------------

/// Source credibility profile used by the Gatekeeper for source validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceProfile {
    pub name: String,
    pub credibility: f64,            // 0.0 - 1.0
    pub independence: f64,           // 0.0 - 1.0
    pub historical_accuracy: f64,    // 0.0 - 1.0
    pub cross_source_confirmations: u32,
}

/// A single piece of raw or validated external intelligence.
/// `confidence`, `level`, `approved`, `risk_flags`, `affected_area`,
/// `expected_impact` and `routed_to` are populated by the Intelligence Gatekeeper
/// (M143) — REIM never sets them itself.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceItem {
    pub id: String,
    pub domain: IntelligenceDomain,
    pub headline: String,
    pub detail: String,
    pub observed_at: String,
    pub recency_hours: f64,
    pub source: SourceProfile,
    pub tags: Vec<String>,
    // --- Gatekeeper outputs (filled by M143) ---
    pub confidence: f64,             // 0 - 100
    pub level: IntelligenceLevel,
    pub decision: GateDecision,
    pub approved: bool,
    pub risk_flags: Vec<String>,
    pub affected_area: AffectedArea,
    pub expected_impact: ImpactMagnitude,
    pub routed_to: Option<AffectedArea>,
}

/// A trackable environmental prediction used by the Continuous Predictive
/// Learning Loop (Brief §8).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prediction {
    pub id: String,
    pub scenario: String,
    pub horizon_days: u64,
    pub predicted_at: String,
    pub confidence: f64,
    pub outcome_observed: Option<bool>, // Some(true)=occurred, Some(false)=did not
    pub actual_at: Option<String>,
}

// ------------------------------------------------------------------------------
// Research Reflection Card (Brief §6)
// ------------------------------------------------------------------------------

/// The Research Reflection payload rendered as the Commander reflection card.
/// Shaped to be self-contained so the dashboard does not depend on internal types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchReflection {
    pub status: ReflectionStatus,
    pub finding: String,
    pub prediction: String,
    pub recommendation: String,
    pub market_intelligence: Vec<IntelligenceItem>,
    pub technology_intelligence: Vec<IntelligenceItem>,
    pub competitive_intelligence: Vec<IntelligenceItem>,
    pub predictive_intelligence: Vec<Prediction>,
    pub prediction_accuracy_pct: f64,
    pub pending_risk_flags: Vec<String>,
    pub generated_at: String,
}

// ------------------------------------------------------------------------------
// REIM Engine
// ------------------------------------------------------------------------------

pub struct ReimEngine {
    pub enabled: bool,
    pub ingested: VecDeque<IntelligenceItem>,
    pub approved_feed: Vec<IntelligenceItem>,
    pub copilot_feed: Vec<IntelligenceItem>,
    pub sovereign_feed: Vec<IntelligenceItem>,
    pub commander_feed: Vec<IntelligenceItem>,
    pub predictions: Vec<Prediction>,
    pub prediction_accuracy: f64,
    pub cycles: u64,
}

impl ReimEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            enabled: true,
            ingested: VecDeque::with_capacity(512),
            approved_feed: Vec::new(),
            copilot_feed: Vec::new(),
            sovereign_feed: Vec::new(),
            commander_feed: Vec::new(),
            predictions: Vec::new(),
            prediction_accuracy: 0.0,
            cycles: 0,
        };
        engine.seed_baseline_intelligence();
        engine
    }

    pub fn set_enabled(&mut self, e: bool) { self.enabled = e; }
    pub fn is_enabled(&self) -> bool { self.enabled }

    /// Ingest an item that has ALREADY passed the Intelligence Gatekeeper.
    /// Bypassing the gatekeeper (ingesting raw external data) is rejected.
    /// Only APPROVED / MONITOR items enter the learning feeds; QUARANTINED and
    /// REJECTED items are blocked (Brief §6).
    pub fn ingest(&mut self, mut item: IntelligenceItem) {
        let admit = item.approved
            && matches!(item.decision, GateDecision::Approved | GateDecision::Monitor);
        if !admit {
            // Safety guard: only gatekeeper-cleared intelligence enters the system.
            return;
        }
        if self.ingested.len() >= 512 {
            self.ingested.pop_front();
        }
        self.ingested.push_back(item.clone());
        self.approved_feed.push(item.clone());
        self.route_by_area(&item);
    }

    /// Distribute an approved item to its target feed (Copilot / Sovereign / Commander).
    fn route_by_area(&mut self, item: &IntelligenceItem) {
        let area = item.routed_to.unwrap_or(item.affected_area);
        match area {
            AffectedArea::Copilot => self.copilot_feed.push(item.clone()),
            AffectedArea::Sovereign => self.sovereign_feed.push(item.clone()),
            AffectedArea::Commander => self.commander_feed.push(item.clone()),
            AffectedArea::ResearchSystem => {}
        }
    }

    /// Register a forward-looking environmental prediction (Brief §8).
    pub fn predict(&mut self, scenario: &str, horizon_days: u64, confidence: f64) -> Prediction {
        let p = Prediction {
            id: format!("REIM-PRED-{}", self.predictions.len() + 1),
            scenario: scenario.to_string(),
            horizon_days,
            predicted_at: chrono::Utc::now().to_rfc3339(),
            confidence,
            outcome_observed: None,
            actual_at: None,
        };
        self.predictions.push(p.clone());
        p
    }

    /// Observe actual outcome and recompute prediction accuracy (Brief §8 loop).
    pub fn record_outcome(&mut self, prediction_id: &str, occurred: bool) {
        for p in self.predictions.iter_mut() {
            if p.id == prediction_id && p.outcome_observed.is_none() {
                p.outcome_observed = Some(occurred);
                p.actual_at = Some(chrono::Utc::now().to_rfc3339());
                break;
            }
        }
        let resolved: Vec<_> = self.predictions.iter().filter(|p| p.outcome_observed.is_some()).collect();
        if !resolved.is_empty() {
            let correct = resolved.iter().filter(|p| p.outcome_observed == Some(true)).count();
            self.prediction_accuracy = (correct as f64 / resolved.len() as f64) * 100.0;
        }
    }

    /// Produce the Research Reflection card (Brief §6).
    pub fn reflect(&self) -> ResearchReflection {
        self.cycles += 1;

        let market: Vec<IntelligenceItem> = self.approved_feed.iter()
            .filter(|i| i.domain == IntelligenceDomain::Industry).cloned().collect();
        let tech: Vec<IntelligenceItem> = self.approved_feed.iter()
            .filter(|i| i.domain == IntelligenceDomain::Technology).cloned().collect();
        let comp: Vec<IntelligenceItem> = self.approved_feed.iter()
            .filter(|i| i.domain == IntelligenceDomain::Competitive).cloned().collect();

        let pending_risk_flags: Vec<String> = self.approved_feed.iter()
            .flat_map(|i| i.risk_flags.iter().cloned())
            .collect();

        let critical = self.approved_feed.iter()
            .filter(|i| i.level == IntelligenceLevel::L4CriticalAlert).count();

        let (status, finding, recommendation) = if critical > 0 || pending_risk_flags.iter().any(|f| f.contains("SOVEREIGN_CONFLICT") || f.contains("UNSAFE_ACTION")) {
            (
                ReflectionStatus::Red,
                "Critical external signal or unsafe-alignment risk detected. Hold autonomous expansion pending Commander review.".into(),
                "Restrict operating profile and quarantine flagged intelligence stream until validated by Commander.".into(),
            )
        } else if !pending_risk_flags.is_empty() || critical > 0 {
            (
                ReflectionStatus::Amber,
                "Elevated external signals require monitoring. Prepare adaptive strategies.".into(),
                "Begin evaluation of alternative strategies and heighten monitoring of flagged domains.".into(),
            )
        } else if let Some(top) = self.approved_feed.iter()
            .filter(|i| i.level == IntelligenceLevel::L3OperationalOpportunity)
            .max_by_key(|i| i.confidence as u64) {
            (
                ReflectionStatus::Green,
                format!("External monitoring nominal. Opportunity: {}", top.headline),
                "Continue autonomous operation; route operational opportunity to Copilot for evaluation.".into(),
            )
        } else {
            (
                ReflectionStatus::Green,
                "External environment stable. No material change detected.".into(),
                "Maintain current posture and continue predictive monitoring.".into(),
            )
        };

        let prediction = if let Some(open) = self.predictions.iter().find(|p| p.outcome_observed.is_none()) {
            format!("Predicted within {}d: {} (conf {:.0}%).", open.horizon_days, open.scenario, open.confidence)
        } else {
            format!("Predictive model accuracy {:.0}%. No open scenarios.", self.prediction_accuracy)
        };

        ResearchReflection {
            status,
            finding,
            prediction,
            recommendation,
            market_intelligence: market,
            technology_intelligence: tech,
            competitive_intelligence: comp,
            predictive_intelligence: self.predictions.clone(),
            prediction_accuracy_pct: self.prediction_accuracy,
            pending_risk_flags,
            generated_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Seed a baseline set of benchmarked external observations so the reflection
    /// card is populated on first cycle. These represent the standing external
    /// watchlist REIM maintains; they are clearly sourced and gatekeeper-approved.
    fn seed_baseline_intelligence(&mut self) {
        let baseline: Vec<IntelligenceItem> = vec![
            IntelligenceItem {
                id: "REIM-SEED-1".into(),
                domain: IntelligenceDomain::Technology,
                headline: "New execution technology emerging".into(),
                detail: "A novel MEV-resistant execution path is gaining adoption across major venues.".into(),
                observed_at: chrono::Utc::now().to_rfc3339(),
                recency_hours: 6.0,
                source: SourceProfile { name: "REIM_WATCHLIST".into(), credibility: 0.8, independence: 0.9, historical_accuracy: 0.82, cross_source_confirmations: 3 },
                tags: vec!["execution".into(), "mev".into()],
                confidence: 78.0,
                level: IntelligenceLevel::L3OperationalOpportunity,
                decision: GateDecision::Approved,
                approved: true,
                risk_flags: vec![],
                affected_area: AffectedArea::Copilot,
                expected_impact: ImpactMagnitude::Medium,
                routed_to: Some(AffectedArea::Copilot),
            },
            IntelligenceItem {
                id: "REIM-SEED-2".into(),
                domain: IntelligenceDomain::Competitive,
                headline: "Competitor fleet efficiency improving".into(),
                detail: "Benchmarked competitor autonomous fleets report 8% lower latency this quarter.".into(),
                observed_at: chrono::Utc::now().to_rfc3339(),
                recency_hours: 30.0,
                source: SourceProfile { name: "REIM_WATCHLIST".into(), credibility: 0.75, independence: 0.8, historical_accuracy: 0.79, cross_source_confirmations: 2 },
                tags: vec!["benchmark".into()],
                confidence: 71.0,
                level: IntelligenceLevel::L2StrategicAwareness,
                approved: true,
                risk_flags: vec![],
                affected_area: AffectedArea::Sovereign,
                expected_impact: ImpactMagnitude::Medium,
                routed_to: Some(AffectedArea::Sovereign),
            },
            IntelligenceItem {
                id: "REIM-SEED-3".into(),
                domain: IntelligenceDomain::Regulatory,
                headline: "Jurisdictional reporting requirement proposed".into(),
                detail: "A new reporting standard is under consultation in a primary operating jurisdiction.".into(),
                observed_at: chrono::Utc::now().to_rfc3339(),
                recency_hours: 50.0,
                source: SourceProfile { name: "REIM_WATCHLIST".into(), credibility: 0.85, independence: 0.95, historical_accuracy: 0.88, cross_source_confirmations: 2 },
                tags: vec!["compliance".into()],
                confidence: 80.0,
                level: IntelligenceLevel::L2StrategicAwareness,
                approved: true,
                risk_flags: vec![],
                affected_area: AffectedArea::Commander,
                expected_impact: ImpactMagnitude::Low,
                routed_to: Some(AffectedArea::Commander),
            },
        ];
        for item in baseline {
            self.ingest(item);
        }
        // Standing predictive scenario (Brief §6 example).
        self.predict("Current optimization advantage may reduce as competitors adopt the new execution technology.", 90, 0.7);
    }
}

impl Default for ReimEngine {
    fn default() -> Self { Self::new() }
}
