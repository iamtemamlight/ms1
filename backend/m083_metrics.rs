#![allow(dead_code)]
// ==============================================================================
// MODULE 61: APEX METRICS AGGREGATOR
// Purpose: Aggregates multi-runner metrics into fleet-wide KPIs for the AISE system.
// Dependencies: TELEMETRY_COUNT, WIN_RATE_EMA, LAST_GAS_ADJUSTED_PROFIT_SCALED
// Specialist AI Agent Role: Reports aggregated fleet KPIs to the Alpha-Copilot for optimization.
// ==============================================================================

use std::sync::atomic::{AtomicU64, Ordering};

static FLEET_TRADES_TOTAL: AtomicU64 = AtomicU64::new(0);
static FLEET_TRADES_SUCCESSFUL: AtomicU64 = AtomicU64::new(0);
static FLEET_PROFIT_SCALED: AtomicU64 = AtomicU64::new(0);

#[inline(always)]
pub fn aggregate_fleet_kpis() -> FleetKpis {
    let total = FLEET_TRADES_TOTAL.load(Ordering::SeqCst);
    let successful = FLEET_TRADES_SUCCESSFUL.load(Ordering::SeqCst);
    let profit = FLEET_PROFIT_SCALED.load(Ordering::SeqCst);
    
    FleetKpis {
        total_trades: total,
        successful_trades: successful,
        win_rate_pct: if total > 0 { (successful * 10000) / total } else { 0 },
        profit_scaled: profit,
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FleetKpis {
    pub total_trades: u64,
    pub successful_trades: u64,
    pub win_rate_pct: u64,
    pub profit_scaled: u64,
}
