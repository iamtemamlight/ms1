// ==============================================================================
// M022: Arbitrage Detector
// Purpose: Arbitrage Detector - Profit optimization and management
// CGM Subsystem: Profit
// ==============================================================================

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ModuleMetrics {
    pub executions: u64,
    pub successes: u64,
    pub failures: u64,
    pub last_execution: Option<String>,
    pub average_latency_ms: f64,
    pub opportunities_detected: u64,
    pub profitable_opportunities: u64,
}

#[derive(Debug, Clone)]
pub struct ModuleResult {
    pub success: bool,
    pub message: String,
    pub data: HashMap<String, String>,
    pub execution_time_ms: u64,
    pub opportunities: Vec<ArbitrageOpportunity>,
}

#[derive(Debug, Clone)]
pub struct ArbitrageOpportunity {
    pub pair: String,
    pub buy_dex: String,
    pub sell_dex: String,
    pub expected_profit_eth: f64,
    pub gas_cost_eth: f64,
    pub net_profit_eth: f64,
    pub confidence: f64,
}

#[derive(Debug)]
pub struct M22 {
    pub enabled: bool,
    pub metrics: ModuleMetrics,
    pub config: HashMap<String, String>,
    pub min_profit_threshold_eth: f64,
    pub max_gas_price_gwei: f64,
}

impl M22 {
    pub fn new() -> Self {
        Self {
            enabled: true,
            metrics: ModuleMetrics {
                executions: 0,
                successes: 0,
                failures: 0,
                last_execution: None,
                average_latency_ms: 0.0,
                opportunities_detected: 0,
                profitable_opportunities: 0,
            },
            config: HashMap::new(),
            min_profit_threshold_eth: 0.005,
            max_gas_price_gwei: 100.0,
        }
    }

    pub fn scan_opportunities(&mut self, pool_data: &HashMap<String, (String, String, f64)>) -> ModuleResult {
        if !self.enabled {
            return ModuleResult {
                success: false,
                message: "Module disabled".to_string(),
                data: HashMap::new(),
                execution_time_ms: 0,
                opportunities: Vec::new(),
            };
        }

        let start = std::time::Instant::now();
        self.metrics.executions += 1;

        let mut opportunities = Vec::new();
        let mut pair_index: HashMap<String, Vec<(String, f64)>> = HashMap::new();

        for (key, (dex, pair, price)) in pool_data {
            pair_index.entry(pair.clone()).or_default().push((dex.clone(), *price));
            let _ = key;
        }

        let pair_count = pair_index.len();
        for (pair, entries) in &pair_index {
            if entries.len() < 2 {
                continue;
            }
            for i in 0..entries.len() {
                for j in (i + 1)..entries.len() {
                    let (dex_a, price_a) = &entries[i];
                    let (dex_b, price_b) = &entries[j];
                    let spread = (price_a - price_b).abs();
                    if spread <= 0.0 {
                        continue;
                    }
                    let trade_size = 10_000.0;
                    let gross_profit_eth = spread * trade_size * 0.0001;
                    let gas_cost_eth = self.max_gas_price_gwei * 21000.0 * 1e-9;
                    let net_profit_eth = gross_profit_eth - gas_cost_eth;
                    if net_profit_eth > self.min_profit_threshold_eth {
                        opportunities.push(ArbitrageOpportunity {
                            pair: pair.clone(),
                            buy_dex: if *price_a < *price_b { dex_a.clone() } else { dex_b.clone() },
                            sell_dex: if *price_a > *price_b { dex_a.clone() } else { dex_b.clone() },
                            expected_profit_eth: gross_profit_eth,
                            gas_cost_eth,
                            net_profit_eth,
                            confidence: 0.6,
                        });
                    }
                }
            }
        }

        opportunities.sort_by(|a, b| b.net_profit_eth.partial_cmp(&a.net_profit_eth).unwrap());
        let profitable_count = opportunities.len() as u64;
        self.metrics.opportunities_detected += opportunities.len() as u64;
        self.metrics.profitable_opportunities += profitable_count;

        let mut data = HashMap::new();
        data.insert("pair_count".to_string(), pair_count.to_string());
        data.insert("opportunities".to_string(), opportunities.len().to_string());

        let elapsed = start.elapsed().as_millis() as u64;
        self.metrics.successes += 1;
        self.metrics.last_execution = Some(chrono::Utc::now().to_rfc3339());
        self.metrics.average_latency_ms = if self.metrics.executions == 1 {
            elapsed as f64
        } else {
            (self.metrics.average_latency_ms * (self.metrics.executions - 1) as f64 + elapsed as f64)
                / self.metrics.executions as f64
        };

        ModuleResult {
            success: true,
            message: format!("M022 executed: {} opportunities", opportunities.len()),
            data,
            execution_time_ms: elapsed,
            opportunities,
        }
    }

    pub fn execute(&mut self) -> ModuleResult {
        self.scan_opportunities(&HashMap::new())
    }

    pub fn get_health(&self) -> f64 {
        if self.metrics.executions == 0 {
            return 1.0;
        }
        self.metrics.successes as f64 / self.metrics.executions as f64
    }

    pub fn get_stats(&self) -> String {
        format!(
            r#"{{"executions":{},"successes":{},"failures":{},"health":{:.2},"opportunities_detected":{}}}"#,
            self.metrics.executions,
            self.metrics.successes,
            self.metrics.failures,
            self.get_health(),
            self.metrics.opportunities_detected
        )
    }
}
