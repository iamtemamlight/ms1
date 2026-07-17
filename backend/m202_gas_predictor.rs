// ==============================================================================
// M202: Predictive Gas Price Model
// Purpose: Real-time EIP-1559 gas price prediction with L1/L2 differential modeling
//          Replaces fixed gas thresholds with probabilistic forecasts
// CGM Subsystem: Efficiency
// ==============================================================================

use std::collections::VecDeque;
use serde::{Deserialize, Serialize};

/// Gas price prediction horizon in blocks
const PREDICTION_HORIZON_BLOCKS: usize = 12;

/// Historical data window for gas predictions
const HISTORY_WINDOW_SIZE: usize = 200;

/// EIP-1559 gas parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasParameters {
    pub base_fee_gwei: f64,
    pub priority_fee_gwei: f64,
    pub max_fee_gwei: f64,
    pub block_number: u64,
    pub timestamp_ms: u64,
    pub gas_used_ratio: f64,
    pub burnt_fees_eth: f64,
}

/// L1/L2 gas differential model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L1L2GasModel {
    pub l1_base_fee_gwei: f64,
    pub l1_priority_fee_gwei: f64,
    pub l2_base_fee_gwei: f64,
    pub l2_priority_fee_gwei: f64,
    pub l1_calldata_cost_gwei: f64,
    pub l1_to_l2_message_cost_gwei: f64,
    pub l2_gas_price_ratio: f64,
}

impl Default for L1L2GasModel {
    fn default() -> Self {
        Self {
            l1_base_fee_gwei: 15.0,
            l1_priority_fee_gwei: 2.0,
            l2_base_fee_gwei: 0.01,
            l2_priority_fee_gwei: 0.001,
            l1_calldata_cost_gwei: 16.0,
            l1_to_l2_message_cost_gwei: 1000.0,
            l2_gas_price_ratio: 0.001,
        }
    }
}

/// Market congestion state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CongestionState {
    Low,
    Medium,
    High,
    Extreme,
}

impl CongestionState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "LOW",
            Self::Medium => "MEDIUM",
            Self::High => "HIGH",
            Self::Extreme => "EXTREME",
        }
    }

    pub fn multiplier(&self) -> f64 {
        match self {
            Self::Low => 1.0,
            Self::Medium => 1.5,
            Self::High => 2.5,
            Self::Extreme => 4.0,
        }
    }
}

/// Gas price prediction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasPricePrediction {
    pub predicted_base_fee_gwei: f64,
    pub predicted_priority_fee_gwei: f64,
    pub predicted_max_fee_gwei: f64,
    pub confidence: f64,
    pub prediction_horizon_blocks: usize,
    pub congestion_state: CongestionState,
    pub l1_l2_model: L1L2GasModel,
    pub recommended_strategy: GasStrategy,
    pub expected_cost_eth: f64,
    pub cost_breakdown: GasCostBreakdown,
}

/// Gas cost breakdown for flash loan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasCostBreakdown {
    pub base_fee_cost_eth: f64,
    pub priority_fee_cost_eth: f64,
    pub l1_data_cost_eth: f64,
    pub l2_execution_cost_eth: f64,
    pub total_cost_eth: f64,
}

/// Gas strategy recommendation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GasStrategy {
    Standard,        // Normal priority fee
    Fast,            // Above average priority fee
    Turbo,           // High priority for urgent trades
    Wait,            // Delay execution for lower gas
    Skip,            // Gas too high, skip opportunity
}

impl GasStrategy {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Standard => "STANDARD",
            Self::Fast => "FAST",
            Self::Turbo => "TURBO",
            Self::Wait => "WAIT",
            Self::Skip => "SKIP",
        }
    }
}

/// Historical gas record for time series analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasHistoryRecord {
    pub base_fee_gwei: f64,
    pub priority_fee_gwei: f64,
    pub gas_used_ratio: f64,
    pub block_number: u64,
    pub timestamp_ms: u64,
}

/// Predictive Gas Model
#[derive(Debug, Clone)]
pub struct PredictiveGasModel {
    pub history: VecDeque<GasHistoryRecord>,
    pub l1_l2_model: L1L2GasModel,
    pub ewma_base_fee: f64,
    pub ewma_priority_fee: f64,
    pub ewma_volatility: f64,
    pub last_prediction: Option<GasPricePrediction>,
    pub prediction_accuracy: f64,
}

impl PredictiveGasModel {
    pub fn new() -> Self {
        Self {
            history: VecDeque::with_capacity(HISTORY_WINDOW_SIZE),
            l1_l2_model: L1L2GasModel::default(),
            ewma_base_fee: 20.0,
            ewma_priority_fee: 2.0,
            ewma_volatility: 0.1,
            last_prediction: None,
            prediction_accuracy: 0.85,
        }
    }

