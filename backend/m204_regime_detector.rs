// ==============================================================================
// M204: Market Regime Detector
// Purpose: Hidden Markov Model-based market regime classification
//          Detects Bull/Bear/Sideways/Volatile/Crash/Recovery states
// CGM Subsystem: Profit / Growth
// ==============================================================================

use std::collections::{HashMap, VecDeque};
use std::f64::NEG_INFINITY;
use serde::{Deserialize, Serialize};

/// Market regime states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MarketRegime {
    Bull,
    Bear,
    Sideways,
    Volatile,
    Crash,
    Recovery,
}

impl MarketRegime {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Bull => "BULL",
            Self::Bear => "BEAR",
            Self::Sideways => "SIDEWAYS",
            Self::Volatile => "VOLATILE",
            Self::Crash => "CRASH",
            Self::Recovery => "RECOVERY",
        }
    }

    /// Get optimization weight profile for this regime
    pub fn optimization_weights(&self) -> RegimeWeights {
        match self {
            Self::Bull => RegimeWeights {
                exploration: 0.3,
                exploitation: 0.7,
                risk_tolerance: 0.7,
                position_size_factor: 1.2,
                gas_strategy_bias: 1.1,
            },
            Self::Bear => RegimeWeights {
                exploration: 0.4,
                exploitation: 0.6,
                risk_tolerance: 0.3,
                position_size_factor: 0.6,
                gas_strategy_bias: 0.9,
            },
            Self::Sideways => RegimeWeights {
                exploration: 0.5,
                exploitation: 0.5,
                risk_tolerance: 0.5,
                position_size_factor: 0.9,
                gas_strategy_bias: 1.0,
            },
            Self::Volatile => RegimeWeights {
                exploration: 0.6,
                exploitation: 0.4,
                risk_tolerance: 0.4,
                position_size_factor: 0.7,
                gas_strategy_bias: 1.3,
            },
            Self::Crash => RegimeWeights {
                exploration: 0.7,
                exploitation: 0.3,
                risk_tolerance: 0.1,
                position_size_factor: 0.3,
                gas_strategy_bias: 0.8,
            },
            Self::Recovery => RegimeWeights {
                exploration: 0.4,
                exploitation: 0.6,
                risk_tolerance: 0.5,
                position_size_factor: 1.0,
                gas_strategy_bias: 1.0,
            },
        }
    }
}

/// Regime-specific optimization weights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegimeWeights {
    pub exploration: f64,
    pub exploitation: f64,
    pub risk_tolerance: f64,
    pub position_size_factor: f64,
    pub gas_strategy_bias: f64,
}

impl Default for RegimeWeights {
    fn default() -> Self {
        Self {
            exploration: 0.5,
            exploitation: 0.5,
            risk_tolerance: 0.5,
            position_size_factor: 1.0,
            gas_strategy_bias: 1.0,
        }
    }
}

/// Market features for regime detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketFeatures {
    pub price_return_1h: f64,
    pub price_return_24h: f64,
    pub volatility_1h: f64,
    pub volatility_24h: f64,
    pub volume_ratio: f64,
    pub gas_price_trend: f64,
    pub arbitrage_opportunity_density: f64,
    pub mev_activity_level: f64,
    pub liquidity_depth_change: f64,
    pub timestamp_ms: u64,
}

impl MarketFeatures {
    pub fn to_vector(&self) -> Vec<f64> {
        vec![
            self.price_return_1h,
            self.price_return_24h,
            self.volatility_1h,
            self.volatility_24h,
            self.volume_ratio,
            self.gas_price_trend,
            self.arbitrage_opportunity_density,
            self.mev_activity_level,
            self.liquidity_depth_change,
        ]
    }
}

/// HMM state transition probabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionMatrix {
    pub states: Vec<MarketRegime>,
    pub probabilities: Vec<Vec<f64>>,
}

impl TransitionMatrix {
    pub fn new(states: Vec<MarketRegime>) -> Self {
        let n = states.len();
        let uniform_prob = 1.0 / n as f64;
        Self {
            states,
            probabilities: vec![vec![uniform_prob; n]; n],
        }
    }

    /// Get transition probability from state i to state j
    pub fn get(&self, from: MarketRegime, to: MarketRegime) -> f64 {
        let i = self.states.iter().position(|&s| s == from).unwrap_or(0);
        let j = self.states.iter().position(|&s| s == to).unwrap_or(0);
        self.probabilities[i][j]
    }

