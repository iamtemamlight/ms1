//! Benchmarking Framework (spec §16).
//!
//! The Independent Auditor Agent (IAA) continuously benchmarks the platform
//! across the 14 domains defined in §16.1 and follows the §16.2 process:
//!
//! > Establish Baseline → Identify Gaps → Recommend Improvements → Track Progress → Re-benchmark
//!
//! This module is the canonical, dependency-free benchmarking engine used by the
//! IAA. Each domain is scored on a 0.0..=1.0 scale; a domain below the
//! `gap_threshold` is treated as a gap requiring a recommendation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The 14 benchmarking domains (spec §16.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BenchmarkDomain {
    Architecture,
    Security,
    Governance,
    Engineering,
    Performance,
    Testing,
    Observability,
    Reliability,
    Maintainability,
    Scalability,
    Deployment,
    AiEngineering,
    OperationalReadiness,
    ZeroTrustCompliance,
}

impl BenchmarkDomain {
    /// All 14 benchmarking domains in canonical order (§16.1).
    pub fn all() -> [BenchmarkDomain; 14] {
        [
            BenchmarkDomain::Architecture,
            BenchmarkDomain::Security,
            BenchmarkDomain::Governance,
            BenchmarkDomain::Engineering,
            BenchmarkDomain::Performance,
            BenchmarkDomain::Testing,
            BenchmarkDomain::Observability,
            BenchmarkDomain::Reliability,
            BenchmarkDomain::Maintainability,
            BenchmarkDomain::Scalability,
            BenchmarkDomain::Deployment,
            BenchmarkDomain::AiEngineering,
            BenchmarkDomain::OperationalReadiness,
            BenchmarkDomain::ZeroTrustCompliance,
        ]
    }

    /// Human-readable name used in benchmark reports.
    pub fn label(&self) -> &'static str {
        match self {
            BenchmarkDomain::Architecture => "Architecture",
            BenchmarkDomain::Security => "Security",
            BenchmarkDomain::Governance => "Governance",
            BenchmarkDomain::Engineering => "Engineering",
            BenchmarkDomain::Performance => "Performance",
            BenchmarkDomain::Testing => "Testing",
            BenchmarkDomain::Observability => "Observability",
            BenchmarkDomain::Reliability => "Reliability",
            BenchmarkDomain::Maintainability => "Maintainability",
            BenchmarkDomain::Scalability => "Scalability",
            BenchmarkDomain::Deployment => "Deployment",
            BenchmarkDomain::AiEngineering => "AI Engineering",
            BenchmarkDomain::OperationalReadiness => "Operational Readiness",
            BenchmarkDomain::ZeroTrustCompliance => "Zero Trust Compliance",
        }
    }
}

/// A single domain's score (0.0..=1.0).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DomainScore {
    pub domain: BenchmarkDomain,
    pub score: f64,
}

/// Benchmarking engine following the §16.2 process.
pub struct BenchmarkEngine {
    scores: HashMap<BenchmarkDomain, f64>,
    gap_threshold: f64,
}

impl BenchmarkEngine {
    /// Create an engine that flags domains scoring below `gap_threshold` as gaps.
    pub fn new(gap_threshold: f64) -> Self {
        Self {
            scores: HashMap::new(),
            gap_threshold: gap_threshold.clamp(0.0, 1.0),
        }
    }

    /// Step 1 (§16.2): establish / record a baseline (or current) score.
    pub fn set_score(&mut self, domain: BenchmarkDomain, score: f64) {
        self.scores.insert(domain, score.clamp(0.0, 1.0));
    }

    /// Current score for a domain (0.0 if unscored).
    pub fn score(&self, domain: BenchmarkDomain) -> f64 {
        self.scores.get(&domain).copied().unwrap_or(0.0)
    }

    /// Step 2 (§16.2): identify *scored* domains below the gap threshold.
    /// Domains that have not yet been benchmarked (no score recorded) are not
    /// reported as gaps — they are "not yet baseline" rather than "failing".
    pub fn gaps(&self) -> Vec<BenchmarkDomain> {
        BenchmarkDomain::all()
            .into_iter()
            .filter(|d| self.scores.contains_key(d) && self.score(*d) < self.gap_threshold)
            .collect()
    }

    /// Step 3 (§16.2): recommend improvements for each identified gap.
    /// Returns one recommendation per gap.
    pub fn recommendations(&self) -> Vec<(BenchmarkDomain, String)> {
        self.gaps()
            .into_iter()
            .map(|d| {
                (
                    d,
                    format!(
                        "Improve {} (score {:.2}, below threshold {:.2})",
                        d.label(),
                        self.score(d),
                        self.gap_threshold
                    ),
                )
            })
            .collect()
    }

    /// Step 5 (§16.2): re-benchmark — diff current scores against a previous
    /// snapshot, returning per-domain deltas (positive = improvement).
    pub fn re_benchmark(&self, previous: &BenchmarkEngine) -> HashMap<BenchmarkDomain, f64> {
        BenchmarkDomain::all()
            .into_iter()
            .map(|d| (d, self.score(d) - previous.score(d)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fourteen_domains_defined() {
        assert_eq!(BenchmarkDomain::all().len(), 14);
        assert_eq!(BenchmarkDomain::ZeroTrustCompliance.label(), "Zero Trust Compliance");
    }

    #[test]
    fn baseline_gap_recommend_flow() {
        let mut engine = BenchmarkEngine::new(0.8);
        engine.set_score(BenchmarkDomain::Security, 0.95);
        engine.set_score(BenchmarkDomain::Testing, 0.50); // gap
        engine.set_score(BenchmarkDomain::Governance, 0.60); // gap

        let gaps = engine.gaps();
        assert_eq!(gaps.len(), 2);
        assert!(gaps.contains(&BenchmarkDomain::Testing));
        assert!(gaps.contains(&BenchmarkDomain::Governance));
        assert!(!gaps.contains(&BenchmarkDomain::Security));

        let recs = engine.recommendations();
        assert_eq!(recs.len(), 2);
    }

    #[test]
    fn rebenchmark_reports_deltas() {
        let mut prev = BenchmarkEngine::new(0.8);
        prev.set_score(BenchmarkDomain::Security, 0.70);

        let mut current = BenchmarkEngine::new(0.8);
        current.set_score(BenchmarkDomain::Security, 0.90);

        let deltas = current.re_benchmark(&prev);
        let delta = deltas.get(&BenchmarkDomain::Security).copied().unwrap();
        assert!((delta - 0.20).abs() < 1e-9, "expected ~0.20, got {delta}");
    }
}
