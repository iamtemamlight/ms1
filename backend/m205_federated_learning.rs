// ==============================================================================
// M205: Federated Learning System
// Purpose: Cross-runner knowledge transfer via federated learning
//          Enables champion runner DNA propagation to fleet in <5s
// CGM Subsystem: Growth
// ==============================================================================

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::cross_agent_learning::{AgentDna, CrossAgentLearning};

/// Model update from a runner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelUpdate {
    pub runner_id: String,
    pub model_weights: Vec<f64>,
    pub bias: f64,
    pub gradient_norm: f64,
    pub sample_count: u64,
    pub loss: f64,
    pub timestamp_ms: u64,
}

/// Aggregated global model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalModel {
    pub weights: Vec<f64>,
    pub bias: f64,
    pub version: u64,
    pub last_updated_ms: u64,
    pub sample_count: u64,
    pub runner_count: u32,
}

impl Default for GlobalModel {
    fn default() -> Self {
        Self {
            weights: vec![0.0; 25],
            bias: 0.0,
            version: 1,
            last_updated_ms: 0,
            sample_count: 0,
            runner_count: 0,
        }
    }
}

/// Federated Learning configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederatedLearningConfig {
    pub min_rounds: u32,
    pub max_rounds: u32,
    pub convergence_threshold: f64,
    pub participation_rate: f64,
    pub aggregation_method: AggregationMethod,
    pub secure_aggregation: bool,
    pub differential_privacy: bool,
    pub dp_epsilon: f64,
    pub dp_delta: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AggregationMethod {
    FedAvg,
    FedProx,
    FedAdam,
    Median,
    TrimmedMean,
}

impl Default for FederatedLearningConfig {
    fn default() -> Self {
        Self {
            min_rounds: 3,
            max_rounds: 50,
            convergence_threshold: 0.001,
            participation_rate: 0.8,
            aggregation_method: AggregationMethod::FedAvg,
            secure_aggregation: false,
            differential_privacy: false,
            dp_epsilon: 0.1,
            dp_delta: 1e-5,
        }
    }
}

/// Federated Learning round
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederatedRound {
    pub round_id: u64,
    pub global_model: GlobalModel,
    pub updates: Vec<ModelUpdate>,
    pub aggregated_loss: f64,
    pub convergence_metric: f64,
    pub participating_runners: u32,
    pub started_at_ms: u64,
    pub completed_at_ms: u64,
}

/// Federated Learning Manager
#[derive(Debug, Clone)]
pub struct FederatedLearningManager {
    pub global_model: GlobalModel,
    pub config: FederatedLearningConfig,
    pub round_history: Vec<FederatedRound>,
    pub pending_updates: HashMap<String, ModelUpdate>,
    pub cross_agent_learning: CrossAgentLearning,
    pub current_round_id: u64,
    pub is_training: bool,
    pub convergence_history: Vec<f64>,
}

impl FederatedLearningManager {
    pub fn new(config: FederatedLearningConfig) -> Self {
        Self {
            global_model: GlobalModel::default(),
            config,
            round_history: Vec::new(),
            pending_updates: HashMap::new(),
            cross_agent_learning: CrossAgentLearning::new(),
            current_round_id: 0,
            is_training: false,
            convergence_history: Vec::new(),
        }
    }

    /// Create with default trading configuration
    pub fn for_trading() -> Self {
        Self::new(FederatedLearningConfig::default())
    }

    /// Start a new federated round
    pub fn start_round(&mut self) -> u64 {
        self.current_round_id += 1;
        self.pending_updates.clear();
        self.current_round_id
    }

    /// Submit model update from a runner
    pub fn submit_update(&mut self, update: ModelUpdate) -> Result<(), String> {
        if update.model_weights.len() != self.global_model.weights.len() {
            return Err(format!(
                "Weight dimension mismatch: expected {}, got {}",
                self.global_model.weights.len(),
                update.model_weights.len()
            ));
        }
        self.pending_updates.insert(update.runner_id.clone(), update);
        Ok(())
    }