    /// Record new gas observation
    pub fn record_gas(&mut self, params: &GasParameters) {
        let record = GasHistoryRecord {
            base_fee_gwei: params.base_fee_gwei,
            priority_fee_gwei: params.priority_fee_gwei,
            gas_used_ratio: params.gas_used_ratio,
            block_number: params.block_number,
            timestamp_ms: params.timestamp_ms,
        };

        if self.history.len() >= HISTORY_WINDOW_SIZE {
            self.history.pop_front();
        }
        self.history.push_back(record);

        // Update EWMA
        let alpha = 0.1;
        self.ewma_base_fee = alpha * params.base_fee_gwei + (1.0 - alpha) * self.ewma_base_fee;
        self.ewma_priority_fee = alpha * params.priority_fee_gwei + (1.0 - alpha) * self.ewma_priority_fee;

        // Update volatility estimate
        if self.history.len() > 2 {
            let recent: Vec<f64> = self.history.iter().rev().take(10).map(|r| r.base_fee_gwei).collect();
            let mean = recent.iter().sum::<f64>() / recent.len() as f64;
            let variance = recent.iter().map(|&v| (v - mean).powi(2)).sum::<f64>() / recent.len() as f64;
            self.ewma_volatility = self.ewma_volatility * 0.9 + variance.sqrt() * 0.1;
        }
    }

    /// Predict gas prices for next N blocks
    pub fn predict(&mut self, horizon_blocks: usize) -> GasPricePrediction {
        if self.history.len() < 5 {
            return self.fallback_prediction(horizon_blocks);
        }

        // Extract recent base fees
        let recent_base_fees: Vec<f64> = self.history.iter().rev().take(20).map(|r| r.base_fee_gwei).collect();
        let recent_priority_fees: Vec<f64> = self.history.iter().rev().take(20).map(|r| r.priority_fee_gwei).collect();
        let recent_gas_ratios: Vec<f64> = self.history.iter().rev().take(20).map(|r| r.gas_used_ratio).collect();

        // EWMA-based prediction with trend adjustment
        let base_fee_trend = if recent_base_fees.len() >= 5 {
            let recent_5: f64 = recent_base_fees[0..5].iter().sum::<f64>() / 5.0;
            let older_5: f64 = recent_base_fees[5..10].iter().sum::<f64>() / 5.0;
            (recent_5 - older_5) / (older_5 + 1e-9)
        } else {
            0.0
        };

        let predicted_base = (self.ewma_base_fee * (1.0 + base_fee_trend * 0.5))
            .max(1.0)
            .min(1000.0);

        // Priority fee: bounded by base fee, with congestion adjustment
        let avg_gas_ratio = recent_gas_ratios.iter().sum::<f64>() / recent_gas_ratios.len() as f64;
        let congestion_multiplier = match avg_gas_ratio {
            r if r < 0.5 => 1.0,
            r if r < 0.7 => 1.5,
            r if r < 0.9 => 2.5,
            _ => 4.0,
        };
        let predicted_priority = (self.ewma_priority_fee * congestion_multiplier)
            .max(0.1)
            .min(predicted_base * 0.5);

        let predicted_max = predicted_base * 1.5 + predicted_priority;

        // Determine congestion state
        let congestion_state = match avg_gas_ratio {
            r if r < 0.5 => CongestionState::Low,
            r if r < 0.7 => CongestionState::Medium,
            r if r < 0.9 => CongestionState::High,
            _ => CongestionState::Extreme,
        };

        // Gas strategy
        let recommended_strategy = match congestion_state {
            CongestionState::Low => GasStrategy::Standard,
            CongestionState::Medium => GasStrategy::Fast,
            CongestionState::High => GasStrategy::Turbo,
            CongestionState::Extreme => GasStrategy::Skip,
        };

        // Cost breakdown for flash loan (approximate gas limit: 500,000 for complex arb)
        let gas_limit = 500_000u64;
        let l1_execution_cost = predicted_base + predicted_priority;
        let l1_data_cost = self.l1_l2_model.l1_calldata_cost_gwei * 0.5;
        let l2_cost = 0.0; // L1 execution
        
        let expected_cost_eth = (l1_execution_cost + l1_data_cost + l2_cost) * gas_limit as f64 * 1e-9;

        let confidence = (1.0 - self.ewma_volatility / (self.ewma_base_fee + 1e-9))
            .max(0.3)
            .min(0.99);

        let prediction = GasPricePrediction {
            predicted_base_fee_gwei: predicted_base,
            predicted_priority_fee_gwei: predicted_priority,
            predicted_max_fee_gwei: predicted_max,
            confidence,
            prediction_horizon_blocks: horizon_blocks,
            congestion_state,
            l1_l2_model: self.l1_l2_model.clone(),
            recommended_strategy,
            expected_cost_eth,
            cost_breakdown: GasCostBreakdown {
                base_fee_cost_eth: predicted_base * gas_limit as f64 * 1e-9,
                priority_fee_cost_eth: predicted_priority * gas_limit as f64 * 1e-9,
                l1_data_cost_eth: l1_data_cost * gas_limit as f64 * 1e-9,
                l2_execution_cost_eth: l2_cost * gas_limit as f64 * 1e-9,
                total_cost_eth: expected_cost_eth,
            },
        };

        self.last_prediction = Some(prediction.clone());
        self.prediction_accuracy = self.prediction_accuracy * 0.95 + confidence * 0.05;

        prediction
    }

