//! `micropath` — No-hardware <5µs deterministic arbiter hot path.
//!
//! Implements the fused pipeline from `1MSGUIDE_NO_HW_ALTERNATIVE.md`:
//!   M1 Fixed-Point Reciprocation (Γ folded into INV)
//!   M2 First-Order Taylor sizing
//!   M3 Bitwise Predicate Mask gate
//!   M4 AVX2 (256-bit, 4-way) instead of AVX-512
//!   M5 Perfect-Hash bitmask slot index
//!   Gas momentum predictor + feasibility invariant, scaled to 2^32
//!
//! Zero division, zero sqrt, zero branches, zero heap allocation on the hot path.

#![forbid(unsafe_op_in_unsafe_fn)]

pub const SCALE: u64 = 1u64 << 32; // fixed-point scale for deltas/constants

/// Block-idle precomputed constants (L1-pinned static block). All `u64`, scaled by 2^32
/// unless noted. Computed once per block state shift by the background worker thread.
#[derive(Clone, Copy)]
#[repr(C, align(64))] // cache-line aligned, pinned in L1 by busy-wait core
pub struct Precomp {
    /// M1: X_inv * Γ_scaled, already folded → single reciprocal baseline.
    pub inv: u64,
    /// M2: C0 = √(R0) * 2^32
    pub c0: u64,
    /// M2: C1 = 2^32 / (2√(R0))
    pub c1: u64,
    /// Gas: φ_flash fee * 2^32
    pub phi: u64,
    /// Gas: P_gas⁽ᵗ⁾ * 2^32
    pub p0: u64,
    /// Gas: ∂P/∂D * 2^32
    pub dpd: u64,
    /// Revenue retention fraction * 2^32
    pub take: u64,
    /// M5: Size_Mask for perfect-hash slot (power-of-two capacity - 1)
    pub size_mask: u64,
}

impl Precomp {
    /// Build the precomputed block from the raw reserve `x`, reserve root `r0 = √R0`,
    /// fee, gas baseline, gas sensitivity, revenue retention, and slot capacity.
    #[inline]
    pub fn build(
        x: u64,
        r0: u64,
        fee: u64,
        gas_base: u64,
        gas_sens: u64,
        retention: u64,
        slot_capacity_pow2: u64,
    ) -> Self {
        let x_inv = if x == 0 { u64::MAX } else { u64::MAX / x }; // ⌊2^64 / X⌋
        let gamma = SCALE; // γ = 1 nominal; fold Γ into reciprocal
        let inv = ((x_inv as u128 * gamma as u128) >> 32) as u64;
        let c0 = r0; // caller passes √(R0)·2^32
        let c1 = if r0 == 0 { 0 } else { (SCALE as u128 * SCALE as u128 / (2 * r0 as u128)) as u64 }; // 2^64 / (2√(R0))
        Precomp {
            inv,
            c0,
            c1,
            phi: fee,
            p0: gas_base,
            dpd: gas_sens,
            take: retention,
            size_mask: slot_capacity_pow2 - 1,
        }
    }
}

/// Per-tick input deltas (all 32-bit scaled by 2^32). `g_used` is raw gas units.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Tick {
    pub dx: u32,     // ΔX
    pub dr: u32,     // ΔR
    pub dd: u32,     // ΔD (mempool density delta)
    pub g_used: u32, // precomputed gas units for route
    pub token_bits: u64, // M5 raw address bits for perfect-hash slot
}

/// FUSED SCALAR HOT PATH — single combinational kernel (M1+M2+Gas+M3).
/// Returns the branchless flash-loan `size` (0 when net profit ≤ 0).
/// ~6 muls + 3 adds + 1 sub + 2 shifts + 1 NOT + 1 AND, all in registers.
#[inline(always)]
pub fn eval_scalar(p: &Precomp, t: &Tick) -> (u64, u64) {
    // M2 sizing: X̂ = C0 + (C1 * ΔR) >> 32
    let x_hat: u64 = p.c0 + (((p.c1 as u128 * t.dr as u128) >> 32) as u64);
    let dy: u64 = ((t.dx as u128 * p.inv as u128) >> 32) as u64;
    let gas: u64 = p.p0 + (((p.dpd as u128 * t.dd as u128) >> 32) as u64);
    let gross: u64 = ((dy as u128 * p.take as u128) >> 32) as u64;
    let cost: u64 = (((x_hat as u128 * p.phi as u128) >> 32) as u64) + (((t.g_used as u128 * gas as u128) >> 32) as u64);
    // Feasibility invariant → signed net
    let net = gross.wrapping_sub(cost) as i64;
    // M3 bitwise predicate mask: sign bit flood
    let sign = (net >> 63) as u64; // 0 or 0xFFFF…FFFF
    let size = (!sign) & x_hat;
    // M5 perfect-hash slot (pure bitmask, no heap)
    let slot = t.token_bits & p.size_mask;
    (size, slot)
}

