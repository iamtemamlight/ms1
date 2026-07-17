// ==============================================================================
// M027: Slippage Calculator
// Purpose: Slippage Calculator - Efficiency optimization and management
// CGM Subsystem: Efficiency
// ==============================================================================

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ModuleMetrics {
    pub executions: u64,
    pub successes: u64,
    pub failures: u64,
    pub last_execution: Option<String>,
    pub average_latency_ms: f64,
    pub last_slippage_pct: f64,
    pub last_slippage_bps: f64,
}

#[derive(Debug, Clone)]
pub struct ModuleResult {
    pub success: bool,
    pub message: String,
    pub data: HashMap<String, String>,
    pub execution_time_ms: u64,
    pub slippage_pct: f64,
    pub slippage_bps: f64,
}

#[derive(Debug)]
pub struct M27 {
    pub enabled: bool,
    pub metrics: ModuleMetrics,
    pub config: HashMap<String, String>,
    pub default_amount: f64,
    pub default_liquidity: f64,
}

impl M27 {
    pub fn new() -> Self {
        Self {
            enabled: true,
            metrics: ModuleMetrics {
                executions: 0,
                successes: 0,
                failures: 0,
                last_execution: None,
                average_latency_ms: 0.0,
                last_slippage_pct: 0.0,
                last_slippage_bps: 0.0,
            },
            config: HashMap::new(),
            default_amount: 1000.0,
            default_liquidity: 1_000_000.0,
        }
    }

    pub fn set_defaults(&mut self, amount: f64, liquidity: f64) {
        self.default_amount = amount;
        self.default_liquidity = liquidity;
    }

    pub fn calculate_slippage_model(amount_q: f64, liquidity_l: f64) -> f64 {
        if liquidity_l <= 0.0 { return 1.0; }
        amount_q / (liquidity_l + amount_q)
    }

    pub fn execute(&mut self) -> ModuleResult {
        if !self.enabled {
            return ModuleResult {
                success: false,
                message: "Module disabled".to_string(),
                data: HashMap::new(),
                execution_time_ms: 0,
                slippage_pct: 0.0,
                slippage_bps: 0.0,
            };
        }

        let start = std::time::Instant::now();
        self.metrics.executions += 1;

        let amount_q = self.config.get("amount_q")
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(self.default_amount);
        let liquidity_l = self.config.get("liquidity_l")
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(self.default_liquidity);

        let slippage_pct = Self::calculate_slippage_model(amount_q, liquidity_l);
        let slippage_bps = (slippage_pct * 10_000.0).round() as u32;

        let mut data = HashMap::new();
        data.insert("amount_q".to_string(), amount_q.to_string());
        data.insert("liquidity_l".to_string(), liquidity_l.to_string());
        data.insert("slippage_pct".to_string(), slippage_pct.to_string());
        data.insert("slippage_bps".to_string(), slippage_bps.to_string());

        let elapsed = start.elapsed().as_millis() as u64;
        self.metrics.successes += 1;
        self.metrics.last_execution = Some(chrono::Utc::now().to_rfc3339());
        self.metrics.last_slippage_pct = slippage_pct;
        self.metrics.last_slippage_bps = slippage_bps as f64;

        ModuleResult {
            success: true,
            message: format!("M027 executed: {} bps slippage", slippage_bps),
            data,
            execution_time_ms: elapsed,
            slippage_pct,
            slippage_bps: slippage_bps as f64,
        }
    }

    pub fn get_health(&self) -> f64 {
        if self.metrics.executions == 0 {
            return 1.0;
        }
        self.metrics.successes as f64 / self.metrics.executions as f64
    }

    pub fn get_stats(&self) -> String {
        format!(
            r#"{{"executions":{},"successes":{},"failures":{},"health":{:.2},"slippage_bps":{:.1}}}"#,
            self.metrics.executions,
            self.metrics.successes,
            self.metrics.failures,
            self.get_health(),
            self.metrics.last_slippage_bps
        )
    }
}
