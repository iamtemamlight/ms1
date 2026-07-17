// ==============================================================================
// M019: Relay Monitor
// Purpose: Track relay performance and inclusion rates
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
}

#[derive(Debug, Clone)]
pub struct ModuleResult {
    pub success: bool,
    pub message: String,
    pub data: HashMap<String, String>,
    pub execution_time_ms: u64,
    pub relay_stats: Vec<RelayStats>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RelayStats {
    pub relay: String,
    pub inclusion_rate: f64,
    pub avg_latency_ms: f64,
    pub total_bundles: u64,
    pub successful_bundles: u64,
    pub last_seen: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RelayPerformance {
    pub relay: String,
    pub timestamp: String,
    pub profit_eth: f64,
    pub included: bool,
    pub latency_ms: u64,
}

#[derive(Debug)]
pub struct M19 {
    pub enabled: bool,
    pub metrics: ModuleMetrics,
    pub config: HashMap<String, String>,
    pub relay_history: Vec<RelayPerformance>,
    pub relay_stats: HashMap<String, RelayStats>,
}

impl M19 {
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
            relay_history: Vec::new(),
            relay_stats: HashMap::new(),
        }
    }

    pub fn record_bundle(
        &mut self,
        relay: &str,
        profit_eth: f64,
        included: bool,
        latency_ms: u64,
    ) {
        let performance = RelayPerformance {
            relay: relay.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            profit_eth,
            included,
            latency_ms,
        };

        if self.relay_history.len() > 10000 {
            self.relay_history.remove(0);
        }
        self.relay_history.push(performance);

        let stats = self.relay_stats.entry(relay.to_string()).or_insert(RelayStats {
            relay: relay.to_string(),
            inclusion_rate: 0.0,
            avg_latency_ms: 0.0,
            total_bundles: 0,
            successful_bundles: 0,
            last_seen: chrono::Utc::now().to_rfc3339(),
        });

        stats.total_bundles += 1;
        if included {
            stats.successful_bundles += 1;
        }
        stats.inclusion_rate = stats.successful_bundles as f64 / stats.total_bundles as f64;
        stats.avg_latency_ms = ((stats.avg_latency_ms * (stats.total_bundles - 1) as f64) + latency_ms as f64) / stats.total_bundles as f64;
        stats.last_seen = chrono::Utc::now().to_rfc3339();
    }

    pub fn get_top_relays(&self, limit: usize) -> Vec<&RelayStats> {
        let mut stats: Vec<_> = self.relay_stats.values().collect();
        stats.sort_by(|a, b| b.avg_latency_ms.partial_cmp(&a.avg_latency_ms).unwrap_or(0.cmp(&0));
        stats.iter().take(limit).collect()
    }

    pub fn get_degrading_relays(&self, threshold: f64) -> Vec<&RelayStats> {
        self.relay_stats
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
                relay_stats: Vec::new(),
            };
        }

        let start = std::time::Instant::now();
        self.metrics.executions += 1;

        let relay_stats: Vec<RelayStats> = self.relay_stats.values().cloned().collect();

        let result = ModuleResult {
            success: true,
            message: format!("M019 executed: {} relays tracked", relay_stats.len()),
            data: HashMap::new(),
            execution_time_ms: start.elapsed().as_millis() as u64,
            relay_stats,
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
            r#"{{"executions":{},"successes":{},"failures":{},"health":{:.2},"relays_tracked":{}}}"#,
            self.metrics.executions,
            self.metrics.successes,
            self.metrics.failures,
            self.get_health(),
            self.relay_stats.len()
        )
    }
}