    /// Fallback prediction when insufficient history
    fn fallback_prediction(&self, horizon_blocks: usize) -> GasPricePrediction {
        let predicted_base = self.ewma_base_fee;
        let predicted_priority = self.ewma_priority_fee;
        let predicted_max = predicted_base * 1.5 + predicted_priority;
        let gas_limit = 500_000u64;
        let expected_cost_eth = (predicted_base + predicted_priority) * gas_limit as f64 * 1e-9;

        GasPricePrediction {
            predicted_base_fee_gwei: predicted_base,
            predicted_priority_fee_gwei: predicted_priority,
            predicted_max_fee_gwei: predicted_max,
            confidence: 0.5,
            prediction_horizon_blocks: horizon_blocks,
            congestion_state: CongestionState::Medium,
            l1_l2_model: self.l1_l2_model.clone(),
            recommended_strategy: GasStrategy::Standard,
            expected_cost_eth,
            cost_breakdown: GasCostBreakdown {
                base_fee_cost_eth: predicted_base * gas_limit as f64 * 1e-9,
                priority_fee_cost_eth: predicted_priority * gas_limit as f64 * 1e-9,
                l1_data_cost_eth: 0.0,
                l2_execution_cost_eth: 0.0,
                total_cost_eth: expected_cost_eth,
            },
        }
    }

    /// Update L1/L2 model for a specific chain
    pub fn update_l1_l2_model(&mut self, model: L1L2GasModel) {
        self.l1_l2_model = model;
    }

    /// Get optimal gas parameters for a strategy
    pub fn get_optimal_gas_params(&self, strategy: GasStrategy, max_cost_eth: f64) -> (f64, f64, f64) {
        let base = self.ewma_base_fee;
        let multiplier = match strategy {
            GasStrategy::Turbo => 2.0,
            GasStrategy::Fast => 1.5,
            GasStrategy::Standard => 1.0,
            GasStrategy::Wait => 0.8,
            GasStrategy::Skip => return (0.0, 0.0, 0.0),
        };

        let priority = (self.ewma_priority_fee * multiplier).max(0.1);
        let max_fee = base * 1.5 + priority;

        // Cap at max cost
        let gas_limit = 500_000u64;
        let current_cost = (base + priority) * gas_limit as f64 * 1e-9;
        if current_cost > max_cost_eth {
            let scale = max_cost_eth / current_cost;
            return (base * scale, priority * scale, max_fee * scale);
        }

        (base, priority, max_fee)
    }

    /// Get prediction accuracy
    pub fn get_accuracy(&self) -> f64 {
        self.prediction_accuracy
    }

    /// Reset model (for chain switch)
    pub fn reset(&mut self) {
        self.history.clear();
        self.ewma_base_fee = 20.0;
        self.ewma_priority_fee = 2.0;
        self.ewma_volatility = 0.1;
        self.last_prediction = None;
    }
}

impl Default for PredictiveGasModel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gas_recording() {
        let mut model = PredictiveGasModel::new();
        let params = GasParameters {
            base_fee_gwei: 20.0,
            priority_fee_gwei: 2.0,
            max_fee_gwei: 30.0,
            block_number: 18000000,
            timestamp_ms: 1000000,
            gas_used_ratio: 0.6,
            burnt_fees_eth: 0.5,
        };
        model.record_gas(&params);
        assert_eq!(model.history.len(), 1);
    }

    #[test]
    fn test_gas_prediction() {
        let mut model = PredictiveGasModel::new();
        for i in 0..10 {
            let params = GasParameters {
                base_fee_gwei: 20.0 + i as f64,
                priority_fee_gwei: 2.0,
                max_fee_gwei: 30.0,
                block_number: 18000000 + i,
                timestamp_ms: 1000000 + i * 12000,
                gas_used_ratio: 0.6,
                burnt_fees_eth: 0.5,
            };
            model.record_gas(&params);
        }
        let prediction = model.predict(12);
        assert!(prediction.predicted_base_fee_gwei > 0.0);
        assert!(prediction.confidence > 0.0);
    }

    #[test]
    fn test_l1_l2_model() {
        let mut model = PredictiveGasModel::new();
        let l2_model = L1L2GasModel {
            l2_base_fee_gwei: 0.01,
            l2_priority_fee_gwei: 0.001,
            l2_gas_price_ratio: 0.001,
            ..Default::default()
        };
        model.update_l1_l2_model(l2_model);
        assert_eq!(model.l1_l2_model.l2_base_fee_gwei, 0.01);
    }

    #[test]
    fn test_gas_strategy() {
        let model = PredictiveGasModel::new();
        let (base, priority, max_fee) = model.get_optimal_gas_params(GasStrategy::Standard, 1.0);
        assert!(base > 0.0);
        assert!(priority > 0.0);
    }
}
