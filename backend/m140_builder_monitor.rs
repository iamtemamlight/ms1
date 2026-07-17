// ==============================================================================
// M018: Builder Monitor
// Purpose: Track builder performance and inclusion rates
// CGM Subsystem: Velocity
// ==============================================================================

use std::collections::HashMap;
use std::time::Duration;

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
    pub builder_stats: Vec<BuilderStats>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BuilderStats {
    pub builder: String,
    pub inclusion_rate: f64,
    pub avg_profit_eth: f64,
    pub total_bundles: u64,
    pub successful_bundles: u64,
    pub last_seen: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BuilderPerformance {
    pub builder: String,
    pub timestamp: String,
    pub profit_eth: f64,
    pub included: bool,
    pub latency_ms: u64,
}

#[derive(Debug)]
pub struct M18 {
    pub enabled: bool,
    pub metrics: ModuleMetrics,
    pub config: HashMap<String, String>,
    pub builder_history: Vec<BuilderPerformance>,
    pub builder_stats: HashMap<String, BuilderStats>,
}

impl M18 {
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
            builder_history: Vec::new(),
            builder_stats: HashMap::new(),
        }
    }

    pub fn record_bundle(
        &mut self,
        builder: &str,
        profit_eth: f64,
        included: bool,
        latency_ms: u64,
    ) {
        let performance = BuilderPerformance {
            builder: builder.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            profit_eth,
            included,
            latency_ms,
        };

        if self.builder_history.len() > 10000 {
            self.builder_history.remove(0);
        }
        self.builder_history.push(performance);

        // Update stats
        let stats = self.builder_stats.entry(builder.to_string()).or_insert(BuilderStats {
            builder: builder.to_string(),
            inclusion_rate: 0.0,
            avg_profit_eth: 0.0,
            total_bundles: 0,
            successful_bundles: 0,
            last_seen: chrono::Utc::now().to_rfc3339(),
        });

        stats.total_bundles += 1;
        if included {
            stats.successful_bundles += 1;
        }
        stats.inclusion_rate = stats.successful_bundles as f64 / stats.total_bundles as f64;
        stats.avg_profit_eth = ((stats.avg_profit_eth * (stats.total_bundles - 1) as f64) + profit_eth) / stats.total_bundles as f64;
        stats.last_seen = chrono::Utc::now().to_rfc3339();
    }

    pub fn get_top_builders(&self, limit: usize) -> Vec<&BuilderStats> {
        let mut stats: Vec<_> = self.builder_stats.values().collect();
        stats.sort_by(|a, b| b.avg_profit_eth.partial_cmp(&a.avg_profit_eth).unwrap_or(0.cmp(&0));
        stats.iter().take(limit).collect()
    }

    pub fn get_degrading_builders(&self, threshold: f64) -> Vec<&BuilderStats> {
        self.builder_stats
            .values()
            .filter(|s| s.inclusion_rate < threshold)
            .collect()
    }

    pub fn execute(&mut self) -> ModuleResult {
        if !self.enabled {
            return ModuleResult {
                success: false,
                message: "Module disabled".to_string(),
                data: HashMap::new(),
                execution_time_ms: 0,
                builder_stats: Vec::new(),
            };
        }

        let start = std::time::Instant::now();
        self.metrics.executions += 1;

        let builder_stats: Vec<BuilderStats> = self.builder_stats.values().cloned().collect();

        let result = ModuleResult {
            success: true,
            message: format!("M018 executed: {} builders tracked", builder_stats.len()),
            data: HashMap::new(),
            execution_time_ms: start.elapsed().as_millis() as u64,
            builder_stats,
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
            r#"{{"executions":{},"successes":{},"failures":{},"health":{:.2},"builders_tracked":{}}}"#,
            self.metrics.executions,
            self.metrics.successes,
            self.metrics.failures,
            self.get_health(),
            self.builder_stats.len()
        )
    }
}