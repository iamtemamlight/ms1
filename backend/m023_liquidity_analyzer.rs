// ==============================================================================
// M023: Liquidity Analyzer
// Purpose: Liquidity Analyzer - Velocity optimization and management
// CGM Subsystem: Velocity
// ==============================================================================

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ModuleMetrics {
    pub executions: u64,
    pub successes: u64,
    pub failures: u64,
    pub last_execution: Option<String>,
    pub average_latency_ms: f64,
    pub pools_analyzed: u64,
    pub low_liquidity_flags: u64,
}

#[derive(Debug, Clone)]
pub struct ModuleResult {
    pub success: bool,
    pub message: String,
    pub data: HashMap<String, String>,
    pub execution_time_ms: u64,
    pub analysis: Vec<LiquidityAnalysis>,
}

#[derive(Debug, Clone)]
pub struct LiquidityAnalysis {
    pub pool_id: String,
    pub tvl_usd: f64,
    pub volume_24h: f64,
    pub liquidity_score: f64,
    pub low_liquidity: bool,
    pub recommended_trade_size_usd: f64,
}

#[derive(Debug)]
pub struct M23 {
    pub enabled: bool,
    pub metrics: ModuleMetrics,
    pub config: HashMap<String, String>,
    pub min_tvl_usd: f64,
    pub min_volume_24h_usd: f64,
    pub max_slippage_tolerance_bps: u32,
}

impl M23 {
    pub fn new() -> Self {
        Self {
            enabled: true,
            metrics: ModuleMetrics {
                executions: 0,
                successes: 0,
                failures: 0,
                last_execution: None,
                average_latency_ms: 0.0,
                pools_analyzed: 0,
                low_liquidity_flags: 0,
            },
            config: HashMap::new(),
            min_tvl_usd: 100_000.0,
            min_volume_24h_usd: 10_000.0,
            max_slippage_tolerance_bps: 100,
        }
    }

    pub fn analyze_pools(&mut self, pools: &HashMap<String, (f64, f64)>) -> ModuleResult {
        if !self.enabled {
            return ModuleResult {
                success: false,
                message: "Module disabled".to_string(),
                data: HashMap::new(),
                execution_time_ms: 0,
                analysis: Vec::new(),
            };
        }

        let start = std::time::Instant::now();
        self.metrics.executions += 1;

        let mut analysis = Vec::new();
        for (pool_id, (tvl_usd, volume_24h)) in pools {
            let liquidity_score = if *tvl_usd > 0.0 && *volume_24h > 0.0 {
                (*volume_24h / tvl_usd).min(1.0)
            } else {
                0.0
            };
            let low_liquidity = *tvl_usd < self.min_tvl_usd || *volume_24h < self.min_volume_24h_usd;
            let recommended_trade_size_usd = if liquidity_score > 0.0 {
                (tvl_usd * liquidity_score * 0.01).max(100.0)
            } else {
                0.0
            };
            if low_liquidity {
                self.metrics.low_liquidity_flags += 1;
            }
            analysis.push(LiquidityAnalysis {
                pool_id: pool_id.clone(),
                tvl_usd: *tvl_usd,
                volume_24h: *volume_24h,
                liquidity_score,
                low_liquidity,
                recommended_trade_size_usd,
            });
        }

        analysis.sort_by(|a, b| b.liquidity_score.partial_cmp(&a.liquidity_score).unwrap());
        self.metrics.pools_analyzed += analysis.len() as u64;

        let mut data = HashMap::new();
        data.insert("pools".to_string(), analysis.len().to_string());
        data.insert("low_liquidity_flags".to_string(), self.metrics.low_liquidity_flags.to_string());

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
            message: format!("M023 executed: {} pools analyzed", analysis.len()),
            data,
            execution_time_ms: elapsed,
            analysis,
        }
    }

    pub fn execute(&mut self) -> ModuleResult {
        self.analyze_pools(&HashMap::new())
    }

    pub fn get_health(&self) -> f64 {
        if self.metrics.executions == 0 {
            return 1.0;
        }
        self.metrics.successes as f64 / self.metrics.executions as f64
    }

    pub fn get_stats(&self) -> String {
        format!(
            r#"{{"executions":{},"successes":{},"failures":{},"health":{:.2},"pools_analyzed":{}}}"#,
            self.metrics.executions,
            self.metrics.successes,
            self.metrics.failures,
            self.get_health(),
            self.metrics.pools_analyzed
        )
    }
}
