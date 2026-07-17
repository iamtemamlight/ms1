// ==============================================================================
// M033: Audit Trail
// Purpose: Audit Trail - Quality optimization and management
// CGM Subsystem: Quality
// ==============================================================================

use std::collections::HashMap;
use std::time::SystemTime;

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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuditEntry {
    pub timestamp: String,
    pub entry_type: String,
    pub actor: String,
    pub action: String,
    pub resource: String,
    pub outcome: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub struct M33 {
    pub enabled: bool,
    pub metrics: ModuleMetrics,
    pub config: HashMap<String, String>,
    pub entries: Vec<AuditEntry>,
    pub max_entries: usize,
}

impl M33 {
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
            entries: Vec::new(),
            max_entries: 10000,
        }
    }

    pub fn record(&mut self, entry_type: &str, actor: &str, action: &str, resource: &str, outcome: &str, metadata: Option<HashMap<String, String>>) -> ModuleResult {
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

        let timestamp = chrono::Utc::now().to_rfc3339();
        let entry = AuditEntry {
            timestamp,
            entry_type: entry_type.to_string(),
            actor: actor.to_string(),
            action: action.to_string(),
            resource: resource.to_string(),
            outcome: outcome.to_string(),
            metadata: metadata.unwrap_or_default(),
        };

        // Enforce max entries to prevent unbounded growth
        if self.entries.len() >= self.max_entries {
            self.entries.remove(0);
        }
        self.entries.push(entry);
        self.metrics.successes += 1;
        self.metrics.last_execution = Some(chrono::Utc::now().to_rfc3339());

        ModuleResult {
            success: true,
            message: format!("Audit trail record created for actor={}", actor),
            data: HashMap::new(),
            execution_time_ms: start.elapsed().as_millis() as u64,
        }
    }

    pub fn query(&self, filter_type: Option<&str>, filter_actor: Option<&str>) -> Vec<&AuditEntry> {
        self.entries.iter()
            .filter(|e| {
                filter_type.map_or(true, |t| e.entry_type == t) &&
                filter_actor.map_or(true, |a| e.actor == a)
            })
            .collect()
    }

    pub fn query_by_resource(&self, resource: &str) -> Vec<&AuditEntry> {
        self.entries.iter()
            .filter(|e| e.resource == resource)
            .collect()
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

        let result = ModuleResult {
            success: true,
            message: format!("M033 executed: {} audit entries recorded", self.entries.len()),
            data: HashMap::new(),
            execution_time_ms: start.elapsed().as_millis() as u64,
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
            r#"{{"executions":{},"successes":{},"failures":{},"health":{:.2},"audit_entries":{}}}"#,
            self.metrics.executions,
            self.metrics.successes,
            self.metrics.failures,
            self.get_health(),
            self.entries.len()
        )
    }

    pub fn get_recent_trades(&self, limit: usize) -> Vec<&AuditEntry> {
        self.entries.iter()
            .rev()
            .take(limit)
            .filter(|e| e.entry_type == "TRADE")
            .collect()
    }
}