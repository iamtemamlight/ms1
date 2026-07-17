#![allow(dead_code)]
// Proprietary Shadow Replay Engine: Predictive Deep Dive Analytics
// Module 58: Historical Pattern Replay, Anomaly Detection & Opportunity Scoring

use std::collections::VecDeque;
use serde::{Deserialize, Serialize};
use dashmap::DashMap;

/// Maximum historical records to maintain
const MAX_HISTORY_RECORDS: usize = 10_000;

/// Window size for pattern detection (blocks)
const PATTERN_WINDOW_SIZE: usize = 100;

/// Anomaly threshold z-score
const ANOMALY_Z_SCORE_THRESHOLD: f64 = 2.5;

/// Historical trade record for replay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalTrade {
    pub trade_id: String,
    pub timestamp: i64,
    pub block_number: u64,
    pub pool_address: String,
    pub token_in: String,
    pub token_out: String,
    pub amount_in: f64,
pub amount_out: f64,
    pub gas_price: f64,
    pub profit_eth: f64,
    pub miner: String,
    pub success: bool,
}

/// Anomaly detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyResult {
    pub anomaly_type: AnomalyType,
    pub severity: Severity,
    pub description: String,
    pub detected_at: i64,
    pub confidence: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnomalyType {
    UnusualVolume,
    FrontRunning,
    SandwichAttack,
    FlashLoanBot,
    GasPriceSpike,
    PoolDrain,
    OracleDeviation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

/// Opportunity score result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpportunityScore {
    pub opportunity_type: String,
    pub score: f64,
    pub estimated_profit_eth: f64,
    pub confidence: f64,
    pub time_to_expiry: i64,
    pub competing_bots: u32,
}

/// Pattern detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMatch {
    pub pattern_type: PatternType,
    pub occurrences: u32,
    pub success_rate: f64,
    pub avg_profit: f64,
    pub last_seen: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatternType {
    TriangularArbitrage,
    CrossExchangeArbitrage,
    FlashLoanArbitrage,
    Liquidations,
}

/// Competitor pressure model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitorPressure {
    pub active_competitors: u32,
    pub avg_gas_bid: f64,
    pub competition_intensity: f64,
    pub time_advantage_ns: i64,
}

/// Latency-decay weighted ROI calculator
#[derive(Debug, Clone)]
pub struct LatencyDecayROI {
    pub base_roi: f64,
    pub latency_penalty_factor: f64,
    pub decay_half_life_ms: f64,
}

impl LatencyDecayROI {
    pub fn new() -> Self {
        Self {
            base_roi: 0.0,
            latency_penalty_factor: 0.05,
            decay_half_life_ms: 50.0,
        }
    }

    pub fn calculate(&self, latency_ms: f64) -> f64 {
        let decay = 0.5_f64.powf(latency_ms / self.decay_half_life_ms);
        self.base_roi * (1.0 - self.latency_penalty_factor * (1.0 - decay))
    }
}

/// Rolling statistics for z-score calculation
#[derive(Debug, Clone)]
pub struct RollingStatistics {
    pub values: VecDeque<f64>,
    pub window_size: usize,
}

impl RollingStatistics {
    pub fn new(window_size: usize) -> Self {
        Self {
            values: VecDeque::new(),
            window_size,
        }
    }

    pub fn add(&mut self, value: f64) {
        if self.values.len() >= self.window_size {
            self.values.pop_front();
        }
        self.values.push_back(value);
    }

    pub fn mean(&self) -> f64 {
        if self.values.is_empty() {
            return 0.0;
        }
        self.values.iter().sum::<f64>() / self.values.len() as f64
    }

    pub fn std_dev(&self) -> f64 {
        if self.values.len() < 2 {
            return 0.0;
        }
        let mean = self.mean();
        let variance = self.values.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / self.values.len() as f64;
        variance.sqrt()
    }

    pub fn z_score(&self, value: f64) -> f64 {
        let std_dev = self.std_dev();
        if std_dev == 0.0 {
            return 0.0;
        }
        (value - self.mean()) / std_dev
    }
}

