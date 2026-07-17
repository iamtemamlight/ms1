// Optimization Velocity Tracking Module
// Tracks improvement rates and calculates optimization velocity across 72 KPIs

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use parking_lot::RwLock;

pub struct OptimizationMetrics {
    pub total_cycles: AtomicU64,
    pub cycles_per_hour: AtomicU64,
    pub hourly_eth_projection: Arc<Mutex<f64>>,
    pub avg_gain_pct: Arc<Mutex<f64>>,
    pub optimization_velocity_trend: AtomicU64,
}

impl OptimizationMetrics {
    pub fn new() -> Self {
        Self {
            total_cycles: AtomicU64::new(0),
            cycles_per_hour: AtomicU64::new(0),
            hourly_eth_projection: Arc::new(Mutex::new(0.0)),
            avg_gain_pct: Arc::new(Mutex::new(0.0)),
            optimization_velocity_trend: AtomicU64::new(0),
        }
    }

    /// Calculate KPI gain percentage: (current - baseline) / baseline * 100
    pub fn calculate_kpi_gain(current: f64, baseline: f64) -> f64 {
        if baseline > 0.0 {
            ((current - baseline) / baseline * 100.0).clamp(-50.0, 100.0)
        } else {
            0.0
        }
    }

    /// Calculate optimization velocity from gains history
    pub fn calculate_velocity(
        &self,
        gains_last_30s: f64,
        gains_last_hour: f64,
        total_cycles: u64
    ) -> (u64, f64, &'static str) {
        let cycles_per_hour = total_cycles * 120; // 2 cycles per minute = 120/hour
        let hourly_rate = gains_last_hour * cycles_per_hour as f64;
        let acceleration = gains_last_30s - (gains_last_hour / 120.0);
        
        let trend = if acceleration > 0.1 {
            "ACCELERATING"
        } else if acceleration < -0.1 {
            "DECELERATING"
        } else {
            "STABLE"
        };

        (cycles_per_hour, hourly_rate, trend)
    }

    /// Update optimization cycle count
    pub fn record_optimization_cycle(&self, gain_pct: f64) {
        self.total_cycles.fetch_add(1, Ordering::Relaxed);
        let current_avg = *self.avg_gain_pct.lock().unwrap();
        let total = self.total_cycles.load(Ordering::Relaxed);
        // Exponential moving average for average gain
        let new_avg = (current_avg * 0.95) + (gain_pct * 0.05);
        *self.avg_gain_pct.lock().unwrap() = new_avg;
    }
}
