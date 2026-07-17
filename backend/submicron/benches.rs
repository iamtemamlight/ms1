// ==============================================================================
// Sub-Microsecond Latency Benchmarks
// Validates that core infrastructure meets < 1μs targets
// ==============================================================================

use std::time::{Instant, Duration};
use std::hint::black_box;

use crate::submicron::{MemoryPool, LockFreeQueue, StatePredictionTable, PredictedState};
use crate::submicron::branchless_execution::{BranchlessValidator, branchless_select, branchless_validation_mask, DirectStepArray};
use crate::submicron::zero_copy_network::{BinaryProtocolEncoder, BinaryProtocolDecoder, ZeroCopyMessage};
use crate::submicron::predictive_batching::{BatchOptimizer, Opportunity};

/// Benchmark memory pool allocation
pub fn benchmark_memory_pool() -> BenchmarkResult {
    let pool = MemoryPool::new(1024 * 1024); // 1MB pool
    
    // Warmup
    for _ in 0..1000 {
        let _ = pool.allocate_aligned(64);
    }
    pool.reset();
    
    // Benchmark
    let iterations = 1_000_000;
    let start = Instant::now();
    
    for _ in 0..iterations {
        black_box(pool.allocate_aligned(64));
    }
    
    let duration = start.elapsed();
    let avg_ns = duration.as_nanos() as f64 / iterations as f64;
    
    BenchmarkResult {
        name: "MemoryPool::allocate_aligned".to_string(),
        iterations,
        total_duration: duration,
        avg_latency_ns: avg_ns,
        target_ns: 1.0,
        passed: avg_ns < 1.0,
    }
}

/// Benchmark lock-free queue push
pub fn benchmark_lock_free_queue_push() -> BenchmarkResult {
    let queue = LockFreeQueue::new();
    
    // Warmup
    for i in 0..1000 {
        while !queue.push(i) {
            std::hint::spin_loop();
        }
    }
    
    // Benchmark
    let iterations = 1_000_000;
    let start = Instant::now();
    
    for i in 0..iterations {
        while !black_box(queue.push(i)) {
            std::hint::spin_loop();
        }
    }
    
    let duration = start.elapsed();
    let avg_ns = duration.as_nanos() as f64 / iterations as f64;
    
    BenchmarkResult {
        name: "LockFreeQueue::push".to_string(),
        iterations,
        total_duration: duration,
        avg_latency_ns: avg_ns,
        target_ns: 5.0,
        passed: avg_ns < 5.0,
    }
}

/// Benchmark lock-free queue pop
pub fn benchmark_lock_free_queue_pop() -> BenchmarkResult {
    let queue = LockFreeQueue::new();
    
    // Pre-fill queue
    for i in 0..1_000_000 {
        while !queue.push(i) {
            std::hint::spin_loop();
        }
    }
    
    // Benchmark
    let iterations = 1_000_000;
    let start = Instant::now();
    
    for _ in 0..iterations {
        black_box(queue.pop());
    }
    
    let duration = start.elapsed();
    let avg_ns = duration.as_nanos() as f64 / iterations as f64;
    
    BenchmarkResult {
        name: "LockFreeQueue::pop".to_string(),
        iterations,
        total_duration: duration,
        avg_latency_ns: avg_ns,
        target_ns: 5.0,
        passed: avg_ns < 5.0,
    }
}

/// Benchmark state prediction lookup
pub fn benchmark_state_prediction() -> BenchmarkResult {
    let initial = PredictedState {
        eth_price: 3000_0000000000000000,
        gas_price: 30_0000000000000000,
        pool_reserves: [1_000_000_000_000_000_000; 32],
        block_hash: [0; 32],
        timestamp: 0,
        confidence: 10000,
    };
    
    let table = StatePredictionTable::new(0, 0, &initial);
    
    // Warmup
    for i in 0..1000 {
        black_box(table.get_state(i));
    }
    
    // Benchmark
    let iterations = 10_000_000;
    let start = Instant::now();
    
    for i in 0..iterations {
        black_box(table.get_state(i % 10000));
    }
    
    let duration = start.elapsed();
    let avg_ns = duration.as_nanos() as f64 / iterations as f64;
    
    BenchmarkResult {
        name: "StatePredictionTable::get_state".to_string(),
        iterations,
        total_duration: duration,
        avg_latency_ns: avg_ns,
        target_ns: 10.0,
        passed: avg_ns < 10.0,
    }
}

