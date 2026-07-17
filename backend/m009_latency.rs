#![allow(dead_code)]
// Module for High-Precision Latency Tracking
// Migrated from monolith.rs for production modularity.

use std::sync::atomic::{AtomicU64, Ordering};
use std::collections::VecDeque;
use serde::{Serialize, Deserialize};
use once_cell::sync::Lazy;
use chrono::Utc;

pub static LAST_LOOP_NS: AtomicU64 = AtomicU64::new(19800);
pub static TARGET_LATENCY_NS: u64 = 20000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageLatency {
    pub stage: &'static str,
    pub latency_ns: u64,
    pub timestamp: i64,
}

static DETECTION_NS: AtomicU64 = AtomicU64::new(0);
static DECISION_NS: AtomicU64 = AtomicU64::new(0);
static SIMULATION_NS: AtomicU64 = AtomicU64::new(0);
static SIGNING_NS: AtomicU64 = AtomicU64::new(0);
static BUNDLE_NS: AtomicU64 = AtomicU64::new(0);
static RELAY_NS: AtomicU64 = AtomicU64::new(0);
static INCLUSION_NS: AtomicU64 = AtomicU64::new(0);

static STAGE_BUFFER: Lazy<std::sync::Mutex<VecDeque<StageLatency>>> = Lazy::new(|| std::sync::Mutex::new(VecDeque::with_capacity(1024)));

pub fn record_latency(ns: u64) {
    LAST_LOOP_NS.store(ns, Ordering::SeqCst);
}

pub fn get_current_deflection() -> f64 {
    (LAST_LOOP_NS.load(Ordering::SeqCst) as f64 / TARGET_LATENCY_NS as f64) - 1.0
}

pub fn record_stage(stage: &'static str, ns: u64) {
    match stage {
        "detection" => DETECTION_NS.store(ns, Ordering::SeqCst),
        "decision" => DECISION_NS.store(ns, Ordering::SeqCst),
        "simulation" => SIMULATION_NS.store(ns, Ordering::SeqCst),
        "signing" => SIGNING_NS.store(ns, Ordering::SeqCst),
        "bundle" => BUNDLE_NS.store(ns, Ordering::SeqCst),
        "relay" => RELAY_NS.store(ns, Ordering::SeqCst),
        "inclusion" => INCLUSION_NS.store(ns, Ordering::SeqCst),
        _ => {}
    }
    if let Ok(mut buf) = STAGE_BUFFER.lock() {
        buf.push_back(StageLatency { stage, latency_ns: ns, timestamp: Utc::now().timestamp() });
        if buf.len() > 1024 {
            buf.pop_front();
        }
    }
}

pub fn get_stage_latency(stage: &str) -> u64 {
    match stage {
        "detection" => DETECTION_NS.load(Ordering::SeqCst),
        "decision" => DECISION_NS.load(Ordering::SeqCst),
        "simulation" => SIMULATION_NS.load(Ordering::SeqCst),
        "signing" => SIGNING_NS.load(Ordering::SeqCst),
        "bundle" => BUNDLE_NS.load(Ordering::SeqCst),
        "relay" => RELAY_NS.load(Ordering::SeqCst),
        "inclusion" => INCLUSION_NS.load(Ordering::SeqCst),
        _ => 0,
    }
}

pub fn get_recent_stages(limit: usize) -> Vec<StageLatency> {
    if let Ok(buf) = STAGE_BUFFER.lock() {
        buf.iter().rev().take(limit).cloned().collect()
    } else {
        Vec::new()
    }
}

pub fn reset_stages() {
    DETECTION_NS.store(0, Ordering::SeqCst);
    DECISION_NS.store(0, Ordering::SeqCst);
    SIMULATION_NS.store(0, Ordering::SeqCst);
    SIGNING_NS.store(0, Ordering::SeqCst);
    BUNDLE_NS.store(0, Ordering::SeqCst);
    RELAY_NS.store(0, Ordering::SeqCst);
    INCLUSION_NS.store(0, Ordering::SeqCst);
    if let Ok(mut buf) = STAGE_BUFFER.lock() {
        buf.clear();
    }
}
