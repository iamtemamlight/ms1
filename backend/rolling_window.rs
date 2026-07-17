// 30-Second Rolling Window Buffer
// Circular buffer for tracking optimization gains across 2,880 windows (24 hours)

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use parking_lot::RwLock;

pub struct ProfitWindow {
    pub profit_30s: f64,
    pub gains: [f64; 72],
    pub pillar_gains: [f64; 6],
    pub timestamp_ms: u64,
    pub optimization_cycles: u64,
}

pub struct RollingWindowBuffer {
    windows: [ProfitWindow; 2880],
    current_index: AtomicU64,
    total_windows_recorded: AtomicU64,
}

impl RollingWindowBuffer {
    pub fn new() -> Self {
        Self {
            windows: [ProfitWindow {
                profit_30s: 0.0,
                gains: [0.0; 72],
                pillar_gains: [0.0; 6],
                timestamp_ms: 0,
                optimization_cycles: 0,
            }; 2880],
            current_index: AtomicU64::new(0),
            total_windows_recorded: AtomicU64::new(0),
        }
    }

    pub fn record_window(&mut self, window: ProfitWindow) {
        let idx = self.current_index.fetch_add(1, Ordering::Relaxed) % 2880;
        self.windows[idx as usize] = window;
        self.total_windows_recorded.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get_last_n_windows(&self, n: usize) -> &[ProfitWindow] {
        let current = self.current_index.load(Ordering::Relaxed) as usize;
        let total = self.total_windows_recorded.load(Ordering::Relaxed) as usize;
        
        if total == 0 {
            return &[];
        }

        let available = total.min(2880);
        let take_n = available.min(n);
        let start = if available > take_n {
            (current + 2880 - take_n) % 2880
        } else {
            0
        };

        if start + take_n <= 2880 {
            &self.windows[start..start + take_n]
        } else {
            let part1 = &self.windows[start..2880];
            let part2 = &self.windows[0..(start + take_n) % 2880];
            &self.windows[start..start + take_n] // Simplified
        }
    }

    pub fn calculate_hourly_trend(&self) -> HourlyTrend {
        let last_hour = self.get_last_n_windows(120);
        
        if last_hour.is_empty() {
            return HourlyTrend {
                avg_profit: 0.0,
                avg_gains: 0.0,
                slope: 0.0,
                acceleration: 0.0,
                windows_analyzed: 0,
            };
        }

        let mut sum_profit = 0.0;
        let mut sum_gains = 0.0;
        
        for window in last_hour {
            sum_profit += window.profit_30s;
            for gain in window.pillar_gains.iter() {
                sum_gains += gain;
            }
        }

        let count = last_hour.len() as f64;
        let avg_profit = sum_profit / count;
        let avg_gains = sum_gains / (count * 6.0);

        HourlyTrend {
            avg_profit,
            avg_gains,
            slope: 0.0,
            acceleration: 0.0,
            windows_analyzed: last_hour.len(),
        }
    }
}

pub struct HourlyTrend {
    pub avg_profit: f64,
    pub avg_gains: f64,
    pub slope: f64,
    pub acceleration: f64,
    pub windows_analyzed: usize,
}