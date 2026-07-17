// ==============================================================================
// M034: Anomaly Detector
// Purpose: Anomaly Detector - Security optimization and management
// CGM Subsystem: Security
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
    pub anomalies: Vec<Anomaly>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Anomaly {
    pub anomaly_type: String,
    pub severity: f64,  // 0.0 to 1.0
    pub description: String,
    pub detected_at: String,
    pub metrics_affected: Vec<String>,
    pub threshold_breached: f64,
    pub current_value: f64,
}

#[derive(Debug)]
pub struct M34 {
    pub enabled: bool,
    pub metrics: ModuleMetrics,
    pub config: HashMap<String, String>,
    pub anomaly_history: Vec<Anomaly>,
    pub detection_thresholds: AnomalyThresholds,
}

#[derive(Debug, Clone)]
pub struct AnomalyThresholds {
    pub slippage_spike_bps: f64,      // default 500.0 bps
    pub gas_spike_gwei: f64,          // default 200.0 gwei
    pub latency_spike_ms: f64,        // default 500.0 ms
    pub profit_drop_pct: f64,         // default 0.5 (50% drop)
    pub failure_rate_pct: f64,        // default 0.1 (10% failure)
}

impl Default for AnomalyThresholds {
    fn default() -> Self {
        Self {
            slippage_spike_bps: 500.0,
            gas_spike_gwei: 200.0,
            latency_spike_ms: 500.0,
            profit_drop_pct: 0.5,
            failure_rate_pct: 0.1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RiskAnalysis {
    pub risk_score: f64,
    pub critical_count: u64,
    pub warning_count: u64,
    pub requires_intervention: bool,
}

#[derive(Debug, Clone)]
pub struct FailureAnalysis {
    pub could_have_been_predicted: bool,
    pub simulation_missed: bool,
    pub gas_underestimated: bool,
    pub liquidity_overestimated: bool,
    pub prevention_suggestions: Vec<String>,
}

impl M34 {
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
            anomaly_history: Vec::new(),
            detection_thresholds: AnomalyThresholds::default(),
        }
    }

    pub fn detect_anomalies(&mut self, trade_records: &[(String, f64, f64, f64)]) -> ModuleResult {
        // trade_records: (trade_hash, slippage_bps, gas_cost_eth, net_profit_eth)
        if !self.enabled {
            return ModuleResult {
                success: false,
                message: "Module disabled".to_string(),
                data: HashMap::new(),
                execution_time_ms: 0,
                anomalies: Vec::new(),
            };
        }

        let start = std::time::Instant::now();
        self.metrics.executions += 1;

        let mut anomalies = Vec::new();

        // Detect slippage spikes
        for (trade_hash, slippage_bps, gas_cost_eth, net_profit_eth) in trade_records {
            if *slippage_bps > self.detection_thresholds.slippage_spike_bps {
                anomalies.push(Anomaly {
                    anomaly_type: "SLIPPAGE_SPIKE".to_string(),
                    severity: (*slippage_bps / self.detection_thresholds.slippage_spike_bps).min(1.0),
                    description: format!("Slippage exceeded threshold: {:.1} bps > {:.1} bps", slippage_bps, self.detection_thresholds.slippage_spike_bps),
                    detected_at: chrono::Utc::now().to_rfc3339(),
                    metrics_affected: vec!["slippage".to_string()],
                    threshold_breached: self.detection_thresholds.slippage_spike_bps,
                    current_value: *slippage_bps,
                });
            }

            // Detect gas cost spikes
            if *gas_cost_eth > (*gas_cost_eth * 0.5).max(self.detection_thresholds.gas_spike_gwei * 21000.0 * 1e-9) {
                anomalies.push(Anomaly {
                    anomaly_type: "GAS_SPIKE".to_string(),
                    severity: 0.7,
                    description: format!("Gas cost exceeded expected threshold for trade {}", trade_hash),
                    detected_at: chrono::Utc::now().to_rfc3339(),
                    metrics_affected: vec!["gas_cost".to_string()],
                    threshold_breached: self.detection_thresholds.gas_spike_gwei * 21000.0 * 1e-9,
                    current_value: *gas_cost_eth,
                });
            }

            // Detect profit drops
            if *net_profit_eth < 0.0 {
                anomalies.push(Anomaly {
                    anomaly_type: "NEGATIVE_PROFIT".to_string(),
                    severity: 0.8,
                    description: format!("Negative profit detected for trade {}", trade_hash),
                    detected_at: chrono::Utc::now().to_rfc3339(),
                    metrics_affected: vec!["net_profit".to_string()],
                    threshold_breached: 0.0,
                    current_value: *net_profit_eth,
                });
            }
        }

        // Store anomalies in history
        for anomaly in &anomalies {
            if self.anomaly_history.len() > 1000 {
                self.anomaly_history.remove(0);
            }
            self.anomaly_history.push(anomaly.clone());
        }

        let elapsed = start.elapsed().as_millis() as u64;
        self.metrics.successes += 1;
        self.metrics.last_execution = Some(chrono::Utc::now().to_rfc3339());

        ModuleResult {
            success: true,
            message: format!("M034 detected {} anomalies", anomalies.len()),
            data: HashMap::new(),
            execution_time_ms: elapsed,
            anomalies,
        }
    }

    pub fn analyze_risk_pattern(&self, recent_anomalies: &[Anomaly]) -> RiskAnalysis {
        let total_severity: f64 = recent_anomalies.iter().map(|a| a.severity).sum();
        let avg_severity = if recent_anomalies.is_empty() { 0.0 } else { total_severity / recent_anomalies.len() as f64 };
        
        let critical_count = recent_anomalies.iter().filter(|a| a.severity > 0.8).count();
        let warning_count = recent_anomalies.iter().filter(|a| a.severity > 0.5 && a.severity <= 0.8).count();

        RiskAnalysis {
            risk_score: (avg_severity * 100.0).min(100.0),
            critical_count: critical_count as u64,
            warning_count: warning_count as u64,
            requires_intervention: avg_severity > 0.7,
        }
    }

    /// Analyze why a failed trade occurred - answers NEWREFINE checklist questions
    pub fn analyze_failure(&self, failed_trade: &super::m025_trade_executor::ModuleResult, verdict: Option<&super::SimulationVerdict>) -> FailureAnalysis {
        let mut could_have_been_predicted = false;
        let mut simulation_missed = false;
        let mut gas_underestimated = false;
        let mut liquidity_overestimated = false;
        let mut prevention_suggestions: Vec<String> = Vec::new();

        // Check anomaly history for patterns
        for anomaly in &self.anomaly_history {
            match anomaly.anomaly_type.as_str() {
                "GAS_SPIKE" => {
                    gas_underestimated = true;
                    could_have_been_predicted = true;
                    prevention_suggestions.push("Consider using higher gas buffer or waiting for gas price drop".to_string());
                }
                "SLIPPAGE_SPIKE" => {
                    liquidity_overestimated = true;
                    could_have_been_predicted = true;
                    prevention_suggestions.push("Reduce trade size or use liquidity-aware routing".to_string());
                }
                "NEGATIVE_PROFIT" => {
                    could_have_been_predicted = true;
                    prevention_suggestions.push("Re-run simulation with fresh data before execution".to_string());
                }
                _ => {}
            }
        }

        // Check if simulation gate would have caught this
        if let Some(v) = verdict {
            if !v.passed {
                simulation_missed = true;
                for warning in &v.warnings {
                    if warning.contains("gas") {
                        gas_underestimated = true;
                    }
                    if warning.contains("liquidity") || warning.contains("slippage") {
                        liquidity_overestimated = true;
                    }
                }
            }
        }

        // Check trade rejection reasons
        if !failed_trade.success {
            let msg = &failed_trade.message;
            if msg.contains("SLIPPAGE") {
                liquidity_overestimated = true;
                prevention_suggestions.push("Adjust slippage threshold or split order".to_string());
            }
            if msg.contains("PROFIT") {
                prevention_suggestions.push("Increase minimum profit threshold or skip trade".to_string());
            }
        }

        FailureAnalysis {
            could_have_been_predicted,
            simulation_missed,
            gas_underestimated,
            liquidity_overestimated,
            prevention_suggestions,
        }
    }

    pub fn execute(&mut self) -> ModuleResult {
        if !self.enabled {
            return ModuleResult {
                success: false,
                message: "Module disabled".to_string(),
                data: HashMap::new(),
                execution_time_ms: 0,
                anomalies: Vec::new(),
            };
        }

        let start = std::time::Instant::now();
        self.metrics.executions += 1;

        let result = ModuleResult {
            success: true,
            message: format!("M034 executed: {} anomalies in history", self.anomaly_history.len()),
            data: HashMap::new(),
            execution_time_ms: start.elapsed().as_millis() as u64,
            anomalies: Vec::new(),
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
            r#"{{"executions":{},"successes":{},"failures":{},"health":{:.2},"anomalies_detected":{}}}"#,
            self.metrics.executions,
            self.metrics.successes,
            self.metrics.failures,
            self.get_health(),
            self.anomaly_history.len()
        )
    }
}