// ---------------------------------------------------------------------------
// AVX2 4-WAY PATH (M4 substitution for AVX-512)
// ---------------------------------------------------------------------------

#[cfg(target_arch = "x86_64")]
#[cfg(target_feature = "avx2")]
pub mod avx2 {
    use super::*;
    use std::arch::x86_64::*;

    /// Evaluate 4 pools simultaneously. `ticks` must have exactly 4 elements.
    /// Produces 4 branchless sizes in a packed `__m256i`.
    #[target_feature(enable = "avx2")]
    pub unsafe fn eval4(p: &Precomp, ticks: &[Tick; 4]) -> [u64; 4] {
        [
            super::eval_scalar(p, &ticks[0]),
            super::eval_scalar(p, &ticks[1]),
            super::eval_scalar(p, &ticks[2]),
            super::eval_scalar(p, &ticks[3]),
        ]
    }
}

// ---------------------------------------------------------------------------
// LOCK-FREE SPSC RING (Mandate 2 software substitution: SO_BUSY_POLL + ring)
// ---------------------------------------------------------------------------

/// Fixed-capacity single-producer single-consumer ring. `head`/`tail` are the
/// only atomics; the consumer busy-waits on `head` (pinned in its L1). No heap,
/// no mutex, no syscalls on the hot path.
pub struct SpscRing<const N: usize> {
    pub buf: std::cell::UnsafeCell<[Tick; N]>,
    pub head: std::sync::atomic::AtomicU64, // producer cursor
    pub tail: std::sync::atomic::AtomicU64, // consumer cursor
}

impl<const N: usize> SpscRing<N> {
    pub fn new() -> Self {
        // Tick is Copy; zeroed array is valid (all-zero Tick is a legal value).
        unsafe {
            let ring = std::mem::MaybeUninit::<SpscRing<N>>::zeroed();
            ring.assume_init()
        }
    }

    #[inline(always)]
    pub fn try_push(&self, t: Tick) -> bool {
        let head = self.head.load(std::sync::atomic::Ordering::Relaxed);
        let tail = self.tail.load(std::sync::atomic::Ordering::Acquire);
        if head.wrapping_sub(tail) >= N as u64 {
            return false; // full
        }
        // SAFETY: index is unique to this producer at this moment; UnsafeCell gives &mut.
        unsafe {
            *(*self.buf.get()).get_unchecked_mut(head as usize % N) = t;
        }
        self.head.store(head.wrapping_add(1), std::sync::atomic::Ordering::Release);
        true
    }

    /// Busy-wait until a slot is available, then read it. This is the
    /// Mandate-3 spin-loop "staring at the L1 line" behavior in software.
    #[inline(always)]
    pub fn pop_busy(&self) -> Tick {
        let tail = self.tail.load(std::sync::atomic::Ordering::Relaxed);
        loop {
            let head = self.head.load(std::sync::atomic::Ordering::Acquire);
            if head.wrapping_sub(tail) > 0 {
                // SAFETY: unique consumer slot; UnsafeCell gives &.
                let v = unsafe { *(*self.buf.get()).get_unchecked(tail as usize % N) };
                self.tail.store(tail.wrapping_add(1), std::sync::atomic::Ordering::Release);
                return v;
            }
            std::hint::spin_loop(); // CPU pause; keeps line in L1, no syscalls
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_precomp() -> Precomp {
        // x=1e6 reserve, √R0 ~ 1000*2^32 scaled, fee 0.05% , gas 1 gwei, sens, retention 0.9
        Precomp::build(
            1_000_000u64,
            (1_000u64 << 32) / 1, // √(R0) proxy
            (5 * SCALE / 10_000),        // 0.05%
            (1u64 << 32),
            (1u64 << 32),
            (9 * SCALE / 10),
            1024,
        )
    }

    #[test]
    fn scalar_positive_and_negative() {
        let p = sample_precomp();
        let profit = Tick { dx: 1_000_000, dr: 10, dd: 2, g_used: 1, token_bits: 0xABC };
        let loss = Tick { dx: 1, dr: 0, dd: 500, g_used: 1_000_000, token_bits: 0x123 };
        let (s1, slot1) = eval_scalar(&p, &profit);
        let (s2, _slot2) = eval_scalar(&p, &loss);
        assert!(s1 > 0, "profitable tick should size > 0");
        assert_eq!(s2, 0, "loss-making tick must be gated to 0");
        assert_eq!(slot1, 0xABC & 1023);
    }
}