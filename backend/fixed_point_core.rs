#![allow(dead_code)]

// ==============================================================================
// UPGRADE4: Fixed-Point Core — Modules 1, 2, 3
// Sub-micron exchange primitives and branchless execution masks
// ==============================================================================

use std::sync::atomic::{AtomicU64, Ordering};

// -----------------------------------------------------------------------------
// Module 1: Bitwise Shifting Reciprocals (Zero-Multiplication)
// -----------------------------------------------------------------------------

/// Pre-computed pool shift/mask pair, digitized at block boundary.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PoolShiftState {
    /// Power-of-two shift amount: S_pool ≈ log2(X / (Y * γ))
    pub s_pool: u32,
    /// Max-swap mask enforcing pool reserve limits
    pub m_max_swap: u64,
    /// Cached reserve ratio for diagnostics (non-critical path only)
    pub reserve_ratio_scaled: u64,
}

impl PoolShiftState {
    pub const fn new(s_pool: u32, m_max_swap: u64) -> Self {
        Self {
            s_pool,
            m_max_swap,
            reserve_ratio_scaled: 0,
        }
    }

    /// Compute swap output: ΔY = (ΔX >> S_pool) & M_max_swap
    /// Exactly 1 shift + 1 AND. Zero division. Zero multiplication.
    #[inline(always)]
    pub fn compute_output(&self, delta_x: u64) -> u64 {
        (delta_x >> self.s_pool) & self.m_max_swap
    }
}

// -----------------------------------------------------------------------------
// Module 2: 0-Cycle Differential Delta Scaling (Pre-Computed Step Array)
// -----------------------------------------------------------------------------

/// Pre-baked optimal input array indexed by bit-shifted price delta.
/// Computed once per block in background, read once per tick in hot path.
#[repr(C, align(64))]
#[derive(Debug, Clone)]
pub struct StepArray {
    /// 2^16 pre-computed optimal flash-loan inputs
    pub table: [u64; 65536],
    /// Granularity shift: ΔR >> S_granularity forms clean index
    pub s_granularity: u32,
    /// Baseline reserve R0 used to generate the table (diagnostics only)
    pub baseline_reserve: u64,
}

impl StepArray {
    /// Build a step array from baseline reserve R0.
    /// Background/idle operation — not on hot path.
    pub fn build(baseline_reserve: u64, s_granularity: u32) -> Self {
        let mut table = [0u64; 65536];
        let r0 = baseline_reserve.max(1);
        for i in 0..65536usize {
            let delta_r = (i as u64) << s_granularity;
            // Optimal flash-loan input: X_opt = sqrt(R0(R0 + ΔR)) − R0.
            // Computed exactly here (background/idle path) — no linearization error,
            // and the closed form is correct at ΔR = 0 (X_opt = 0).
            let x_opt = Self::integer_sqrt(r0.wrapping_mul(r0.wrapping_add(delta_r)))
                .wrapping_sub(r0);
            table[i] = x_opt;
        }
        Self {
            table,
            s_granularity,
            baseline_reserve: r0,
        }
    }

    /// Lookup optimal input: X_opt = V_pre_computed[ΔR >> S_granularity]
    /// Exactly 1 shift + 1 array index. Zero math on hot path.
    #[inline(always)]
    pub fn lookup(&self, delta_r: u64) -> u64 {
        let idx = (delta_r >> self.s_granularity) as usize;
        // Safety: idx is u64 shifted by at least 1, max 2^63, but table is 65536.
        // The caller must ensure S_granularity is chosen so delta_r >> S_granularity < 65536.
        // For UPGRADE4 this is guaranteed by the pre-computation block.
        if idx < self.table.len() {
            self.table[idx]
        } else {
            self.table[65535]
        }
    }

    /// Integer square root via binary search (background only).
    #[inline(always)]
    fn integer_sqrt(n: u64) -> u64 {
        let mut x = n;
        let mut root = 0u64;
        let mut bit = 1u64 << 62;
        while bit > 0 {
            let candidate = root | bit;
            if candidate <= x {
                x -= candidate;
                root = (root >> 1) | bit;
            } else {
                root >>= 1;
            }
            bit >>= 2;
        }
        root
    }

    
}

// -----------------------------------------------------------------------------
// Module 3: Simultaneous Double-Predicate Masking
// -----------------------------------------------------------------------------

/// Combined execution mask for branchless profit + gas validation.
/// If Net_Profit < 0, Execution_Mask == 0 and all downstream fields zero out.
#[derive(Debug, Clone, Copy)]
pub struct ExecutionMask {
    pub mask: u64,
}

impl ExecutionMask {
    /// Net_Profit = Gross_Revenue - Total_Gas_Cost (must be precomputed as i64)
    /// Execution_Mask = ~(Net_Profit >> 63)
    #[inline(always)]
    pub fn from_net_profit(net_profit_i64: i64) -> Self {
        let shifted = (net_profit_i64 >> 63) as u64;
        Self {
            mask: !shifted,
        }
    }

    /// Zero-cycle constructor for known-profitable path (bypass calculation).
    #[inline(always)]
    pub fn all_ones() -> Self {
        Self { mask: u64::MAX }
    }

    #[inline(always)]
    pub fn is_executable(&self) -> bool {
        self.mask != 0
    }
}

/// Branchless payload size zeroing.
/// Payload_Size = Execution_Mask & Payload_Template_Size
#[inline(always)]
pub fn apply_execution_mask(payload_size: u64, mask: u64) -> u64 {
    payload_size & mask
}

// -----------------------------------------------------------------------------
// Atomic hot-path counters (Module 4 pre-computation support)
// -----------------------------------------------------------------------------

static MEMPOOL_DENSITY: AtomicU64 = AtomicU64::new(0);

/// Increment mempool transaction density counter (called from packet receiver).
#[inline(always)]
pub fn increment_mempool_density() {
    MEMPOOL_DENSITY.fetch_add(1, Ordering::Relaxed);
}

/// Read and reset mempool density for block-boundary pre-computation.
#[inline(always)]
pub fn take_mempool_density() -> u64 {
    MEMPOOL_DENSITY.swap(0, Ordering::AcqRel)
}

// -----------------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_shift_output() {
        let state = PoolShiftState::new(4, 0xFFFFFFFFFFFFFFFF);
        assert_eq!(state.compute_output(0xFF), 0xF);
        assert_eq!(state.compute_output(0), 0);
    }

    #[test]
    fn test_execution_mask_positive() {
        let mask = ExecutionMask::from_net_profit(100i64);
        assert!(mask.is_executable());
        assert_eq!(apply_execution_mask(512, mask.mask), 512);
    }

    #[test]
    fn test_execution_mask_negative() {
        let mask = ExecutionMask::from_net_profit(-100i64);
        assert!(!mask.is_executable());
        assert_eq!(apply_execution_mask(512, mask.mask), 0);
    }

    #[test]
    fn test_step_array_lookup() {
        let arr = StepArray::build(1_000_000_000, 8);
        // ΔR = 0 -> X_opt = sqrt(R0*R0) - R0 = R0 - R0 = 0 (exact, not approximate)
        assert_eq!(arr.lookup(0), 0);
        // Non-zero ΔR should give positive X_opt
        let val = arr.lookup(100);
        assert!(val > 0);
    }

    #[test]
    fn test_mempool_density() {
        increment_mempool_density();
        increment_mempool_density();
        let d = take_mempool_density();
        assert_eq!(d, 2);
        assert_eq!(take_mempool_density(), 0);
    }
}
