// ==============================================================================
// M056: Learning Engine
// Purpose: Learning Engine - Growth optimization and management
// CGM Subsystem: Growth
// ==============================================================================

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ModuleMetrics {
    pub executions: u64,
    pub successes: u64,
    pub failures: u64,
    pub last_execution: Option<String>,
    pub average_latency_ms: f64,
}

#[derive(Debug, Clone)]
pub struct ModuleResult {
    pub success: bool,
    pub message: String,
    pub data: HashMap<String, String>,
    pub execution_time_ms: u64,
    pub learned_patterns: Vec<LearnedPattern>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LearnedPattern {
    pub pattern_id: String,
    pub pattern_type: String,  // "profitability", "risk", "latency", "slippage"
    pub confidence: f64,
    pub features: HashMap<String, f64>,
    pub outcome: f64,
    pub observed_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TrainingExample {
    pub features: HashMap<String, f64>,
    pub label: f64,
    pub weight: f64,
}

#[derive(Debug)]
pub struct M56 {
    pub enabled: bool,
    pub metrics: ModuleMetrics,
    pub config: HashMap<String, String>,
    pub learned_patterns: Vec<LearnedPattern>,
    pub training_buffer: Vec<TrainingExample>,
    pub model_version: u64,
}

impl M56 {
    pub fn new() -> Self {
        Self {
            enabled: true,
            metrics: ModuleMetrics {
                executions: 0,
                successes: 0,
                failures: 0,
                last_execution: None,
                average_latency_ms: 0.0,
            },
            config: HashMap::new(),
            learned_patterns: Vec::new(),
            training_buffer: Vec::new(),
            model_version: 1,
        }
    }

    pub fn absorb(&mut self, trade_data: &super::TradeRecord) {
        // Learn from trade outcomes for continuous improvement
        let pattern = LearnedPattern {
            pattern_id: format!("trade_{}", trade_data.trade_hash.chars().take(8).collect::<String>()),
            pattern_type: if trade_data.net_profit_eth > 0.0 { "profitability".to_string() } else { "risk".to_string() },
            confidence: (trade_data.slippage_bps as f64 / 1000.0).min(1.0),
            features: {
                let mut f = HashMap::new();
                f.insert("gross_profit".to_string(), trade_data.gross_profit_eth);
                f.insert("gas_cost".to_string(), trade_data.gas_cost_eth);
                f.insert("slippage".to_string(), trade_data.slippage_bps as f64);
                f.insert("net_profit".to_string(), trade_data.net_profit_eth);
                f
            },
            outcome: trade_data.net_profit_eth,
            observed_at: trade_data.executed_at.clone(),
        };

        if self.learned_patterns.len() > 10000 {
            self.learned_patterns.remove(0);
        }
        self.learned_patterns.push(pattern);
    }

    pub fn learn_from_anomaly(&mut self, anomaly: &super::m034_anomaly_detector::Anomaly) {
        // Incorporate anomaly insights into learning buffer
        let example = TrainingExample {
            features: anomaly.metrics_affected.iter().enumerate().map(|(_i, m)| (m.clone(), anomaly.current_value)).collect(),
            label: anomaly.severity,
            weight: 1.0,
        };

        if self.training_buffer.len() > 5000 {
            self.training_buffer.remove(0);
        }
        self.training_buffer.push(example);
    }

    pub fn update_from_kpis(&mut self, kpi_data: &HashMap<String, f64>) {
        // Update learning based on fleet KPI trends
        let pattern = LearnedPattern {
            pattern_id: format!("kpi_update_{}", chrono::Utc::now().timestamp()),
            pattern_type: "efficiency".to_string(),
            confidence: 0.5,
            features: kpi_data.clone(),
            outcome: kpi_data.values().sum(),
            observed_at: chrono::Utc::now().to_rfc3339(),
        };

        if self.learned_patterns.len() > 10000 {
            self.learned_patterns.remove(0);
        }
        self.learned_patterns.push(pattern);
    }

    pub fn predict_optimal_params(&self, strategy: &str) -> HashMap<String, f64> {
        // Use learned patterns to suggest optimal parameters
        let mut params = HashMap::new();

        // Analyze historical patterns for this strategy type
        let relevant_patterns: Vec<_> = self.learned_patterns.iter()
            .filter(|p| p.pattern_type == "profitability")
            .collect();

        if !relevant_patterns.is_empty() {
            let avg_slippage = relevant_patterns.iter()
                .filter_map(|p| p.features.get("slippage"))
                .sum::<f64>() / relevant_patterns.len() as f64;

            params.insert("recommended_max_slippage_bps".to_string(), avg_slippage);
            params.insert("confidence".to_string(), 0.75);
        } else {
            params.insert("recommended_max_slippage_bps".to_string(), 100.0);
            params.insert("confidence".to_string(), 0.5);
        }

        params.insert("strategy".to_string(), strategy.len() as f64);
        params
    }

    pub fn get_recent_patterns(&self, limit: usize) -> Vec<&LearnedPattern> {
        self.learned_patterns.iter()
            .rev()
            .take(limit)
            .collect()
    }

    pub fn execute(&mut self) -> ModuleResult {
        if !self.enabled {
            return ModuleResult {
                success: false,
                message: "Module disabled".to_string(),
                data: HashMap::new(),
                execution_time_ms: 0,
                learned_patterns: Vec::new(),
            };
        }

        let start = std::time::Instant::now();
        self.metrics.executions += 1;

        // Return top patterns as learned output
        let learned_patterns: Vec<LearnedPattern> = self.learned_patterns.iter()
            .rev()
            .take(10)
            .cloned()
            .collect();

        let mut data = HashMap::new();
        data.insert("model_version".to_string(), self.model_version.to_string());
        data.insert("training_buffer_size".to_string(), self.training_buffer.len().to_string());

        let result = ModuleResult {
            success: true,
            message: format!("M056 executed: {} patterns learned, {} training examples", 
                self.learned_patterns.len(), self.training_buffer.len()),
            data,
            execution_time_ms: start.elapsed().as_millis() as u64,
            learned_patterns,
        };

        self.metrics.successes += 1;
        self.metrics.last_execution = Some(chrono::Utc::now().to_rfc3339());
        result
    }

    pub fn get_health(&self) -> f64 {
        if self.metrics.executions == 0 {
            return 1.0;
        }
        self.metrics.successes as f64 / self.metrics.executions as f64
    }

    pub fn get_stats(&self) -> String {
        format!(
            r#"{{"executions":{},"successes":{},"failures":{},"health":{:.2},"patterns_learned":{},"training_examples":{}}}"#,
            self.metrics.executions,
            self.metrics.successes,
            self.metrics.failures,
            self.get_health(),
            self.learned_patterns.len(),
            self.training_buffer.len()
        )
    }
}