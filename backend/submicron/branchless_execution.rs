// ==============================================================================
// Branchless Execution Engine (BEE)
// Mathematical Foundation: Result = (Condition x True_Path) + ((1 - Condition) x False_Path)
// ==============================================================================


#[repr(C, align(64))]
pub struct BranchlessMask {
    pub mask: u64,
}

impl BranchlessMask {
    #[inline(always)]
    pub fn from_bool(condition: bool) -> Self {
        Self {
            mask: if condition { !0u64 } else { 0u64 },
        }
    }

    #[inline(always)]
    pub fn from_u64(condition: u64) -> Self {
        Self {
            mask: -((condition != 0) as i64) as u64,
        }
    }

    #[inline(always)]
    pub fn select(&self, true_val: u64, false_val: u64) -> u64 {
        (self.mask & true_val) | (!self.mask & false_val)
    }

    #[inline(always)]
    pub fn select_i64(&self, true_val: i64, false_val: i64) -> i64 {
        ((self.mask as i64) & true_val) | ((!self.mask as i64) & false_val)
    }
}

#[inline(always)]
pub fn branchless_select(condition: bool, true_val: u64, false_val: u64) -> u64 {
    let mask = if condition { !0u64 } else { 0u64 };
    (mask & true_val) | (!mask & false_val)
}

#[inline(always)]
pub fn branchless_select_i64(condition: bool, true_val: i64, false_val: i64) -> i64 {
    let mask = -((condition as i64) as i64);
    (mask & true_val) | (!mask & false_val)
}

#[inline(always)]
pub fn branchless_validation_mask(net_profit: i64) -> u64 {
    let msb = (net_profit >> 63) as u64;
    !msb
}

#[repr(C, align(64))]
pub struct BranchlessValidator {
    pub slippage_exceeded: u64,
    pub profit_below: u64,
    pub rejection_mask: u64,
}

impl BranchlessValidator {
    #[inline(always)]
    pub fn new(slippage_bps: u32, max_slippage_bps: u32, net_profit: i64, min_profit: i64) -> Self {
        let slippage_exceeded = (slippage_bps > max_slippage_bps) as u64;
        let profit_below = (net_profit < min_profit) as u64;
        let rejection_mask = slippage_exceeded | profit_below;

        Self {
            slippage_exceeded,
            profit_below,
            rejection_mask,
        }
    }

    #[inline(always)]
    pub fn is_rejected(&self) -> bool {
        self.rejection_mask != 0
    }

    #[inline(always)]
    pub fn rejection_reason(&self) -> u64 {
        self.slippage_exceeded
    }
}

#[repr(C, align(64))]
#[derive(Debug, Clone)]
pub struct BitwiseReciprocalTable {
    pub shifts: [u8; 256],
    pub masks: [u64; 256],
}

impl BitwiseReciprocalTable {
    pub fn new() -> Self {
        let mut shifts = [0u8; 256];
        let mut masks = [0u64; 256];

        for i in 0..256 {
            shifts[i] = (i as u8).leading_zeros() as u8;
            masks[i] = if i == 0 { 0 } else { u64::MAX >> (64 - i) };
        }

        Self { shifts, masks }
    }

    #[inline(always)]
    pub fn reciprocal_shift(&self, value: u64) -> u8 {
        self.shifts[value.min(255) as usize]
    }

    #[inline(always)]
    pub fn apply_mask(&self, value: u64, bits: u64) -> u64 {
        value & self.masks[bits.min(63) as usize]
    }
}

impl Default for BitwiseReciprocalTable {
    fn default() -> Self {
        Self::new()
    }
}

#[repr(C, align(64))]
#[derive(Debug, Clone)]
pub struct DirectStepArray {
    pub steps: [i64; 1024],
    pub granularity_shift: u8,
}

impl DirectStepArray {
    pub fn new(granularity_shift: u8) -> Self {
        let mut steps = [0i64; 1024];

        for i in 0..1024 {
            steps[i] = ((i as i64) << granularity_shift) / 1024;
        }

        Self {
            steps,
            granularity_shift,
        }
    }

    #[inline(always)]
    pub fn lookup(&self, delta_r: u64) -> i64 {
        let idx = ((delta_r >> self.granularity_shift) & 0x3FF) as usize;
        self.steps[idx]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_branchless_select() {
        assert_eq!(branchless_select(true, 10, 20), 10);
        assert_eq!(branchless_select(false, 10, 20), 20);
    }

    #[test]
    fn test_branchless_select_i64() {
        assert_eq!(branchless_select_i64(true, 100, -100), 100);
        assert_eq!(branchless_select_i64(false, 100, -100), -100);
    }

    #[test]
    fn test_validation_mask() {
        assert_eq!(branchless_validation_mask(100), !0u64);
        assert_eq!(branchless_validation_mask(-100), 0);
    }

    #[test]
    fn test_branchless_validator() {
        let v = BranchlessValidator::new(100, 50, 10, 5);
        assert!(v.is_rejected());
        assert_eq!(v.rejection_reason(), 1);
    }

    #[test]
    fn test_bitwise_reciprocal_table() {
        let table = BitwiseReciprocalTable::new();
        assert_eq!(table.reciprocal_shift(256), 8);
        assert_eq!(table.apply_mask(0xFFFF, 16), 0xFFFF);
    }

    #[test]
    fn test_direct_step_array() {
        let arr = DirectStepArray::new(6);
        let val = arr.lookup(512);
        assert!(val >= 0);
    }
}