    /// Update transition probabilities
    pub fn update_transition(&mut self, from: MarketRegime, to: MarketRegime, learning_rate: f64) {
        let i = self.states.iter().position(|&s| s == from).unwrap_or(0);
        let j = self.states.iter().position(|&s| s == to).unwrap_or(0);
        
        for k in 0..self.probabilities[i].len() {
            if k == j {
                self.probabilities[i][k] += learning_rate * (1.0 - self.probabilities[i][k]);
            } else {
                self.probabilities[i][k] *= (1.0 - learning_rate);
            }
        }
        
        // Normalize rows
        for row in &mut self.probabilities {
            let sum: f64 = row.iter().sum();
            if sum > 0.0 {
                for val in row.iter_mut() {
                    *val /= sum;
                }
            }
        }
    }
}

impl Default for TransitionMatrix {
    fn default() -> Self {
        let states = vec![
            MarketRegime::Bull,
            MarketRegime::Bear,
            MarketRegime::Sideways,
            MarketRegime::Volatile,
            MarketRegime::Crash,
            MarketRegime::Recovery,
        ];
        Self::new(states)
    }
}

/// Gaussian emission model for each state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmissionModel {
    pub means: Vec<Vec<f64>>, // [state][feature]
    pub variances: Vec<Vec<f64>>, // [state][feature]
}

impl EmissionModel {
    pub fn new(num_states: usize, num_features: usize) -> Self {
        Self {
            means: vec![vec![0.0; num_features]; num_states],
            variances: vec![vec![1.0; num_features]; num_states],
        }
    }

    /// Compute log-likelihood of observation given state
    pub fn log_likelihood(&self, state: usize, features: &[f64]) -> f64 {
        let mut log_likelihood = 0.0;
        for (i, &feature) in features.iter().enumerate() {
            let mean = self.means[state][i];
            let variance = self.variances[state][i].max(1e-6);
            let diff = feature - mean;
            log_likelihood += -0.5 * ((diff * diff) / variance + (2.0 * std::f64::consts::PI * variance).ln());
        }
        log_likelihood
    }

    /// Update emission model with new observation
    pub fn update(&mut self, state: usize, features: &[f64], learning_rate: f64) {
        for (i, &feature) in features.iter().enumerate() {
            let error = feature - self.means[state][i];
            self.means[state][i] += learning_rate * error;
            self.variances[state][i] = self.variances[state][i] * (1.0 - learning_rate) + learning_rate * error * error;
            self.variances[state][i] = self.variances[state][i].max(1e-6);
        }
    }
}

impl Default for EmissionModel {
    fn default() -> Self {
        Self::new(6, 9)
    }
}

/// Regime detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegimeDetectionResult {
    pub current_regime: MarketRegime,
    pub regime_probabilities: HashMap<MarketRegime, f64>,
    pub confidence: f64,
    pub predicted_next_regime: MarketRegime,
    pub regime_duration_blocks: u64,
    pub regime_change_probability: f64,
    pub recommended_weights: RegimeWeights,
}

/// Hidden Markov Model for regime detection
#[derive(Debug, Clone)]
pub struct RegimeDetector {
    pub states: Vec<MarketRegime>,
    pub initial_probabilities: Vec<f64>,
    pub transition_matrix: TransitionMatrix,
    pub emission_model: EmissionModel,
    pub feature_history: VecDeque<MarketFeatures>,
    pub regime_history: VecDeque<MarketRegime>,
    pub last_result: Option<RegimeDetectionResult>,
    pub current_regime: MarketRegime,
    pub regime_start_block: u64,
    pub max_history: usize,
}

impl RegimeDetector {
    pub fn new() -> Self {
        let states = vec![
            MarketRegime::Bull,
            MarketRegime::Bear,
            MarketRegime::Sideways,
            MarketRegime::Volatile,
            MarketRegime::Crash,
            MarketRegime::Recovery,
        ];
        let n = states.len();
        Self {
            states: states.clone(),
            initial_probabilities: vec![1.0 / n as f64; n],
            transition_matrix: TransitionMatrix::new(states.clone()),
            emission_model: EmissionModel::new(n, 9),
            feature_history: VecDeque::with_capacity(200),
            regime_history: VecDeque::with_capacity(200),
            last_result: None,
            current_regime: MarketRegime::Sideways,
            regime_start_block: 0,
            max_history: 200,
        }
    }

    /// Add market observation and detect regime
    pub fn observe(&mut self, features: MarketFeatures, current_block: u64) -> RegimeDetectionResult {
        if self.feature_history.len() >= self.max_history {
            self.feature_history.pop_front();
        }
        self.feature_history.push_back(features.clone());

        // Viterbi-like forward pass for most likely state sequence
        let feature_vec = features.to_vector();
        let result = self.forward_pass(&feature_vec);

        // Update emission model
        let state_idx = self.states.iter().position(|&s| s == result.current_regime).unwrap_or(0);
        self.emission_model.update(state_idx, &feature_vec, 0.05);

        // Update transition matrix
        if let Some(&prev_regime) = self.regime_history.back() {
            self.transition_matrix.update_transition(prev_regime, result.current_regime, 0.03);
        }

        // Record regime
        self.regime_history.push_back(result.current_regime);
        if self.regime_history.len() >= self.max_history {
            self.regime_history.pop_front();
        }

        // Check for regime change
        let regime_changed = self.current_regime != result.current_regime;
        if regime_changed {
            self.current_regime = result.current_regime;
            self.regime_start_block = current_block;
        }

        self.last_result = Some(result.clone());
        result
    }

