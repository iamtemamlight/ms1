// ==============================================================================
// UPGRADE4 REWRITE: M007 Gas Price Oracle
// Modules 4-5: Atomic CLZ density counting + Pre-baked Bid_Matrix
// ==============================================================================

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

// -----------------------------------------------------------------------------
// Module 4: Bitwise Shift Density Counting
// -----------------------------------------------------------------------------

/// Pre-baked elasticity constant. S_elasticity = log2(elasticity_factor).
/// Network-specific, set at initialization.
pub const S_ELASTICITY: u32 = 3;

/// CLZ-based gas momentum predictor.
/// P_base(t+1) = P_base(t) + (P_base(t) >> (S_elasticity + Clz(Delta_D)))
static MEMPOOL_DENSITY: AtomicU64 = AtomicU64::new(0);

#[inline(always)]
pub fn increment_mempool_density() {
    MEMPOOL_DENSITY.fetch_add(1, Ordering::Relaxed);
}

#[inline(always)]
pub fn take_mempool_density() -> u64 {
    MEMPOOL_DENSITY.swap(0, Ordering::AcqRel)
}

/// Estimate next block base fee using CLZ density counting.
/// Delta_D = current mempool density (number of txs in last 100μs window)
/// Clz(Delta_D) gives the leading zeros, which maps density to a power-of-two shift.
#[inline(always)]
pub fn estimate_next_base_fee(current_base: u64, density: u64) -> u64 {
    if density == 0 {
        return current_base;
    }
    let clz = density.leading_zeros();
    let shift = (S_ELASTICITY + clz).min(63);
    current_base + (current_base >> shift)
}

// -----------------------------------------------------------------------------
// Module 5: Pre-Baked Bid Matrix
// -----------------------------------------------------------------------------

/// Pre-computed competitive priority fee tiers indexed by density.
/// Built at block boundaries, accessed in 2 cycles on hot path.
#[repr(C, align(64))]
#[derive(Debug)]
pub struct BidMatrix {
    /// 256 density-indexed priority fees in wei
    pub table: [u64; 256],
    /// Static 1-wei buffer to front-run highest competitor
    pub delta_b: u64,
}

impl BidMatrix {
    /// Build bid matrix from observed competitor data (background only).
    pub fn build(base_tiers: &[u64; 256]) -> Self {
        let mut table = [0u64; 256];
        for i in 0..256 {
            table[i] = base_tiers[i];
        }
        Self { table, delta_b: 1 }
    }

    /// Lookup priority fee: P_priority = Bid_Matrix[Density_Index] + Delta_b
    /// Exactly 1 array index + 1 add. Zero math on hot path.
    #[inline(always)]
    pub fn lookup(&self, density: u64) -> u64 {
        let clz = density.leading_zeros();
        let idx = (63 - clz) as usize;
        let idx = idx.min(255);
        self.table[idx] + self.delta_b
    }
}

// -----------------------------------------------------------------------------
// Legacy GasPriceOracle — reduced to non-critical-path wrapper
// -----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct GasPrice {
    pub slow: u64,
    pub standard: u64,
    pub fast: u64,
    pub instant: u64,
}

impl Default for GasPrice {
    fn default() -> Self {
        Self {
            slow: 20_000_000_000,
            standard: 25_000_000_000,
            fast: 30_000_000_000,
            instant: 45_000_000_000,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GasStrategy {
    Slow,
    Standard,
    Fast,
    Instant,
}

#[derive(Debug)]
pub struct GasPriceOracle {
    pub current_price: GasPrice,
    pub bid_matrix: BidMatrix,
}

impl GasPriceOracle {
    pub fn new() -> Self {
        let base_tiers = [25_000_000_000u64; 256];
        Self {
            current_price: GasPrice::default(),
            bid_matrix: BidMatrix::build(&base_tiers),
        }
    }

    /// Hot-path gas price lookup.
    #[inline(always)]
    pub fn get_gas_price(&self, strategy: GasStrategy) -> u64 {
        match strategy {
            GasStrategy::Slow => self.current_price.slow,
            GasStrategy::Standard => self.current_price.standard,
            GasStrategy::Fast => self.current_price.fast,
            GasStrategy::Instant => self.current_price.instant,
        }
    }

    /// Hot-path next-block base fee estimation (Module 4).
    #[inline(always)]
    pub fn estimate_next_base_fee_hot(&self, current_base: u64) -> u64 {
        let density = take_mempool_density();
        estimate_next_base_fee(current_base, density)
    }

    /// Hot-path priority fee estimation (Module 5).
    #[inline(always)]
    pub fn estimate_priority_fee_hot(&self, density: u64) -> u64 {
        self.bid_matrix.lookup(density)
    }

    /// Hot-path total gas cost: G * P_total
    #[inline(always)]
    pub fn estimate_total_gas_cost(&self, gas_limit: u64, base_fee: u64, priority_fee: u64) -> u64 {
        gas_limit * (base_fee + priority_fee)
    }
}

// -----------------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_next_base_fee() {
        let base = 30_000_000_000;
        let next = estimate_next_base_fee(base, 100);
        assert!(next >= base);
    }

    #[test]
    fn test_bid_matrix_lookup() {
        let mut tiers = [10_000_000_000u64; 256];
        tiers[0] = 1_000_000_000;
        let matrix = BidMatrix::build(&tiers);
        let fee = matrix.lookup(0);
        assert_eq!(fee, 1_000_000_001);
    }

    #[test]
    fn test_mempool_density() {
        increment_mempool_density();
        increment_mempool_density();
        let d = take_mempool_density();
        assert_eq!(d, 2);
    }
}