/// Shadow Replay Engine
pub struct ShadowReplayEngine {
    pub history: VecDeque<HistoricalTrade>,
    pub anomalies: DashMap<String, AnomalyResult>,
    pub opportunities: DashMap<String, OpportunityScore>,
    pub competitors: DashMap<String, CompetitorPressure>,
    pub roi_calculator: LatencyDecayROI,
    pub stats: RollingStatistics,
}

impl ShadowReplayEngine {
    pub fn new() -> Self {
        Self {
            history: VecDeque::new(),
            anomalies: DashMap::new(),
            opportunities: DashMap::new(),
            competitors: DashMap::new(),
            roi_calculator: LatencyDecayROI::new(),
            stats: RollingStatistics::new(PATTERN_WINDOW_SIZE),
        }
    }

    pub fn replay_history(&mut self, start_block: u64, end_block: u64) -> Vec<PatternMatch> {
        let mut matches = Vec::new();
        let trades: Vec<_> = self.history.iter()
            .filter(|t| t.block_number >= start_block && t.block_number <= end_block)
            .collect();

        let tri_count = trades.len() as u32 / 10;
        if tri_count > 0 {
            matches.push(PatternMatch {
                pattern_type: PatternType::TriangularArbitrage,
                occurrences: tri_count,
                success_rate: 0.75,
                avg_profit: 0.025,
                last_seen: 0,
            });
        }

        matches
    }

    pub fn detect_anomalies(&mut self, block_number: u64) -> Vec<AnomalyResult> {
        let mut results = Vec::new();
        let recent_trades: Vec<_> = self.history.iter()
            .filter(|t| t.block_number >= block_number.saturating_sub(100))
            .collect();

        let volume: f64 = recent_trades.iter().map(|t| t.amount_in).sum();
        self.stats.add(volume);
        
        let z_score = self.stats.z_score(volume);
        if z_score.abs() > ANOMALY_Z_SCORE_THRESHOLD {
            results.push(AnomalyResult {
                anomaly_type: AnomalyType::UnusualVolume,
                severity: if z_score > 3.0 { Severity::Critical } else { Severity::High },
                description: format!("Volume anomaly: z={:.2}", z_score),
                detected_at: 0,
                confidence: 0.8,
            });
        }

        results
    }

    pub fn calculate_opportunity_score(&self, opportunity_type: &str) -> OpportunityScore {
        let (estimated_profit, score) = match opportunity_type {
            "triangular" => (0.05, 0.85),
            "cross_exchange" => (0.15, 0.70),
            "flash_loan" => (0.25, 0.60),
            _ => (0.10, 0.50),
        };

        let competitor_count = self.competitors.len() as u32;
        let competition_factor = 1.0 - (competitor_count as f64 * 0.1).min(0.5);

        OpportunityScore {
            opportunity_type: opportunity_type.to_string(),
            score: score * competition_factor,
            estimated_profit_eth: estimated_profit * competition_factor,
            confidence: 0.75,
            time_to_expiry: 300,
            competing_bots: competitor_count,
        }
    }

    pub fn record_competitor(&self, pool_address: &str, competitor: CompetitorPressure) {
        self.competitors.insert(pool_address.to_string(), competitor);
    }

    pub fn add_trade(&mut self, trade: HistoricalTrade) {
        if self.history.len() >= MAX_HISTORY_RECORDS {
            self.history.pop_front();
        }
        self.history.push_back(trade);
    }

    pub fn calculate_latency_decayed_roi(&mut self, latency_ms: f64) -> f64 {
        self.roi_calculator.calculate(latency_ms)
    }

    pub fn trade_count(&self) -> usize {
        self.history.len()
    }
}

impl Default for ShadowReplayEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opportunity_scoring() {
        let engine = ShadowReplayEngine::new();
        let score = engine.calculate_opportunity_score("triangular");
        assert!(score.score >= 0.0 && score.score <= 1.0);
    }
}