/// Benchmark end-to-end simulated transaction
pub fn benchmark_transaction_pipeline() -> BenchmarkResult {
    let pool = MemoryPool::new(1024 * 1024);
    let queue = LockFreeQueue::new();
    let initial = PredictedState {
        eth_price: 3000_0000000000000000,
        gas_price: 30_0000000000000000,
        pool_reserves: [1_000_000_000_000_000_000; 32],
        block_hash: [0; 32],
        timestamp: 0,
        confidence: 10000,
    };
    let table = StatePredictionTable::new(0, 0, &initial);
    
    // Simulate transaction: allocate memory, queue, predict state
    let iterations = 100_000;
    let start = Instant::now();
    
    for i in 0..iterations {
        // Allocate transaction data
        let tx_data = pool.allocate_aligned(256);
        
        // Queue transaction
        while !queue.push(i) {
            std::hint::spin_loop();
        }
        
        // Predict state
        let _state = table.get_state(i % 1000);
        
        black_box(tx_data);
    }
    
    let duration = start.elapsed();
    let avg_ns = duration.as_nanos() as f64 / iterations as f64;
    
    BenchmarkResult {
        name: "TransactionPipeline (end-to-end)".to_string(),
        iterations,
        total_duration: duration,
        avg_latency_ns: avg_ns,
        target_ns: 1000.0, // 1μs target
        passed: avg_ns < 1000.0,
    }
}

/// Benchmark branchless validation (BEE)
pub fn benchmark_branchless_validation() -> BenchmarkResult {
    let iterations = 10_000_000;
    let start = Instant::now();

    for i in 0..iterations {
        let slippage = (i % 200) as u32;
        let max_slippage = 100u32;
        let profit = (i as i64) - 1000;
        let min_profit = 0i64;
        let _validator = BranchlessValidator::new(slippage, max_slippage, profit, min_profit);
        black_box(_validator.rejection_mask);
    }

    let duration = start.elapsed();
    let avg_ns = duration.as_nanos() as f64 / iterations as f64;

    BenchmarkResult {
        name: "BranchlessValidator::new".to_string(),
        iterations,
        total_duration: duration,
        avg_latency_ns: avg_ns,
        target_ns: 5.0,
        passed: avg_ns < 5.0,
    }
}

/// Benchmark branchless select
pub fn benchmark_branchless_select() -> BenchmarkResult {
    let iterations = 10_000_000;
    let start = Instant::now();

    for i in 0..iterations {
        let cond = (i % 2) == 0;
        let _val = branchless_select(cond, 10, 20);
        black_box(_val);
    }

    let duration = start.elapsed();
    let avg_ns = duration.as_nanos() as f64 / iterations as f64;

    BenchmarkResult {
        name: "branchless_select".to_string(),
        iterations,
        total_duration: duration,
        avg_latency_ns: avg_ns,
        target_ns: 1.0,
        passed: avg_ns < 1.0,
    }
}

/// Benchmark direct step array lookup
pub fn benchmark_step_array() -> BenchmarkResult {
    let step_array = DirectStepArray::new(6);
    let iterations = 10_000_000;
    let start = Instant::now();

    for i in 0..iterations {
        let _val = step_array.lookup(i as u64);
        black_box(_val);
    }

    let duration = start.elapsed();
    let avg_ns = duration.as_nanos() as f64 / iterations as f64;

    BenchmarkResult {
        name: "DirectStepArray::lookup".to_string(),
        iterations,
        total_duration: duration,
        avg_latency_ns: avg_ns,
        target_ns: 2.0,
        passed: avg_ns < 2.0,
    }
}

/// Benchmark zero-copy message serialization
pub fn benchmark_zero_copy_message() -> BenchmarkResult {
    let iterations = 1_000_000;
    let start = Instant::now();

    for i in 0..iterations {
        let mut msg = ZeroCopyMessage::new(1);
        msg.timestamp = i as u64;
        msg.data_len = 64;
        let _bytes = msg.as_bytes();
        black_box(_bytes);
    }

    let duration = start.elapsed();
    let avg_ns = duration.as_nanos() as f64 / iterations as f64;

    BenchmarkResult {
        name: "ZeroCopyMessage::as_bytes".to_string(),
        iterations,
        total_duration: duration,
        avg_latency_ns: avg_ns,
        target_ns: 10.0,
        passed: avg_ns < 10.0,
    }
}

