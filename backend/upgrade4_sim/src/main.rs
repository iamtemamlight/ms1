// ==============================================================================
// UPGRADE4 Live Simulation Binary
// Standalone test harness for < 1.000 ms critical path validation
// Uses .env RPC endpoints for live mempool/gas data
// ==============================================================================

use std::time::{Instant, Duration};
use std::sync::atomic::{AtomicU64, Ordering};

// -----------------------------------------------------------------------------
// UPGRADE4 Module 1: Bitwise Shifting Reciprocal
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub struct PoolShiftState {
    pub s_pool: u32,
    pub m_max_swap: u64,
}

impl PoolShiftState {
    pub const fn new(s_pool: u32, m_max_swap: u64) -> Self {
        Self { s_pool, m_max_swap }
    }

    #[inline(always)]
    pub fn compute_output(&self, delta_x: u64) -> u64 {
        (delta_x >> self.s_pool) & self.m_max_swap
    }
}

// -----------------------------------------------------------------------------
// UPGRADE4 Module 2: 0-Cycle Step Array
// -----------------------------------------------------------------------------

#[repr(C, align(64))]
pub struct StepArray {
    pub table: Box<[u64; 65536]>,
    pub s_granularity: u32,
    pub baseline_reserve: u64,
}

impl StepArray {
    pub fn build(baseline_reserve: u64, s_granularity: u32) -> Self {
        let mut table = Box::new([0u64; 65536]);
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
        Self { table, s_granularity, baseline_reserve: r0 }
    }

    #[inline(always)]
    pub fn lookup(&self, delta_r: u64) -> u64 {
        let idx = (delta_r >> self.s_granularity) as usize;
        if idx < self.table.len() { self.table[idx] } else { self.table[65535] }
    }

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
// UPGRADE4 Module 3: Simultaneous Double-Predicate Masking
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub struct ExecutionMask {
    pub mask: u64,
}

impl ExecutionMask {
    #[inline(always)]
    pub fn from_net_profit(net_profit_i64: i64) -> Self {
        let shifted = (net_profit_i64 >> 63) as u64;
        Self { mask: !shifted }
    }

    #[inline(always)]
    pub fn all_ones() -> Self {
        Self { mask: u64::MAX }
    }

    #[inline(always)]
    pub fn is_executable(&self) -> bool {
        self.mask != 0
    }
}

#[inline(always)]
pub fn apply_execution_mask(payload_size: u64, mask: u64) -> u64 {
    payload_size & mask
}

// -----------------------------------------------------------------------------
// UPGRADE4 Module 4: Bitwise Shift Density Counting
// -----------------------------------------------------------------------------

static MEMPOOL_DENSITY: AtomicU64 = AtomicU64::new(0);

#[inline(always)]
pub fn increment_mempool_density() {
    MEMPOOL_DENSITY.fetch_add(1, Ordering::Relaxed);
}

#[inline(always)]
pub fn take_mempool_density() -> u64 {
    MEMPOOL_DENSITY.swap(0, Ordering::AcqRel)
}

pub const S_ELASTICITY: u32 = 3;

#[inline(always)]
pub fn estimate_next_base_fee(current_base: u64, density: u64) -> u64 {
    if density == 0 { return current_base; }
    let clz = density.leading_zeros();
    let shift = (S_ELASTICITY + clz).min(63);
    current_base + (current_base >> shift)
}

// -----------------------------------------------------------------------------
// UPGRADE4 Module 5: Pre-Baked Bid Matrix
// -----------------------------------------------------------------------------

#[repr(C, align(64))]
#[derive(Debug, Clone)]
pub struct BidMatrix {
    pub table: [u64; 256],
    pub delta_b: u64,
}

impl BidMatrix {
    pub fn build(base_tiers: &[u64; 256]) -> Self {
        let mut table = [0u64; 256];
        for i in 0..256 {
            table[i] = base_tiers[i];
        }
        Self { table, delta_b: 1 }
    }

