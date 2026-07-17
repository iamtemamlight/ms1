// ==============================================================================
// Predictive Transaction Batching (PTB)
// Mathematical Foundation: Batch_Optimality = Sum(Profit_i) - Sum(Gas_i) - Batch_Overhead
// Optimal_Batch_Size = argmax(Batch_Optimality)
// ==============================================================================

use std::sync::atomic::{AtomicU64, Ordering};
use std::collections::VecDeque;

#[repr(C, align(64))]
#[derive(Debug, Clone, Copy)]
pub struct Opportunity {
    pub profit: i64,
    pub gas_cost: u64,
    pub gas_limit: u64,
    pub priority: u32,
    pub pool_hash: u64,
    pub pair_hash: u64,
}

#[repr(C, align(64))]
pub struct BatchResult {
    pub indices: [u32; 64],
    pub count: u32,
    pub total_profit: i64,
    pub total_gas: u64,
    pub batch_overhead: u64,
    pub optimal_score: i64,
}

#[repr(C, align(64))]
pub struct PreComputedMatrix {
    pub profit_pairs: [[i64; 64]; 64],
    pub gas_pairs: [[u64; 64]; 64],
    pub interaction_penalty: [[u64; 64]; 64],
}

impl PreComputedMatrix {
    pub fn new() -> Self {
        Self {
            profit_pairs: [[0i64; 64]; 64],
            gas_pairs: [[0u64; 64]; 64],
            interaction_penalty: [[0u64; 64]; 64],
        }
    }

    #[inline(always)]
    pub fn update_pair(&mut self, i: usize, j: usize, profit_delta: i64, gas_delta: u64) {
        if i < 64 && j < 64 {
            self.profit_pairs[i][j] = profit_delta;
            self.gas_pairs[i][j] = gas_delta;
            self.interaction_penalty[i][j] = if profit_delta > 0 { 0 } else { u64::MAX };
        }
    }

    #[inline(always)]
    pub fn pair_profit(&self, i: usize, j: usize) -> i64 {
        if i < 64 && j < 64 {
            self.profit_pairs[i][j]
        } else {
            0
        }
    }

    #[inline(always)]
    pub fn pair_gas(&self, i: usize, j: usize) -> u64 {
        if i < 64 && j < 64 {
            self.gas_pairs[i][j]
        } else {
            0
        }
    }
}

impl Default for PreComputedMatrix {
    fn default() -> Self {
        Self::new()
    }
}

#[repr(C, align(64))]
pub struct BatchOptimizer {
    pub matrix: PreComputedMatrix,
    pub max_batch_size: u32,
    pub base_gas_overhead: u64,
    pub per_tx_overhead: u64,
    pub gas_price: AtomicU64,
    pub min_profit_threshold: AtomicU64,
    pub batch_counter: AtomicU64,
}

impl BatchOptimizer {
    pub fn new(max_batch_size: u32, base_gas_overhead: u64, per_tx_overhead: u64) -> Self {
        Self {
            matrix: PreComputedMatrix::new(),
            max_batch_size,
            base_gas_overhead,
            per_tx_overhead,
            gas_price: AtomicU64::new(30_000_000_000),
            min_profit_threshold: AtomicU64::new(10_000_000_000_000),
            batch_counter: AtomicU64::new(0),
        }
    }

    #[inline(always)]
    pub fn set_gas_price(&self, price: u64) {
        self.gas_price.store(price, Ordering::SeqCst);
    }

    #[inline(always)]
    pub fn set_min_profit(&self, profit: i64) {
        self.min_profit_threshold.store(profit as u64, Ordering::SeqCst);
    }

    #[inline(always)]
    pub fn gas_price(&self) -> u64 {
        self.gas_price.load(Ordering::SeqCst)
    }

    #[inline(always)]
    pub fn min_profit(&self) -> i64 {
        self.min_profit_threshold.load(Ordering::SeqCst) as i64
    }

    #[inline(always)]
    pub fn estimate_gas_cost(&self, opportunity: &Opportunity) -> u64 {
        let gas_price = self.gas_price();
        opportunity.gas_limit.saturating_mul(gas_price / 1_000_000_000)
    }

    #[inline(always)]
    pub fn batch_overhead(&self, count: u32) -> u64 {
        self.base_gas_overhead + (self.per_tx_overhead * count as u64)
    }

