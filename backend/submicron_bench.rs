// ==============================================================================
// Standalone Sub-Microsecond Latency Benchmark Tool
// Validates core infrastructure without full project build
// ==============================================================================

use std::time::{Instant, Duration};
use std::hint::black_box;

// Include the submicron modules directly
mod submicron {
    pub mod memory_pool;
    pub mod lock_free_queue;
    pub mod state_prediction;
}

use submicron::{MemoryPool, LockFreeQueue, StatePredictionTable, PredictedState};

fn main() {
    println!("=== Sub-Microsecond Latency Benchmarks ===\n");
    
    let results = vec![
        benchmark_memory_pool(),
        benchmark_lock_free_queue_push(),
        benchmark_lock_free_queue_pop(),
        benchmark_state_prediction(),
        benchmark_transaction_pipeline(),
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

fn benchmark_memory_pool() -> BenchmarkResult {
    let pool = MemoryPool::new(1024 * 1024);
    
    for _ in 0..1000 {
        let _ = pool.allocate_aligned(64);
    }
    pool.reset();
    
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

fn benchmark_lock_free_queue_push() -> BenchmarkResult {
    let queue = LockFreeQueue::new();
    
    for i in 0..1000 {
        while !queue.push(i) {
            std::hint::spin_loop();
        }
    }
    
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

fn benchmark_lock_free_queue_pop() -> BenchmarkResult {
    let queue = LockFreeQueue::new();
    
    for i in 0..1_000_000 {
        while !queue.push(i) {
            std::hint::spin_loop();
        }
    }
    
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

fn benchmark_state_prediction() -> BenchmarkResult {
    let initial = PredictedState {
        eth_price: 3000_0000000000000000,
        gas_price: 30_0000000000000000,
        pool_reserves: [1_000_000_000_000_000_000; 32],
        block_hash: [0; 32],
        timestamp: 0,
        confidence: 10000,
    };
    
    let table = StatePredictionTable::new(0, 0, &initial);
    
    for i in 0..1000 {
        black_box(table.get_state(i));
    }
    
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

fn benchmark_transaction_pipeline() -> BenchmarkResult {
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
    
    let iterations = 100_000;
    let start = Instant::now();
    
    for i in 0..iterations {
        let tx_data = pool.allocate_aligned(256);
        
        while !queue.push(i) {
            std::hint::spin_loop();
        }
        
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
        target_ns: 1000.0,
        passed: avg_ns < 1000.0,
    }
}

#[derive(Debug)]
struct BenchmarkResult {
    name: String,
    iterations: u64,
    total_duration: Duration,
    avg_latency_ns: f64,
    target_ns: f64,
    passed: bool,
}

impl BenchmarkResult {
    fn print(&self) {
        let status = if self.passed { "✅ PASS" } else { "❌ FAIL" };
        println!(
            "{}: {} | {} iterations | {:.2}ns avg (target: {:.2}ns)",
            status, self.name, self.iterations, self.avg_latency_ns, self.target_ns
        );
    }
}
