#![allow(dead_code)]
// ==============================================================================
// ENGINE MODULES 21-25 - Regional Mesh Control & Network Resilience
// Part of the Allbright AISE Framework
// ==============================================================================

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH, Instant};
use std::collections::HashMap;

use std::sync::atomic::{AtomicBool, Ordering};

/// Cache-line aligned global halt flag managed out-of-band by the Security Captain Agent.
/// Expanded to a Specialist Supervisor Matrix for the 10 Intelligence Agents.
pub static SECURITY_CAPTAIN_HALT: AtomicBool = AtomicBool::new(false);
pub static FINANCIAL_SUPERVISOR_HALT: AtomicBool = AtomicBool::new(false);
pub static STRATEGIC_SUPERVISOR_HALT: AtomicBool = AtomicBool::new(false);
pub static ARCHITECTURE_SUPERVISOR_HALT: AtomicBool = AtomicBool::new(false);
pub static QUANTITATIVE_SUPERVISOR_HALT: AtomicBool = AtomicBool::new(false);
pub static RISK_SUPERVISOR_HALT: AtomicBool = AtomicBool::new(false);
pub static OPTIMIZATION_SUPERVISOR_HALT: AtomicBool = AtomicBool::new(false);
pub static PREDICTIVE_SUPERVISOR_HALT: AtomicBool = AtomicBool::new(false);
pub static RESEARCH_SUPERVISOR_HALT: AtomicBool = AtomicBool::new(false);
pub static INNOVATION_SUPERVISOR_HALT: AtomicBool = AtomicBool::new(false);
pub static COMPLIANCE_SUPERVISOR_HALT: AtomicBool = AtomicBool::new(false);

/// M21: Cross-Region State Sync (Gossip Latency)
/// Represents a piece of state information to be gossiped across regions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GossipedState {
    pub region_id: String,
    pub timestamp_ms: u64,
    pub payload: String, // e.g., "runner_status: active", "pool_shard_update: ..."
}

/// Simulates the latency of gossiping state between regions.
pub fn simulate_gossip_latency(sender_region: &str, receiver_region: &str, payload_size_bytes: usize) -> u64 {
    // Placeholder: In a real system, this would involve network topology,
    // current congestion, and historical latency data.
    let base_latency_ms = match (sender_region, receiver_region) {
        ("us-west-2", "eu-central-1") | ("eu-central-1", "us-west-2") => 80,
        ("us-west-2", "ap-southeast-1") | ("ap-southeast-1", "us-west-2") => 150,
        ("eu-central-1", "ap-southeast-1") | ("ap-southeast-1", "eu-central-1") => 120,
        _ => 10, // Intra-region or unknown
    };
    // Add a small overhead based on payload size
    base_latency_ms + (payload_size_bytes / 1000) as u64
}

/// M22: Validator Peering Health (P2P Mesh Monitoring)
/// Monitors the health and performance of connections to blockchain validators.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorMonitor {
    pub validator_id: String,
    pub avg_latency_ms: f64,
    pub packet_loss_pct: f64,
    pub last_seen_ms: u64,
    #[serde(skip)]
    pub last_update_instant: Option<Instant>,
    pub is_healthy: bool,
}

impl ValidatorMonitor {
    pub fn update_health(&mut self, current_latency_ms: f64, current_packet_loss_pct: f64, threshold_latency: f64, threshold_packet_loss: f64) {
        // Check Security Captain (Primary Infrastructure Halt)
        if SECURITY_CAPTAIN_HALT.load(Ordering::SeqCst) {
            self.is_healthy = false;
            return;
        }

        // Exponential Moving Average (EMA) for stability
        self.avg_latency_ms = (self.avg_latency_ms * 0.9) + (current_latency_ms * 0.1);
        self.packet_loss_pct = (self.packet_loss_pct * 0.9) + (current_packet_loss_pct * 0.1);
        
        self.is_healthy = self.avg_latency_ms < threshold_latency && self.packet_loss_pct < threshold_packet_loss;

        // Update monotonic reference for timing logic
        self.last_update_instant = Some(Instant::now());

        // Update wall-clock reference for reporting
        self.last_seen_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(self.last_seen_ms); // Fallback to last known time on clock error
    }
}

/// M23: UMECO Gateway Aggregation
/// Aggregates regional summaries into a global view.
pub fn aggregate_regional_data(regional_summaries: &[crate::allbright_c2::RegionalSummary]) -> HashMap<String, f64> {
    let mut global_metrics = HashMap::new();
    let mut total_yield = 0.0;
    let mut total_runners = 0;

    for summary in regional_summaries {
        total_yield += summary.total_yield_eth;
        total_runners += summary.active_runners;
    }

    global_metrics.insert("total_yield_eth".to_string(), total_yield);
    global_metrics.insert("total_active_runners".to_string(), total_runners as f64);
    global_metrics
}

/// M24: Latency Jitter Mitigation (Network Smoothing)
/// Calculates a jitter score from a series of latency measurements.
pub fn calculate_jitter_score(latencies_ms: &[u64]) -> f64 {
    if latencies_ms.len() < 2 { return 0.0; }
    let mut diffs: Vec<f64> = latencies_ms.windows(2).map(|w| (w[1] as f64 - w[0] as f64).abs()).collect();
    
    // Safe sort: Avoid unwrap() which panics on NaN
    diffs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    
    // Use interquartile range as a robust measure of jitter
    let q1_idx = diffs.len() / 4;
    let q3_idx = 3 * diffs.len() / 4;
    diffs[q3_idx] - diffs[q1_idx]
}

/// M25: Regional Failsafe Trigger (Partition Handling)
/// Detects if a region is isolated or experiencing severe connectivity issues.
pub fn detect_network_partition(region_id: &str, heartbeats: &HashMap<String, u64>, max_heartbeat_age_ms: u64) -> bool {
    // Check Architecture Supervisor for global network partitioning
    if ARCHITECTURE_SUPERVISOR_HALT.load(Ordering::SeqCst) {
        return true;
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    heartbeats.get(region_id).map_or(true, |&last_hb| (now > last_hb) && (now - last_hb) > max_heartbeat_age_ms)
}
