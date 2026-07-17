// ==============================================================================
// M078: Governance Auditor
// Purpose: Governance Auditor - audits policy-reference well-formedness.
// CGM Subsystem: Quality
//
// NOTE: Backend adapter for the AllBright AgentOS governance layer. The
// authoritative audit logic lives in `crates/governance`. This module performs
// a real audit of configured policy references (they must be well-formed
// "AOS-*" domains, per the Gatekeeper's PolicyCompliance criterion §11.2) rather
// than unconditionally returning success.
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
pub struct M78 {
    pub enabled: bool,
    pub metrics: ModuleMetrics,
    pub config: HashMap<String, String>,
}

impl M78 {
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

        // Real audit: every configured policy reference must be a well-formed
        // "AOS-*" governance domain (Gatekeeper PolicyCompliance criterion §11.2).
        let mut malformed = 0usize;
        for value in self.config.values() {
            if !value.is_empty() && !value.starts_with("AOS-") {
                malformed += 1;
            }
        }

        let mut data = HashMap::new();
        data.insert("configured_refs".to_string(), self.config.len().to_string());
        data.insert("malformed_refs".to_string(), malformed.to_string());

        let clean = malformed == 0;
        let result = if clean {
            ModuleResult {
                success: true,
                message: "M078 audit passed: all policy references well-formed".to_string(),
                data,
                execution_time_ms: start.elapsed().as_millis() as u64,
            }
        } else {
            ModuleResult {
                success: false,
                message: format!("M078 audit failed: {malformed} malformed policy reference(s)"),
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