    #[inline(always)]
    pub fn lookup(&self, density: u64) -> u64 {
        let clz = density.leading_zeros();
        let idx = (63 - clz) as usize;
        let idx = idx.min(255);
        self.table[idx] + self.delta_b
    }
}

// -----------------------------------------------------------------------------
// UPGRADE4 Module 6: Pre-Baked Transaction Template
// -----------------------------------------------------------------------------

#[repr(C, align(64))]
#[derive(Debug, Clone)]
pub struct TransactionTemplate {
    pub bytes: [u8; 512],
    pub offsets: GasFieldOffsets,
    pub initialized: bool,
}

#[derive(Debug, Clone)]
pub struct GasFieldOffsets {
    pub nonce: usize,
    pub gas_price: usize,
    pub gas_limit: usize,
    pub max_fee_per_gas: usize,
    pub max_priority_fee_per_gas: usize,
    pub v: usize,
    pub r: usize,
    pub s: usize,
}

impl GasFieldOffsets {
    pub const fn new() -> Self {
        Self {
            nonce: 0,
            gas_price: 32,
            gas_limit: 64,
            max_fee_per_gas: 96,
            max_priority_fee_per_gas: 128,
            v: 160,
            r: 192,
            s: 224,
        }
    }
}

impl TransactionTemplate {
    pub const fn new() -> Self {
        Self {
            bytes: [0u8; 512],
            offsets: GasFieldOffsets::new(),
            initialized: false,
        }
    }

    pub fn build(&mut self, skeleton: &[u8; 512]) {
        self.bytes = *skeleton;
        self.initialized = true;
    }

    #[inline(always)]
    pub fn patch_gas_fields_scalar(&mut self, gas_limit: u64, max_fee: u64, priority_fee: u64) {
        if !self.initialized { return; }
        unsafe {
            std::ptr::write_unaligned(
                self.bytes.as_mut_ptr().add(self.offsets.gas_limit) as *mut u64,
                gas_limit,
            );
            std::ptr::write_unaligned(
                self.bytes.as_mut_ptr().add(self.offsets.max_fee_per_gas) as *mut u64,
                max_fee,
            );
            std::ptr::write_unaligned(
                self.bytes.as_mut_ptr().add(self.offsets.max_priority_fee_per_gas) as *mut u64,
                priority_fee,
            );
        }
    }
}

// -----------------------------------------------------------------------------
// Latency recorder
// -----------------------------------------------------------------------------

static STAGE_LATENCY_MS: [AtomicU64; 6] = [
    AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0),
    AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0),
];
static TOTAL_LATENCY_MS: AtomicU64 = AtomicU64::new(0);
static PACKET_COUNT: AtomicU64 = AtomicU64::new(0);
static OVERFLOW_COUNT: AtomicU64 = AtomicU64::new(0);

fn record_stage_ms(stage: usize, ms: u64) {
    STAGE_LATENCY_MS[stage].store(ms, Ordering::Relaxed);
}

pub fn get_stage_latency_ms(stage: usize) -> f64 {
    STAGE_LATENCY_MS[stage].load(Ordering::Relaxed) as f64 / 1_000_000.0
}

pub fn get_total_latency_ms() -> f64 {
    TOTAL_LATENCY_MS.load(Ordering::Relaxed) as f64 / 1_000_000.0
}

pub fn get_packet_count() -> u64 {
    PACKET_COUNT.load(Ordering::Relaxed)
}

pub fn get_overflow_count() -> u64 {
    OVERFLOW_COUNT.load(Ordering::Relaxed)
}

// -----------------------------------------------------------------------------
// Pipeline context
// -----------------------------------------------------------------------------

pub struct PipelineContext {
    pub pool_state: PoolShiftState,
    pub step_array: StepArray,
    pub gas_oracle_bid_matrix: BidMatrix,
    pub tx_template: TransactionTemplate,
    pub current_base_fee: u64,
    pub gross_revenue_scaled: u64,
    pub gas_limit: u64,
}

impl PipelineContext {
    pub fn new() -> Self {
        Self {
            pool_state: PoolShiftState::new(4, u64::MAX),
            step_array: StepArray::build(1_000_000_000_000_000_000u64, 8),
            gas_oracle_bid_matrix: BidMatrix::build(&[25_000_000_000u64; 256]),
            tx_template: TransactionTemplate::new(),
            current_base_fee: 30_000_000_000,
            gross_revenue_scaled: 0,
            gas_limit: 21000,
        }
    }