/// Benchmark binary protocol encoder
pub fn benchmark_binary_encoder() -> BenchmarkResult {
    let mut encoder = BinaryProtocolEncoder::new();
    let iterations = 1_000_000;
    let start = Instant::now();

    for i in 0..iterations {
        encoder.reset();
        encoder.encode_u64(i as u64);
        encoder.encode_u32(0xDEADBEEF);
        let _out = encoder.finalize();
        black_box(_out);
    }

    let duration = start.elapsed();
    let avg_ns = duration.as_nanos() as f64 / iterations as f64;

    BenchmarkResult {
        name: "BinaryProtocolEncoder".to_string(),
        iterations,
        total_duration: duration,
        avg_latency_ns: avg_ns,
        target_ns: 20.0,
        passed: avg_ns < 20.0,
    }
}

/// Benchmark batch optimizer
pub fn benchmark_batch_optimizer() -> BenchmarkResult {
    let optimizer = BatchOptimizer::new(64, 21000, 21000);
    let opportunities = vec![
        Opportunity { profit: 100_000_000_000_000_000, gas_cost: 50_000_000_000, gas_limit: 150000, priority: 1, pool_hash: 1, pair_hash: 2 },
        Opportunity { profit: 200_000_000_000_000_000, gas_cost: 60_000_000_000, gas_limit: 150000, priority: 1, pool_hash: 2, pair_hash: 3 },
        Opportunity { profit: 150_000_000_000_000_000, gas_cost: 55_000_000_000, gas_limit: 150000, priority: 1, pool_hash: 3, pair_hash: 4 },
    ];

    let iterations = 100_000;
    let start = Instant::now();

    for _ in 0..iterations {
        let _result = optimizer.find_optimal_batch(&opportunities);
        black_box(_result);
    }

    let duration = start.elapsed();
    let avg_ns = duration.as_nanos() as f64 / iterations as f64;

    BenchmarkResult {
        name: "BatchOptimizer::find_optimal_batch".to_string(),
        iterations,
        total_duration: duration,
        avg_latency_ns: avg_ns,
        target_ns: 1000.0,
        passed: avg_ns < 1000.0,
    }
}

#[derive(Debug)]
pub struct BenchmarkResult {
    pub name: String,
    pub iterations: u64,
    pub total_duration: Duration,
    pub avg_latency_ns: f64,
    pub target_ns: f64,
    pub passed: bool,
}

impl BenchmarkResult {
    pub fn print(&self) {
        let status = if self.passed { "✅ PASS" } else { "❌ FAIL" };
        println!(
            "{}: {} | {} iterations | {:.2}ns avg (target: {:.2}ns)",
            status, self.name, self.iterations, self.avg_latency_ns, self.target_ns
        );
    }
}

/// Run all benchmarks and print results
pub fn run_all_benchmarks() {
    println!("=== Sub-Microsecond Latency Benchmarks ===\n");
    
    let results = vec![
        benchmark_memory_pool(),
        benchmark_lock_free_queue_push(),
        benchmark_lock_free_queue_pop(),
        benchmark_state_prediction(),
        benchmark_transaction_pipeline(),
        benchmark_branchless_validation(),
        benchmark_branchless_select(),
        benchmark_step_array(),
        benchmark_zero_copy_message(),
        benchmark_binary_encoder(),
        benchmark_batch_optimizer(),
    ];
    
    let passed = results.iter().filter(|r| r.passed).count();
    let total = results.len();
    
    println!("\n=== Summary ===");
    println!("Passed: {}/{}", passed, total);
    
    if passed == total {
        println!("✅ All benchmarks passed - Phase 1 complete!");
    } else {
        println!("❌ Some benchmarks failed - optimization needed");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_memory_pool_benchmark() {
        let result = benchmark_memory_pool();
        result.print();
        assert!(result.passed, "Memory pool allocation should be < 1ns");
    }
    
    #[test]
    fn test_queue_push_benchmark() {
        let result = benchmark_lock_free_queue_push();
        result.print();
        assert!(result.passed, "Queue push should be < 5ns");
    }
    
    #[test]
    fn test_queue_pop_benchmark() {
        let result = benchmark_lock_free_queue_pop();
        result.print();
        assert!(result.passed, "Queue pop should be < 5ns");
    }
    
    #[test]
    fn test_state_prediction_benchmark() {
        let result = benchmark_state_prediction();
        result.print();
        assert!(result.passed, "State lookup should be < 10ns");
    }
    
    #[test]
    fn test_transaction_pipeline_benchmark() {
        let result = benchmark_transaction_pipeline();
        result.print();
        assert!(result.passed, "Transaction pipeline should be < 1μs");
    }
}
