// ==============================================================================
// M011: Yield Aggregator
// Purpose: Scan and aggregate yield opportunities across protocols
// CGM Subsystem: Growth
// ==============================================================================

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct YieldOpportunity {
    pub protocol: String,
    pub pool: String,
    pub asset: String,
    pub apy: f64,
    pub tvl: f64,
    pub risk_score: f64,
    pub min_deposit: f64,
    pub lock_period_days: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskTolerance {
    Conservative,
    Moderate,
    Aggressive,
}

#[derive(Debug, Clone)]
pub struct YieldStrategy {
    pub opportunities: Vec<YieldOpportunity>,
    pub total_allocated: f64,
    pub expected_apy: f64,
    pub risk_adjusted_return: f64,
}

#[derive(Debug)]
pub struct YieldAggregator {
    pub enabled: bool,
    pub opportunities: HashMap<String, YieldOpportunity>,
    pub risk_tolerance: RiskTolerance,
    pub min_apy_threshold: f64,
    pub max_protocols: usize,
    pub total_scanned: u64,
    pub last_scan: Option<String>,
}

impl YieldAggregator {
    pub fn new() -> Self {
        Self {
            enabled: true,
            opportunities: HashMap::new(),
            risk_tolerance: RiskTolerance::Moderate,
            min_apy_threshold: 0.05,
            max_protocols: 10,
            total_scanned: 0,
            last_scan: None,
        }
    }

    pub fn scan_opportunity(&mut self, opportunity: YieldOpportunity) {
        if !self.enabled {
            return;
        }

        self.total_scanned += 1;
        self.opportunities.insert(
            format!("{}/{}", opportunity.protocol, opportunity.pool),
            opportunity,
        );
    }

    pub fn get_best_opportunities(&self, count: usize) -> Vec<&YieldOpportunity> {
        let mut opps: Vec<_> = self.opportunities.values().collect();
        opps.sort_by(|a, b| b.apy.partial_cmp(&a.apy).unwrap());
        opps.into_iter().take(count).collect()
    }

    pub fn filter_by_risk(&self, max_risk: f64) -> Vec<&YieldOpportunity> {
        self.opportunities
            .values()
            .filter(|o| o.risk_score <= max_risk && o.apy >= self.min_apy_threshold)
            .collect()
    }

    pub fn calculate_optimal_allocation(&self, capital: f64) -> YieldStrategy {
        let filtered = self.filter_by_risk(match self.risk_tolerance {
            RiskTolerance::Conservative => 0.2,
            RiskTolerance::Moderate => 0.5,
            RiskTolerance::Aggressive => 0.8,
        });

        let mut opportunities = filtered;
        opportunities.sort_by(|a, b| {
            let score_a = a.apy * (1.0 - a.risk_score);
            let score_b = b.apy * (1.0 - b.risk_score);
            score_b.partial_cmp(&score_a).unwrap()
        });

        let opportunities = opportunities.into_iter().take(self.max_protocols).cloned().collect::<Vec<_>>();

        let total_allocated = if opportunities.is_empty() {
            0.0
        } else {
            capital / opportunities.len() as f64
        };

        let expected_apy = if opportunities.is_empty() {
            0.0
        } else {
            opportunities.iter().map(|o| o.apy).sum::<f64>() / opportunities.len() as f64
        };

        let risk_adjusted_return = expected_apy * (1.0 - opportunities.iter().map(|o| o.risk_score).sum::<f64>() / opportunities.len() as f64);

        YieldStrategy {
            opportunities,
            total_allocated,
            expected_apy,
            risk_adjusted_return,
        }
    }

    pub fn get_stats(&self) -> String {
        format!(
            r#"{{"opportunities":{},"total_scanned":{},"risk_tolerance":"{:?}","min_apy":{:.2}}}"#,
            self.opportunities.len(),
            self.total_scanned,
            self.risk_tolerance,
            self.min_apy_threshold
        )
    }
}