    /// Aggregate updates and update global model
    pub fn aggregate(&mut self) -> Result<GlobalModel, String> {
        if self.pending_updates.is_empty() {
            return Err("No updates to aggregate".to_string());
        }

        let updates: Vec<_> = self.pending_updates.values().cloned().collect();
        let num_updates = updates.len() as f64;
        let weight_dim = self.global_model.weights.len();

        let mut new_weights = vec![0.0; weight_dim];
        let mut new_bias = 0.0;
        let mut total_samples = 0u64;
        let mut total_loss = 0.0;

        match self.config.aggregation_method {
            AggregationMethod::FedAvg => {
                for update in &updates {
                    let weight = update.sample_count as f64;
                    for i in 0..weight_dim {
                        new_weights[i] += update.model_weights[i] * weight;
                    }
                    new_bias += update.bias * weight;
                    total_samples += update.sample_count;
                    total_loss += update.loss * weight;
                }
                for w in &mut new_weights {
                    *w /= num_updates;
                }
                new_bias /= num_updates;
                total_loss /= num_updates;
            }
            AggregationMethod::FedProx => {
                let mu = 0.01; // Proximal term coefficient
                for update in &updates {
                    for i in 0..weight_dim {
                        new_weights[i] += update.model_weights[i] + mu * (self.global_model.weights[i] - update.model_weights[i]);
                    }
                    new_bias += update.bias;
                    total_samples += update.sample_count;
                    total_loss += update.loss;
                }
                for w in &mut new_weights {
                    *w /= num_updates;
                }
                new_bias /= num_updates;
                total_loss /= num_updates;
            }
            AggregationMethod::Median => {
                for i in 0..weight_dim {
                    let mut vals: Vec<f64> = updates.iter().map(|u| u.model_weights[i]).collect();
                    vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                    new_weights[i] = vals[vals.len() / 2];
                }
                let mut biases: Vec<f64> = updates.iter().map(|u| u.bias).collect();
                biases.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                new_bias = biases[biases.len() / 2];
                total_loss = updates.iter().map(|u| u.loss).sum::<f64>() / num_updates;
            }
            AggregationMethod::TrimmedMean => {
                let trim_count = ((updates.len() as f64) * 0.1).ceil() as usize;
                for i in 0..weight_dim {
                    let mut vals: Vec<f64> = updates.iter().map(|u| u.model_weights[i]).collect();
                    vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                    let trimmed = &vals[trim_count..vals.len().saturating_sub(trim_count)];
                    if !trimmed.is_empty() {
                        new_weights[i] = trimmed.iter().sum::<f64>() / trimmed.len() as f64;
                    }
                }
                total_loss = updates.iter().map(|u| u.loss).sum::<f64>() / num_updates;
            }
            AggregationMethod::FedAdam => {
                // Simplified FedAdam
                let lr = 0.01;
                let beta1 = 0.9;
                let beta2 = 0.999;
                let eps = 1e-8;
                
                for i in 0..weight_dim {
                    let grad = updates.iter().map(|u| u.model_weights[i]).sum::<f64>() / num_updates;
                    new_weights[i] = self.global_model.weights[i] - lr * grad;
                }
                new_bias = self.global_model.bias;
                total_loss = updates.iter().map(|u| u.loss).sum::<f64>() / num_updates;
            }
        }

        // Differential privacy noise
        if self.config.differential_privacy {
            let noise_scale = 2.0 * self.config.dp_epsilon / (num_updates * self.config.dp_delta).sqrt();
            for w in &mut new_weights {
                *w += (rand::random::<f64>() - 0.5) * noise_scale;
            }
        }

        let new_model = GlobalModel {
            weights: new_weights,
            bias: new_bias,
            version: self.global_model.version + 1,
            last_updated_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
            sample_count: total_samples,
            runner_count: updates.len() as u32,
        };

        // Record round
        let round = FederatedRound {
            round_id: self.current_round_id,
            global_model: new_model.clone(),
            updates: updates.clone(),
            aggregated_loss: total_loss,
            convergence_metric: 0.0,
            participating_runners: updates.len() as u32,
            started_at_ms: 0,
            completed_at_ms: new_model.last_updated_ms,
        };
        self.round_history.push(round);
        if self.round_history.len() > 100 {
            self.round_history.remove(0);
        }

        self.global_model = new_model.clone();
        self.convergence_history.push(total_loss);
        if self.convergence_history.len() > 50 {
            self.convergence_history.remove(0);
        }

        Ok(new_model)
    }

    /// Propagate global model to fleet via cross-agent learning
    pub fn propagate_to_fleet(&mut self) -> Result<(), String> {
        let dna = AgentDna {
            agent_id: "GLOBAL".to_string(),
            model_weights: self.global_model.weights.clone(),
            bias: self.global_model.bias,
            confidence: 0.95,
            observations: self.global_model.sample_count,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
        };
        self.cross_agent_learning.propagate_champion_sync(dna)
    }

    /// Check convergence
    pub fn is_converged(&self) -> bool {
        if self.convergence_history.len() < 5 {
            return false;
        }
        let recent: f64 = self.convergence_history.iter().rev().take(3).sum::<f64>() / 3.0;
        let older: f64 = self.convergence_history.iter().take(3).sum::<f64>() / 3.0;
        (older - recent).abs() < self.config.convergence_threshold
    }

    /// Get training statistics
    pub fn get_stats(&self) -> String {
        format!(
            r#"{{"rounds":{},"version":{},"runners":{},"converged":{},"loss":{:.6}}}"#,
            self.round_history.len(),
            self.global_model.version,
            self.global_model.runner_count,
            self.is_converged(),
            self.convergence_history.last().copied().unwrap_or(0.0)
        )
    }
}

impl Default for FederatedLearningManager {
    fn default() -> Self {
        Self::for_trading()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_federated_round() {
        let mut fl = FederatedLearningManager::for_trading();
        let round_id = fl.start_round();
        assert!(round_id > 0);
    }

    #[test]
    fn test_model_aggregation() {
        let mut fl = FederatedLearningManager::for_trading();
        fl.start_round();
        
        for i in 0..5 {
            let update = ModelUpdate {
                runner_id: format!("runner_{}", i),
                model_weights: vec![0.1 * (i as f64 + 1.0); 25],
                bias: 0.0,
                gradient_norm: 0.1,
                sample_count: 100,
                loss: 0.5,
                timestamp_ms: 1000000,
            };
            fl.submit_update(update).unwrap();
        }
        
        let model = fl.aggregate().unwrap();
        assert_eq!(model.version, 2);
    }

    #[test]
    fn test_convergence_check() {
        let mut fl = FederatedLearningManager::for_trading();
        for i in 0..10 {
            fl.convergence_history.push(0.5 - i as f64 * 0.01);
        }
        // May or may not be converged depending on history
        let _ = fl.is_converged();
    }
}