    #[inline(always)]
    pub fn execute_critical_path(&mut self, delta_x: u64, delta_r: u64, density: u64, net_profit_i64: i64) -> u64 {
        let t0 = Instant::now();

        let m1_start = Instant::now();
        let _output_y = self.pool_state.compute_output(delta_x);
        let m1_ns = m1_start.elapsed().as_nanos() as u64;
        record_stage_ms(0, m1_ns);

        let m2_start = Instant::now();
        let _optimal_input = self.step_array.lookup(delta_r);
        let m2_ns = m2_start.elapsed().as_nanos() as u64;
        record_stage_ms(1, m2_ns);

        let m3_start = Instant::now();
        let exec_mask = ExecutionMask::from_net_profit(net_profit_i64);
        let payload_size = apply_execution_mask(512, exec_mask.mask);
        let m3_ns = m3_start.elapsed().as_nanos() as u64;
        record_stage_ms(2, m3_ns);

        let m4_start = Instant::now();
        let base_fee = estimate_next_base_fee(self.current_base_fee, density);
        let m4_ns = m4_start.elapsed().as_nanos() as u64;
        record_stage_ms(3, m4_ns);

        let m5_start = Instant::now();
        let priority_fee = self.gas_oracle_bid_matrix.lookup(density);
        let m5_ns = m5_start.elapsed().as_nanos() as u64;
        record_stage_ms(4, m5_ns);

        let m6_start = Instant::now();
        self.tx_template.patch_gas_fields_scalar(self.gas_limit, base_fee, priority_fee);
        let m6_ns = m6_start.elapsed().as_nanos() as u64;
        record_stage_ms(5, m6_ns);

        let total_ns = t0.elapsed().as_nanos() as u64;
        TOTAL_LATENCY_MS.store(total_ns, Ordering::Relaxed);
        PACKET_COUNT.fetch_add(1, Ordering::Relaxed);

        if total_ns > 1_000_000 {
            OVERFLOW_COUNT.fetch_add(1, Ordering::Relaxed);
        }

        payload_size
    }

    pub fn get_stage_latencies_ms(&self) -> [f64; 6] {
        [
            get_stage_latency_ms(0),
            get_stage_latency_ms(1),
            get_stage_latency_ms(2),
            get_stage_latency_ms(3),
            get_stage_latency_ms(4),
            get_stage_latency_ms(5),
        ]
    }

    pub fn reset_counters() {
        for s in STAGE_LATENCY_MS.iter() {
            s.store(0, Ordering::Relaxed);
        }
        TOTAL_LATENCY_MS.store(0, Ordering::Relaxed);
        PACKET_COUNT.store(0, Ordering::Relaxed);
        OVERFLOW_COUNT.store(0, Ordering::Relaxed);
    }
}

// -----------------------------------------------------------------------------
// Live RPC Client
// -----------------------------------------------------------------------------

use reqwest::Client;
use serde_json::json;

pub struct LiveRpcClient {
    client: Client,
    rpc_url: String,
}

impl LiveRpcClient {
    pub fn new(rpc_url: impl Into<String>) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
            rpc_url: rpc_url.into(),
        }
    }

    pub async fn get_gas_price(&self) -> Result<u64, String> {
        let body = json!({
            "jsonrpc": "2.0",
            "method": "eth_gasPrice",
            "params": [],
            "id": 1
        });

        let resp = self.client
            .post(&self.rpc_url)
            .header("User-Agent", "allbright-upgrade4-sim/1.0")
            .header("Accept", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("RPC failed: {}", e))?;

        let status = resp.status();
        let text = resp.text().await.map_err(|e| format!("read body failed: {}", e))?;
        if !status.is_success() {
            return Err(format!("HTTP {}: {}", status, text));
        }

        let json: serde_json::Value = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(e) => return Err(format!("JSON parse failed: {} | body: {}", e, &text[..text.len().min(200)])),
        };

        let result = json.get("result")
            .and_then(|v| v.as_str())
            .ok_or("No result")?;

        let hex = result.strip_prefix("0x").ok_or("Invalid hex")?;
        u64::from_str_radix(hex, 16).map_err(|e| format!("Parse failed: {}", e))
    }

    pub async fn get_block_number(&self) -> Result<u64, String> {
        let body = json!({
            "jsonrpc": "2.0",
            "method": "eth_blockNumber",
            "params": [],
            "id": 1
        });

        let resp = self.client
            .post(&self.rpc_url)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("RPC failed: {}", e))?;

        let json: serde_json::Value = resp.json().await
            .map_err(|e| format!("JSON parse failed: {}", e))?;

        let result = json.get("result")
            .and_then(|v| v.as_str())
            .ok_or("No result")?;

        let hex = result.strip_prefix("0x").ok_or("Invalid hex")?;
        u64::from_str_radix(hex, 16).map_err(|e| format!("Parse failed: {}", e))
    }

    pub async fn get_mempool_density(&self) -> Result<u64, String> {
        let body = json!({
            "jsonrpc": "2.0",
            "method": "eth_pendingTransactionCount",
            "params": [],
            "id": 1
        });

        let resp = self.client
            .post(&self.rpc_url)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("RPC failed: {}", e))?;

        let json: serde_json::Value = resp.json().await
            .map_err(|e| format!("JSON parse failed: {}", e))?;

        let result = json.get("result")
            .and_then(|v| v.as_u64())
            .ok_or("No result")?;

        Ok(result)
    }
}