    pub fn find_optimal_batch(&self, opportunities: &[Opportunity]) -> BatchResult {
        let n = opportunities.len().min(64);
        if n == 0 {
            return BatchResult {
                indices: [0; 64],
                count: 0,
                total_profit: 0,
                total_gas: 0,
                batch_overhead: 0,
                optimal_score: 0,
            };
        }

        let mut best_score: i64 = 0;
        let mut best_mask: u64 = 0;
        let total_ops = 1u64 << n;

        for mask in 0..total_ops {
            let mut profit: i64 = 0;
            let mut gas: u64 = 0;
            let mut valid = true;

            for i in 0..n {
                if (mask >> i) & 1 != 0 {
                    profit = profit.saturating_add(opportunities[i].profit);
                    gas = gas.saturating_add(self.estimate_gas_cost(&opportunities[i]));

                    for j in (i + 1)..n {
                        if (mask >> j) & 1 != 0 {
                            let interaction = self.matrix.interaction_penalty[i][j];
                            if interaction == u64::MAX {
                                valid = false;
                                break;
                            }
                            gas = gas.saturating_add(interaction);
                        }
                    }

                    if !valid {
                        break;
                    }
                }
            }

            if !valid {
                continue;
            }

            let overhead = self.batch_overhead(mask.count_ones());
            let score = profit - (gas + overhead) as i64;

            if score > best_score {
                best_score = score;
                best_mask = mask;
            }
        }

        let mut indices = [0u32; 64];
        let mut count = 0;
        let mut total_profit: i64 = 0;
        let mut total_gas: u64 = 0;

        for i in 0..n {
            if (best_mask >> i) & 1 != 0 {
                indices[count] = i as u32;
                count += 1;
                total_profit = total_profit.saturating_add(opportunities[i].profit);
                total_gas = total_gas.saturating_add(self.estimate_gas_cost(&opportunities[i]));
            }
        }

        let batch_overhead = self.batch_overhead(count as u32);

        self.batch_counter.fetch_add(1, Ordering::SeqCst);

        BatchResult {
            indices,
            count: count as u32,
            total_profit,
            total_gas,
            batch_overhead,
            optimal_score: best_score,
        }
    }

    pub fn batch_count(&self) -> u64 {
        self.batch_counter.load(Ordering::SeqCst)
    }
}

#[repr(C, align(64))]
pub struct BatchingWindow {
    pub window_ns: u64,
    pub max_opportunities: usize,
    pub pending: VecDeque<Opportunity>,
    pub last_flush: AtomicU64,
    pub flush_count: AtomicU64,
}

impl BatchingWindow {
    pub fn new(window_ns: u64, max_opportunities: usize) -> Self {
        Self {
            window_ns,
            max_opportunities,
            pending: VecDeque::with_capacity(max_opportunities),
            last_flush: AtomicU64::new(0),
            flush_count: AtomicU64::new(0),
        }
    }

    #[inline(always)]
    pub fn should_flush(&self, now_ns: u64) -> bool {
        let last = self.last_flush.load(Ordering::Acquire);
        now_ns.saturating_sub(last) >= self.window_ns || self.pending.len() >= self.max_opportunities
    }

    pub fn add(&mut self, opp: Opportunity) {
        if self.pending.len() < self.max_opportunities {
            self.pending.push_back(opp);
        }
    }

    pub fn drain(&mut self) -> Vec<Opportunity> {
        self.last_flush.store(instant_now_ns(), Ordering::Release);
        self.flush_count.fetch_add(1, Ordering::SeqCst);
        self.pending.drain(..).collect()
    }

    pub fn len(&self) -> usize {
        self.pending.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }
}

#[inline(always)]
pub fn instant_now_ns() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_optimizer_creation() {
        let opt = BatchOptimizer::new(64, 21000, 21000);
        assert_eq!(opt.max_batch_size, 64);
    }

    #[test]
    fn test_find_optimal_batch_empty() {
        let opt = BatchOptimizer::new(64, 21000, 21000);
        let result = opt.find_optimal_batch(&[]);
        assert_eq!(result.count, 0);
    }

    #[test]
    fn test_find_optimal_batch_single() {
        let opt = BatchOptimizer::new(64, 21000, 21000);
        let opps = vec![Opportunity {
            profit: 100_000_000_000_000_000,
            gas_cost: 50_000_000_000,
            gas_limit: 150000,
            priority: 1,
            pool_hash: 0xABCD,
            pair_hash: 0x1234,
        }];
        let result = opt.find_optimal_batch(&opps);
        assert_eq!(result.count, 1);
    }

    #[test]
    fn test_batching_window() {
        let mut window = BatchingWindow::new(1_000_000_000, 64);
        assert!(window.is_empty());
        window.add(Opportunity {
            profit: 100,
            gas_cost: 10,
            gas_limit: 21000,
            priority: 1,
            pool_hash: 1,
            pair_hash: 2,
        });
        assert_eq!(window.len(), 1);
        let drained = window.drain();
        assert_eq!(drained.len(), 1);
        assert!(window.is_empty());
    }

    #[test]
    fn test_precomputed_matrix() {
        let mut matrix = PreComputedMatrix::new();
        matrix.update_pair(0, 1, 500, 100);
        assert_eq!(matrix.pair_profit(0, 1), 500);
        assert_eq!(matrix.pair_gas(0, 1), 100);
    }
}
