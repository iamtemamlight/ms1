use crate::GlobalFleetState;
use crate::error::DomainError;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct LearningEngine {
    pub confidence: f64,
    pub observations: u64,
    pub pattern_library: HashMap<String, Pattern>,
    pub prediction_model: PredictionModel,
}

#[derive(Debug, Clone)]
pub struct Pattern {
    pub signature: Vec<f64>,
    pub occurrences: u64,
    pub success_rate: f64,
    pub last_seen: u64,
}

#[derive(Debug, Default)]
pub struct PredictionModel {
    pub weights: Vec<f64>,
    pub bias: f64,
    pub last_trained: u64,
}

#[derive(Debug, Clone)]
pub struct PatternResult {
    pub pattern_id: String,
    pub confidence: f64,
    pub expected_outcome: f64,
}

#[derive(Debug, Clone)]
pub struct PredictionResult {
    pub value: f64,
    pub lower_bound: f64,
    pub upper_bound: f64,
    pub confidence: f64,
}

impl LearningEngine {
    pub fn new() -> Self {
        Self {
            confidence: 0.85,
            observations: 0,
            pattern_library: HashMap::new(),
            prediction_model: PredictionModel::default(),
        }
    }

    pub fn observe_fleet_state(&mut self, state: &GlobalFleetState) {
        let _ = state;
        self.observations += 1;
        if self.observations % 100 == 0 {
            self.confidence = (self.confidence + 0.001).min(1.0);
        }
    }

    /// M068: Pattern Recognition with confidence scoring and rule-based fallback
    pub fn detect_pattern(&mut self, features: &[f64]) -> Result<PatternResult, DomainError> {
        if features.is_empty() {
            return Err(DomainError::InvalidInput {
                message: "Empty feature vector".to_string()
            });
        }

        let best_match = self.find_best_pattern_match(features);
        let confidence = best_match.map(|p| p.success_rate).unwrap_or(0.0);

        if confidence < 0.7 {
            let fallback = self.rule_based_detection(features);
            return Ok(fallback);
        }

        Ok(PatternResult {
            pattern_id: best_match.unwrap().signature.iter()
                .map(|v| format!("{:.2}", v))
                .collect::<Vec<_>>()
                .join("_"),
            confidence,
            expected_outcome: confidence * self.confidence,
        })
    }

    /// M071: Model Prediction with confidence bounds
    pub fn predict_with_confidence(&self, input: &[f64]) -> Result<PredictionResult, DomainError> {
        if input.len() != self.prediction_model.weights.len() {
            return Err(DomainError::InvalidInput {
                message: format!("Input dimension mismatch: expected {}, got {}", 
                    self.prediction_model.weights.len(), input.len())
            });
        }

        let raw_prediction = self.forward_pass(input);
        let uncertainty = self.estimate_uncertainty(input);
        let confidence = self.confidence * (1.0 - uncertainty);

        Ok(PredictionResult {
            value: raw_prediction,
            lower_bound: raw_prediction - uncertainty,
            upper_bound: raw_prediction + uncertainty,
            confidence,
        })
    }

    /// Update pattern library with new observation
    pub fn learn_pattern(&mut self, features: &[f64], outcome: f64) {
        let signature_hash = self.hash_features(features);
        let pattern = self.pattern_library.entry(signature_hash).or_insert(Pattern {
            signature: features.to_vec(),
            occurrences: 0,
            success_rate: 0.0,
            last_seen: 0,
        });

        pattern.occurrences += 1;
        pattern.success_rate = (pattern.success_rate * (pattern.occurrences - 1) as f64 + outcome) 
            / pattern.occurrences as f64;
        pattern.last_seen = self.observations;
    }

    /// Train prediction model on labeled dataset
    pub fn train_model(&mut self, inputs: &[Vec<f64>], targets: &[f64]) -> Result<(), DomainError> {
        if inputs.len() != targets.len() || inputs.is_empty() {
            return Err(DomainError::InvalidInput {
                message: "Training data dimension mismatch".to_string()
            });
        }

        let feature_dim = inputs[0].len();
        self.prediction_model.weights = vec![0.01; feature_dim];
        self.prediction_model.bias = 0.0;

        let learning_rate = 0.001;
        let epochs = 10;

        for _ in 0..epochs {
            for (x, y) in inputs.iter().zip(targets.iter()) {
                let prediction = self.forward_pass(x);
                let error = y - prediction;
                
                for (w, xi) in self.prediction_model.weights.iter_mut().zip(x.iter()) {
                    *w += learning_rate * error * xi;
                }
                self.prediction_model.bias += learning_rate * error;
            }
        }

        self.prediction_model.last_trained = self.observations;
        Ok(())
    }

    fn find_best_pattern_match(&self, features: &[f64]) -> Option<&Pattern> {
        self.pattern_library.values()
            .filter(|p| p.signature.len() == features.len())
            .max_by(|a, b| {
                let dist_a: f64 = a.signature.iter().zip(features).map(|(a, b)| (a - b).powi(2)).sum();
                let dist_b: f64 = b.signature.iter().zip(features).map(|(a, b)| (a - b).powi(2)).sum();
                dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    fn rule_based_detection(&self, features: &[f64]) -> PatternResult {
        let mean: f64 = features.iter().sum::<f64>() / features.len() as f64;
        let variance: f64 = features.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / features.len() as f64;

        let pattern_id = format!("rule_based_{:.2}", mean);
        PatternResult {
            pattern_id,
            confidence: (1.0 - variance.sqrt()).max(0.0),
            expected_outcome: mean,
        }
    }

    fn forward_pass(&self, input: &[f64]) -> f64 {
        input.iter()
            .zip(&self.prediction_model.weights)
            .map(|(x, w)| x * w)
            .sum::<f64>() + self.prediction_model.bias
    }

    fn estimate_uncertainty(&self, input: &[f64]) -> f64 {
        let feature_norm: f64 = input.iter().map(|x| x.powi(2)).sum::<f64>().sqrt();
        (feature_norm * 0.1).min(0.5)
    }

    fn hash_features(&self, features: &[f64]) -> String {
        features.iter()
            .map(|v| format!("{:.3}", v))
            .collect::<Vec<_>>()
            .join("|")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_detection() {
        let mut engine = LearningEngine::new();
        let features = vec![1.0, 2.0, 3.0];
        
        let result = engine.detect_pattern(&features).unwrap();
        assert!(result.confidence >= 0.0 && result.confidence <= 1.0);
    }

    #[test]
    fn test_prediction_with_confidence() {
        let engine = LearningEngine::new();
        let input = vec![1.0, 2.0];
        let result = engine.predict_with_confidence(&input).unwrap();
        assert!(result.lower_bound <= result.value);
        assert!(result.value <= result.upper_bound);
    }

    #[test]
    fn test_rule_based_fallback() {
        let engine = LearningEngine::new();
        let features = vec![5.0, 5.0, 5.0];
        let result = engine.rule_based_detection(&features);
        assert!(result.confidence > 0.0);
    }

    #[test]
    fn test_model_training() {
        let mut engine = LearningEngine::new();
        let inputs = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
        let targets = vec![1.0, 0.0];
        
        let result = engine.train_model(&inputs, &targets);
        assert!(result.is_ok());
    }
}
