#![allow(dead_code)]
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FleetTelemetry {
    pub runner_id: String,
    pub timestamp_ms: u64,
    pub metrics: std::collections::HashMap<String, f64>,
}