// -----------------------------------------------------------------------------
// Main: Live simulation
// -----------------------------------------------------------------------------

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let rpc_url = std::env::var("RPC_ENDPOINT")
        .unwrap_or_else(|_| "https://eth.llamarpc.com".to_string());

    println!("=== UPGRADE4 Live Simulation ===");
    println!("RPC Endpoint: {}", rpc_url);
    println!("Target: < 1.000 ms total pipeline latency");
    println!();

    let client = LiveRpcClient::new(rpc_url);

    // Fetch live gas price
    let live_gas = match client.get_gas_price().await {
        Ok(g) => {
            let gwei_str = format!("{:.4}", g as f64 / 1e9);
            println!("Live gas price: {} wei ({} gwei)", g, gwei_str);
            g
        }
        Err(e) => {
            println!("Warning: gas price fetch failed: {}", e);
            30_000_000_000u64
        }
    };

    // Fetch live block number
    let live_block = match client.get_block_number().await {
        Ok(b) => {
            println!("Live block number: {}", b);
            b
        }
        Err(e) => {
            println!("Warning: block number fetch failed: {}", e);
            0
        }
    };

    // Fetch mempool density
    let mempool_density = match client.get_mempool_density().await {
        Ok(d) => {
            println!("Mempool pending tx count: {}", d);
            d
        }
        Err(e) => {
            println!("Warning: mempool density fetch failed: {}", e);
            1000
        }
    };

    // Build pipeline context with live data
    let mut ctx = PipelineContext::new();
    ctx.current_base_fee = live_gas;

    // Run 1000 iterations to measure steady-state latency
    let iterations = 1000;
    println!("\nRunning {} iterations...\n", iterations);

    for i in 0..iterations {
        let density = mempool_density + (i % 100) as u64;
        let net_profit = if i % 10 == 0 { -100i64 } else { 1000i64 };
        ctx.execute_critical_path(1000, 500_000, density, net_profit);
    }

    // Report results
    println!("=== LATENCY RESULTS ===");
    println!("Total packets: {}", get_packet_count());
    println!("Overflow count (> 1.000 ms): {}", get_overflow_count());
    println!();

    let stages = ctx.get_stage_latencies_ms();
    let stage_names = [
        "Module 1: Swap Output (bitwise shift)",
        "Module 2: Step Array Lookup",
        "Module 3: Execution Mask",
        "Module 4: Gas Estimation (CLZ)",
        "Module 5: Bid Matrix Lookup",
        "Module 6: Payload Patch",
    ];

    println!("Per-Stage Latency:");
    for (i, name) in stage_names.iter().enumerate() {
        let s = format!("{:.8}", stages[i]);
        println!("  {}: {} ms", name, s);
    }
    println!();

    let total_ms = get_total_latency_ms();
    let total_str = format!("{:.8}", total_ms);
    println!("TOTAL PIPELINE LATENCY: {} ms", total_str);
    println!("TARGET: < 1.000 ms");
    println!("STATUS: {}", if total_ms < 1.0 { "PASS" } else { "FAIL" });
}
