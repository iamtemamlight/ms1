// ==============================================================================
// CONTINUUM OPTIMIZATION ENGINE - Continuous 72-KPI Auto-Optimization Loop
// Purpose: Real-time optimization gains tracking and dimension adjustment for ALL 72 KPIs
// ==============================================================================

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

// Re-export types from other modules
use crate::kpi_telemetry::{KpiTelemetryCollector, MeasuredKpi, SubSystem};
use crate::m054_auto_optimizer::{AutoOptimizationAgent, SubcategoryMeasurements};

/// 30-second window aggregation for optimization gains
pub struct OptimizationContinuum {
    profit_30s_current: Arc<Mutex<f64>>,
    profit_30s_previous: Arc<Mutex<f64>>,
    kpi_gains: [Arc<Mutex<f64>>; 72],
    pillar_gains: [Arc<Mutex<f64>>; 6],
    total_cycles: AtomicU64,
    last_optimization_ms: AtomicU64,
    baseline_30s_profit: f64,
}

impl OptimizationContinuum {
    pub fn new() -> Self {
        Self {
            profit_30s_current: Arc::new(Mutex::new(0.0)),
            profit_30s_previous: Arc::new(Mutex::new(0.0)),
            kpi_gains: core::array::from_fn(|_| Arc::new(Mutex::new(0.0))),
            pillar_gains: core::array::from_fn(|_| Arc::new(Mutex::new(0.0))),
            total_cycles: AtomicU64::new(0),
            last_optimization_ms: AtomicU64::new(0),
            baseline_30s_profit: 0.0347, // 100 ETH / 2880 = 0.0347 ETH per 30s
        }
    }

    /// Process all 72 KPIs every 30s and calculate gains
    pub fn process_kpi_gains(&self, telemetry: &KpiTelemetryCollector) {
        // Get current timestamp
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        // Update 30s profit window
        let prev = *self.profit_30s_previous.lock().unwrap();
        let curr = *self.profit_30s_current.lock().unwrap();
        
        // Calculate profit gap for this window
        let profit_gap = if curr > 0.0 {
            (curr - self.baseline_30s_profit) / self.baseline_30s_profit
        } else {
            -1.0
        };
        
        // Aggregate gains for each pillar from its KPIs
        for pillar_idx in 0..6 {
            let pillar = match pillar_idx {
                0 => SubSystem::Profit,
                1 => SubSystem::Velocity,
                2 => SubSystem::Shield,
                3 => SubSystem::Efficiency,
                4 => SubSystem::Continuity,
                5 => SubSystem::Growth,
                _ => continue,
            };
            
            let (start_kpi, end_kpi) = pillar.kpi_range();
            let mut sum_gain = 0.0;
            let mut count = 0;
            
            for kpi_id in start_kpi..=end_kpi {
                if let Some(kpi) = telemetry.get_kpi(kpi_id) {
                    // Gain = (current - baseline) / baseline
                    let gain_pct = if kpi.baseline_value > 0.0 {
                        ((kpi.measured_value - kpi.baseline_value) / kpi.baseline_value)
                    } else {
                        kpi.measured_value / 100.0
                    };
                    *self.kpi_gains[kpi_id as usize].lock().unwrap() = gain_pct.max(-1.0).min(1.0);
                    sum_gain += gain_pct;
                    count += 1;
                }
            }
            
            // Store average pillar gain
            if count > 0 {
                *self.pillar_gains[pillar_idx].lock().unwrap() = sum_gain / count as f64;
            }
        }
        
        self.last_optimization_ms.store(now_ms, Ordering::Relaxed);
    }

    /// Get KPI gain percentage (0-100 scale)
    pub fn get_kpi_gain(&self, kpi_id: usize) -> f64 {
        *self.kpi_gains[kpi_id].lock().unwrap() * 100.0
    }

    /// Get pillar gain percentage (0-100 scale)
    pub fn get_pillar_gain(&self, pillar_idx: usize) -> f64 {
        *self.pillar_gains[pillar_idx].lock().unwrap() * 100.0
    }

    /// Drive optimizer adjustments from KPI gains
    pub fn drive_optimizer_adjustments(
        &self,
        agent: &mut AutoOptimizationAgent,
        telemetry: &KpiTelemetryCollector
    ) -> Vec<(String, f64)> {
        let mut adjustments = Vec::new();
        
        // Process each KPI and map to dimension adjustments
        for kpi_id in 0..72 {
            let gain = *self.kpi_gains[kpi_id].lock().unwrap();
            
            // Only adjust if there's significant deviation (>2% gain/loss)
            if gain.abs() > 0.02 {
                let (dim_id, adjustment_factor) = agent.get_dimension_adjustment_for_kpi(kpi_id, gain);
                
                // Record this as a KPI-driven adjustment
                let deviation_scaled = (gain.abs() * 100.0) as u64;
                agent.update_kpi_deviation(kpi_id, deviation_scaled);
                
                // Map dimension ID to meaningful name
                let dim_name = self.dimension_id_to_name(dim_id);
                adjustments.push((dim_name, adjustment_factor));
            }
        }
        
        adjustments
    }

    fn dimension_id_to_name(&self, dim_id: usize) -> String {
        match dim_id {
            0 => "CORRIDOR_WIDTH".into(),
            1 => "REGION_ROUTING".into(),
            2 => "PAIR_SELECTION".into(),
            3 => "MODE_REGIME".into(),
            4 => "FLASH_LOAN_SIZE".into(),
            5 => "COMPETITOR_RESPONSE".into(),
            6 => "REGIONAL_PARAMS".into(),
            7 => "SOLVER_TOLERANCE".into(),
            8 => "SHIELD_ROUTING".into(),
            9 => "CAPITAL_ALLOCATION".into(),
            10 => "CAP_EFFICIENCY".into(),
            11 => "MULTI_HOP".into(),
            12 => "ARB_TYPE".into(),
            13 => "POOL_TIER".into(),
            14 => "GAS_CYCLE".into(),
            15 => "BRIBE_BIAS".into(),
            16 => "POOL_DISPATCH".into(),
            17 => "SYNCHRONIZATION".into(),
            18 => "SESSION_CONT".into(),
            19 => "BUNDLE_SIZE".into(),
            20 => "CHAIN_SELECTION".into(),
            21 => "GAS_CYCLE_PHASE".into(),
            22 => "RUNNER_CAPACITY".into(),
            23 => "JIT_LIQUIDITY".into(),
            24 => "SOLVER_ACCURACY".into(),
            _ => "UNKNOWN".into(),
        }
    }

    /// Update 30s profit window with new measurement
    pub fn record_profit_30s(&self, profit_eth: f64) {
        let prev = *self.profit_30s_current.lock().unwrap();
        *self.profit_30s_previous.lock().unwrap() = prev;
        *self.profit_30s_current.lock().unwrap() = profit_eth;
    }

    /// Get current profit gap percentage
    pub fn get_profit_gap_30s(&self) -> f64 {
        let curr = *self.profit_30s_current.lock().unwrap();
        ((self.baseline_30s_profit - curr) / self.baseline_30s_profit).max(0.0) * 100.0
    }

    /// Check if optimization should trigger based on 30s window
    pub fn should_optimize(&self) -> bool {
        self.get_profit_gap_30s() > 10.0 // 10% deficit threshold
    }
}

impl Default for OptimizationContinuum {
    fn default() -> Self {
        Self::new()
    }
}