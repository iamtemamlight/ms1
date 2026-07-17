// ==============================================================================
// M050: Governance Engine
// Purpose: Governance Engine - coordinates and validates governance readiness.
// CGM Subsystem: All
//
// NOTE: This is the backend adapter for the AllBright AgentOS governance layer.
// The authoritative governance logic (Gatekeeper, Reflection Engine, dual-agent
// orchestration) lives in `crates/governance`. This module performs a real
// readiness self-check rather than unconditionally returning success.
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
}

#[derive(Debug)]
pub struct M50 {
    pub enabled: bool,
    pub metrics: ModuleMetrics,
    pub config: HashMap<String, String>,
}

impl M50 {
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
        }
    }

    pub fn execute(&mut self) -> ModuleResult {
        if !self.enabled {
            return ModuleResult {
                success: false,
                message: "Module disabled".to_string(),
                data: HashMap::new(),
                execution_time_ms: 0,
            };
        }

        let start = std::time::Instant::now();
        self.metrics.executions += 1;

        // Real governance readiness check: every core policy domain the
        // Gatekeeper requires (spec §11.2: "references a known policy domain")
        // must be configured and non-empty.
        let required_policies = [
            "AOS-GOV-01",
            "AOS-GOV-02",
            "AOS-GOV-03",
            "AOS-GOV-04",
            "AOS-GOV-05",
        ];
        let mut missing = Vec::new();
        for p in &required_policies {
            if !self.config.get(*p).map_or(false, |v| !v.is_empty()) {
                missing.push((*p).to_string());
            }
        }

        let ready = missing.is_empty();
        let mut data = HashMap::new();
        data.insert("required_policies".to_string(), required_policies.len().to_string());
        data.insert("missing_policies".to_string(), missing.len().to_string());

        let result = if ready {
            ModuleResult {
                success: true,
                message: "M050 governance engine ready".to_string(),
                data,
                execution_time_ms: start.elapsed().as_millis() as u64,
            }
        } else {
            ModuleResult {
                success: false,
                message: format!("M050 missing required policies: {}", missing.join(",")),
                data,
                execution_time_ms: start.elapsed().as_millis() as u64,
            }
        };

        if result.success {
            self.metrics.successes += 1;
        } else {
            self.metrics.failures += 1;
        }
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
            r#"{{"executions":{},"successes":{},"failures":{},"health":{:.2}}}"#,
            self.metrics.executions,
            self.metrics.successes,
            self.metrics.failures,
            self.get_health()
        )
    }
}
