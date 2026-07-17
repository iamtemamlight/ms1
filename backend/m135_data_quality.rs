// ==============================================================================
// M020: Data Quality
// Purpose: Detect duplicate events, missing blocks, and data anomalies
// CGM Subsystem: Quality
// ==============================================================================

use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct ModuleMetrics {
    pub executions: u64,
    pub successes: u64,
    pub failures: u64,
    pub last_execution: Option<String>,
    pub average_latency_ms: f64,
    pub duplicates_detected: u64,
    pub missing_blocks_detected: u64,
}

#[derive(Debug, Clone)]
pub struct ModuleResult {
    pub success: bool,
    pub message: String,
    pub data: HashMap<String, String>,
    pub execution_time_ms: u64,
    pub duplicates_found: u64,
    pub missing_blocks: Vec<u64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MarketEvent {
    pub event_id: String,
    pub block_number: u64,
    pub timestamp: i64,
    pub event_type: String,
    pub data: HashMap<String, String>,
}

#[derive(Debug)]
pub struct M20 {
    pub enabled: bool,
    pub metrics: ModuleMetrics,
    pub config: HashMap<String, String>,
    pub event_registry: HashSet<String>,
    pub last_block_seen: u64,
    pub block_timestamps: HashMap<u64, i64>,
    pub max_gap_blocks: u64,
}

impl M20 {
    pub fn new() -> Self {
        Self {
            enabled: true,
            metrics: ModuleMetrics {
                executions: 0,
                successes: 0,
                failures: 0,
                last_execution: None,
                average_latency_ms: 0.0,
                duplicates_detected: 0,
                missing_blocks_detected: 0,
            },
            config: HashMap::new(),
            event_registry: HashSet::new(),
            last_block_seen: 0,
            block_timestamps: HashMap::new(),
            max_gap_blocks: 10,
        }
    }

    pub fn record_event(&mut self, event: MarketEvent) -> ModuleResult {
        if !self.enabled {
            return ModuleResult {
                success: false,
                message: "Module disabled".to_string(),
                data: HashMap::new(),
                execution_time_ms: 0,
                duplicates_found: 0,
                missing_blocks: Vec::new(),
            };
        }

        let start = Instant::now();
        self.metrics.executions += 1;

        let mut duplicates_found = 0;
        
        // Check for duplicates
        if !self.event_registry.insert(event.event_id.clone()) {
            duplicates_found += 1;
            self.metrics.duplicates_detected += 1;
        }

        // Check for missing blocks
        let mut missing_blocks = Vec::new();
        if self.last_block_seen > 0 && event.block_number > self.last_block_seen + self.max_gap_blocks {
            for b in self.last_block_seen + 1..event.block_number {
                missing_blocks.push(b);
            }
            self.metrics.missing_blocks_detected += missing_blocks.len() as u64;
        }

        // Update block tracking
        self.block_timestamps.insert(event.block_number, event.timestamp);
        self.last_block_seen = std::cmp::max(self.last_block_seen, event.block_number);

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
            message: format!("M020 recorded: {} events processed", 1),
            data: HashMap::new(),
            execution_time_ms: elapsed,
            duplicates_found,
            missing_blocks,
        }
    }

    pub fn detect_duplicates(&self, events: &[MarketEvent]) -> Vec<String> {
        events
            .iter()
            .filter(|e| !self.event_registry.contains(&e.event_id))
            .map(|e| e.event_id.clone())
            .collect()
    }

    pub fn find_missing_blocks(&self) -> Vec<u64> {
        let mut missing = Vec::new();
        let mut blocks: Vec<u64> = self.block_timestamps.keys().cloned().collect();
        blocks.sort();

        for i in 0..blocks.len().saturating_sub(1) {
            let current = blocks[i];
            let next = blocks[i + 1];
            if next > current + 1 {
                for b in current + 1..next {
                    missing.push(b);
                }
            }
        }
        missing
    }

    pub fn execute(&mut self) -> ModuleResult {
        if !self.enabled {
            return ModuleResult {
                success: false,
                message: "Module disabled".to_string(),
                data: HashMap::new(),
                execution_time_ms: 0,
                duplicates_found: 0,
                missing_blocks: Vec::new(),
            };
        }

        let start = Instant::now();
        self.metrics.executions += 1;

        let missing_blocks = self.find_missing_blocks();

        let result = ModuleResult {
            success: true,
            message: format!("M020 executed: {} blocks tracked, {} missing", 
                self.block_timestamps.len(), missing_blocks.len()),
            data: HashMap::new(),
            execution_time_ms: start.elapsed().as_millis() as u64,
            duplicates_found: self.metrics.duplicates_detected,
            missing_blocks,
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
            r#"{{"executions":{},"successes":{},"failures":{},"health":{:.2},"duplicates_detected":{},"missing_blocks_detected":{}}}"#,
            self.metrics.executions,
            self.metrics.successes,
            self.metrics.failures,
            self.get_health(),
            self.metrics.duplicates_detected,
            self.metrics.missing_blocks_detected
        )
    }
}