    /// Forward pass to compute most likely state
    fn forward_pass(&self, features: &[f64]) -> RegimeDetectionResult {
        let n = self.states.len();
        let mut log_probs = Vec::with_capacity(n);
        let mut state_probs = HashMap::new();

        // Compute log-likelihood for each state
        for (i, _) in self.states.iter().enumerate() {
            let emission_ll = self.emission_model.log_likelihood(i, features);
            let prior_ll = self.initial_probabilities[i].ln();
            log_probs.push(prior_ll + emission_ll);
            state_probs.insert(self.states[i], emission_ll.exp());
        }

        // Softmax normalization
        let max_log_prob = log_probs.iter().copied().fold(NEG_INFINITY, f64::max);
        let mut sum_exp = 0.0;
        for &lp in &log_probs {
            sum_exp += (lp - max_log_prob).exp();
        }
        for (i, lp) in log_probs.iter_mut().enumerate() {
            *lp = (*lp - max_log_prob) - sum_exp.ln();
            state_probs.insert(self.states[i], lp.exp());
        }

        // Find most likely state
        let best_idx = log_probs.iter().enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
            .unwrap_or(0);
        
        let current_regime = self.states[best_idx];
        let confidence = state_probs.get(&current_regime).copied().unwrap_or(0.5);

        // Predict next regime based on transition matrix
        let mut next_probs = Vec::with_capacity(n);
        for (j, &to_state) in self.states.iter().enumerate() {
            let mut prob = 0.0;
            for (i, &from_state) in self.states.iter().enumerate() {
                prob += state_probs.get(&from_state).copied().unwrap_or(0.0) * self.transition_matrix.get(from_state, to_state);
            }
            next_probs.push((to_state, prob));
        }
        next_probs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let predicted_next = next_probs.first().map(|&(s, _)| s).unwrap_or(current_regime);

        // Regime duration
        let regime_duration = if self.current_regime == current_regime {
            self.regime_history.iter().rev()
                .take_while(|&&s| s == current_regime)
                .count() as u64
        } else {
            1
        };

        // Regime change probability
        let regime_change_prob = 1.0 - self.transition_matrix.get(current_regime, current_regime);

        let recommended_weights = current_regime.optimization_weights();

        RegimeDetectionResult {
            current_regime,
            regime_probabilities: state_probs,
            confidence: confidence.min(0.99),
            predicted_next_regime: predicted_next,
            regime_duration_blocks: regime_duration,
            regime_change_probability: regime_change_prob,
            recommended_weights,
        }
    }

    /// Get current regime without adding new observation
    pub fn get_current_regime(&self) -> MarketRegime {
        self.current_regime
    }

    /// Get regime statistics
    pub fn get_regime_stats(&self) -> HashMap<MarketRegime, usize> {
        let mut stats = HashMap::new();
        for &regime in &self.regime_history {
            *stats.entry(regime).or_insert(0) += 1;
        }
        stats
    }

    /// Reset detector (for backtesting)
    pub fn reset(&mut self) {
        self.feature_history.clear();
        self.regime_history.clear();
        self.current_regime = MarketRegime::Sideways;
        self.regime_start_block = 0;
        self.last_result = None;
        self.transition_matrix = TransitionMatrix::default();
        self.emission_model = EmissionModel::default();
    }
}

impl Default for RegimeDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regime_detection() {
        let mut detector = RegimeDetector::new();
        let features = MarketFeatures {
            price_return_1h: 0.02,
            price_return_24h: 0.05,
            volatility_1h: 0.01,
            volatility_24h: 0.02,
            volume_ratio: 1.5,
            gas_price_trend: 0.1,
            arbitrage_opportunity_density: 0.8,
            mev_activity_level: 0.6,
            liquidity_depth_change: 0.05,
            timestamp_ms: 1000000,
        };
        let result = detector.observe(features, 18000000);
        assert!(result.confidence > 0.0);
    }

    #[test]
    fn test_regime_weights() {
        let weights = MarketRegime::Volatile.optimization_weights();
        assert!(weights.exploration > 0.0);
        assert!(weights.risk_tolerance < 0.5);
    }

    #[test]
    fn test_transition_matrix() {
        let mut tm = TransitionMatrix::default();
        tm.update_transition(MarketRegime::Bull, MarketRegime::Bear, 0.1);
        let prob = tm.get(MarketRegime::Bull, MarketRegime::Bear);
        assert!(prob > 0.0);
    }
}
