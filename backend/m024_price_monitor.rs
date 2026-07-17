// ==============================================================================
// M024: Price Monitor
// Purpose: Price Monitor - Profit optimization and management
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
    pub prices_tracked: u64,
    pub stale_prices_detected: u64,
}

#[derive(Debug, Clone)]
pub struct ModuleResult {
    pub success: bool,
    pub message: String,
    pub data: HashMap<String, String>,
    pub execution_time_ms: u64,
    pub prices: Vec<PriceTick>,
}

#[derive(Debug, Clone)]
pub struct PriceTick {
    pub symbol: String,
    pub price_usd: f64,
    pub timestamp: String,
    pub source: String,
    pub stale: bool,
    pub age_seconds: u64,
}

#[derive(Debug)]
pub struct M24 {
    pub enabled: bool,
    pub metrics: ModuleMetrics,
    pub config: HashMap<String, String>,
    pub tracked_symbols: Vec<String>,
    pub max_staleness_seconds: u64,
    pub last_prices: HashMap<String, PriceTick>,
}

impl M24 {
    pub fn new() -> Self {
        Self {
            enabled: true,
            metrics: ModuleMetrics {
                executions: 0,
                successes: 0,
                failures: 0,
                last_execution: None,
                average_latency_ms: 0.0,
                prices_tracked: 0,
                stale_prices_detected: 0,
            },
            config: HashMap::new(),
            tracked_symbols: vec!["ETH".to_string(), "USDC".to_string(), "WBTC".to_string()],
            max_staleness_seconds: 30,
            last_prices: HashMap::new(),
        }
    }

    pub fn update_prices(&mut self, prices: &HashMap<String, (f64, String)>) -> ModuleResult {
        if !self.enabled {
            return ModuleResult {
                success: false,
                message: "Module disabled".to_string(),
                data: HashMap::new(),
                execution_time_ms: 0,
                prices: Vec::new(),
            };
        }

        let start = std::time::Instant::now();
        self.metrics.executions += 1;

        let mut ticks = Vec::new();
        let now = chrono::Utc::now();
        for symbol in &self.tracked_symbols {
            if let Some((price_usd, source)) = prices.get(symbol) {
                let timestamp = now.to_rfc3339();
                let age_seconds = 0;
                let stale = false;
                let tick = PriceTick {
                    symbol: symbol.clone(),
                    price_usd: *price_usd,
                    timestamp,
                    source: source.clone(),
                    stale,
                    age_seconds,
                };
                self.last_prices.insert(symbol.clone(), tick.clone());
                ticks.push(tick);
            }
        }

        self.metrics.prices_tracked += ticks.len() as u64;

        let mut data = HashMap::new();
        data.insert("symbols".to_string(), ticks.len().to_string());
        data.insert("stale".to_string(), "0".to_string());

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
            message: format!("M024 executed: {} prices updated", ticks.len()),
            data,
            execution_time_ms: elapsed,
            prices: ticks,
        }
    }

    pub fn detect_stale(&mut self) -> ModuleResult {
        if !self.enabled {
            return ModuleResult {
                success: false,
                message: "Module disabled".to_string(),
                data: HashMap::new(),
                execution_time_ms: 0,
                prices: Vec::new(),
            };
        }

        let start = std::time::Instant::now();
        self.metrics.executions += 1;

        let now = chrono::Utc::now();
        let mut ticks = Vec::new();
        for (symbol, tick) in &self.last_prices {
            if let Ok(ts) = chrono::DateTime::parse_from_rfc3339(&tick.timestamp) {
                let age = now.timestamp() - ts.timestamp();
                let stale = age as u64 > self.max_staleness_seconds;
                if stale {
                    self.metrics.stale_prices_detected += 1;
                }
                ticks.push(PriceTick {
                    symbol: symbol.clone(),
                    price_usd: tick.price_usd,
                    timestamp: tick.timestamp.clone(),
                    source: tick.source.clone(),
                    stale,
                    age_seconds: age as u64,
                });
            }
        }

        let mut data = HashMap::new();
        data.insert("symbols".to_string(), ticks.len().to_string());
        data.insert("stale_prices_detected".to_string(), self.metrics.stale_prices_detected.to_string());

        let elapsed = start.elapsed().as_millis() as u64;
        self.metrics.successes += 1;
        self.metrics.last_execution = Some(chrono::Utc::now().to_rfc3339());

        ModuleResult {
            success: true,
            message: format!("M024 executed: {} ticks, {} stale", ticks.len(), self.metrics.stale_prices_detected),
            data,
            execution_time_ms: elapsed,
            prices: ticks,
        }
    }

    pub fn execute(&mut self) -> ModuleResult {
        if self.last_prices.is_empty() {
            let mut prices = HashMap::new();
            prices.insert("ETH".to_string(), (2000.0, "default".to_string()));
            prices.insert("USDC".to_string(), (1.0, "default".to_string()));
            self.update_prices(&prices)
        } else {
            self.detect_stale()
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
            r#"{{"executions":{},"successes":{},"failures":{},"health":{:.2},"prices_tracked":{}}}"#,
            self.metrics.executions,
            self.metrics.successes,
            self.metrics.failures,
            self.get_health(),
            self.metrics.prices_tracked
        )
    }
}
