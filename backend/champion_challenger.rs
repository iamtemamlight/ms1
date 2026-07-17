// ==============================================================================
// MODULE: Champion/Challenger Framework
// Purpose: Implements A/B testing for strategy optimization
//          Enables automated winner propagation
// ==============================================================================

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Experiment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experiment {
    pub id: String,
    pub name: String,
    pub champion_config: StrategyConfig,
    pub challenger_config: StrategyConfig,
    pub traffic_split: f64,  // 0.0-1.0 = % to challenger
    pub status: ExperimentStatus,
    pub start_time: u64,
    pub end_time: Option<u64>,
    pub winner: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExperimentStatus {
    Running,
    Completed,
    Paused,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfig {
    pub risk_mode: u64,
    pub min_profit_threshold: f64,
    pub max_slippage: f64,
    pub jitter_ms: f64,
    pub parameters: Vec<(String, f64)>,
}

impl StrategyConfig {
    pub fn default_champion() -> Self {
        Self {
            risk_mode: 1,
            min_profit_threshold: 0.01,
            max_slippage: 0.001,
            jitter_ms: 1.0,
            parameters: vec![],
        }
    }
}

/// Champion/Challenger Manager
pub struct ChampionChallengerManager {
    experiments: VecDeque<Experiment>,
    max_concurrent: usize,
}

impl ChampionChallengerManager {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            experiments: VecDeque::new(),
            max_concurrent,
        }
    }

    /// Create new experiment
    pub fn create_experiment(
        &mut self,
        id: String,
        name: String,
        challenger_config: StrategyConfig,
        traffic_split: f64,
    ) -> Result<&Experiment, String> {
        if self.experiments.len() >= self.max_concurrent {
            return Err("Max concurrent experiments reached".to_string());
        }

        let experiment = Experiment {
            id: id.clone(),
            name,
            champion_config: StrategyConfig::default_champion(),
            challenger_config,
            traffic_split,
            status: ExperimentStatus::Running,
            start_time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            end_time: None,
            winner: None,
        };

        self.experiments.push_back(experiment);
self.experiments.back().ok_or_else(|| "Failed to create experiment".to_string())
    }

    /// Record experiment result
    pub fn record_result(&mut self, experiment_id: &str, winner: String) {
        for exp in &mut self.experiments {
            if exp.id == experiment_id {
                exp.winner = Some(winner);
                exp.status = ExperimentStatus::Completed;
                exp.end_time = Some(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                );
                break;
            }
        }
    }

    /// Get active experiments
    pub fn get_active(&self) -> Vec<&Experiment> {
        self.experiments
            .iter()
            .filter(|e| e.status == ExperimentStatus::Running)
            .collect()
    }

    /// Get winner configuration for an experiment
    pub fn get_winner_config(&self, experiment_id: &str) -> Option<StrategyConfig> {
        self.experiments
            .iter()
            .find(|e| e.id == experiment_id && e.status == ExperimentStatus::Completed)
            .and_then(|e| {
                e.winner.as_ref().map(|w| {
                    if w == "challenger" {
                        e.challenger_config.clone()
                    } else {
                        e.champion_config.clone()
                    }
                })
            })
    }
}

impl Default for ChampionChallengerManager {
    fn default() -> Self {
        Self::new(5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_experiment() {
        let mut manager = ChampionChallengerManager::new(5);
        let challenger = StrategyConfig {
            risk_mode: 2,
            min_profit_threshold: 0.02,
            max_slippage: 0.002,
            jitter_ms: 2.0,
            parameters: vec![],
        };
        
        let result = manager.create_experiment(
            "exp_001".to_string(),
            "Test Experiment".to_string(),
            challenger,
            0.1,
        );
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_record_result() {
        let mut manager = ChampionChallengerManager::new(5);
        
        let challenger = StrategyConfig {
            risk_mode: 2,
            min_profit_threshold: 0.02,
            max_slippage: 0.002,
            jitter_ms: 2.0,
            parameters: vec![],
        };
        
        let _ = manager.create_experiment(
            "exp_001".to_string(),
            "Test".to_string(),
            challenger,
            0.1,
        );
        
        manager.record_result("exp_001", "challenger".to_string());
        
        // Find the experiment and check winner
        let exp = manager.experiments.iter().find(|e| e.id == "exp_001");
        assert!(exp.is_some());
    }
}
