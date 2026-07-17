// ==============================================================================
// ALLBRIGHT KPI BENCHMARK HARNESS — Phase 1 Evidence Infrastructure
// ==============================================================================
// Framework: Criterion
// Output: bench-results/*.json + HTML report
// ==============================================================================

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use std::time::Instant;
use rand::Rng;

// ---- Import Allbright modules under test ----
use crate::trading_engine::{NewtonRaphsonSolver, calculate_slippage_model, GasCycleMonitor, prioritize_arb_type};
use crate::monolith::{
    calculate_optimal_size, DexPriceState, estimate_yield, calculate_bayesian_bribe,
    get_bloom_pair, add_to_filter, is_pool_compliant, perform_self_healing,
    verify_runway, calculate_sub_bundles,
};
use crate::shield_guardrails::EthicsEngine;

// ==============================================================================
// BENCHMARK 1: SOLVER CONVERGENCE (KPI-14 / profit PILLAR)
// Target: P50 ≤ 18.5 µs per solve
// ==============================================================================
fn bench_solver_convergence(c: &mut Criterion) {
    let solver = NewtonRaphsonSolver::new(20, 1e-8);
    let mut group = c.benchmark_group("solver_convergence");

    for size in [1_000, 10_000, 100_000, 1_000_000].iter() {
        let res_x = *size;
        let res_y = *size;
        group.bench_with_input(
            BenchmarkId::new("babylonian_q_star", size),
            &res_x,
            |b, &sx| {
                b.iter(|| {
                    let (q_star, precision) = calculate_optimal_size(sx, sx, 0, false, 6);
                    (q_star, precision)
                });
            },
        );
    }

    // Newton-Raphson pure solve on f(x) = x^2 - C
    for c_val in [100.0, 10_000.0, 1_000_000.0].iter() {
        group.bench_with_input(
            BenchmarkId::new("newton_raphson_pure", format!("{}", c_val)),
            c_val,
            |b, &c_val| {
                b.iter(|| {
                    solver.solve(&|x| x * x - c_val, c_val.sqrt())
                });
            },
        );
    }

    group.finish();
}

// ==============================================================================
// BENCHMARK 2: LOOP LATENCY (KPI-01 / VELOCITY PILLAR)
// Target: P99 ≤ 19,800 ns
// This measures the full hot-path: price batch → solver → bribe → bundle
// ==============================================================================
fn bench_loop_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("loop_latency");
    group.sample_size(10_000);

    // Simulate 8-pool price batch processing
    let mut prices = [DexPriceState {
        price: 3000.0,
        liquidity: 10_000_000_000_000_000u128,
        gas_cost_wei: 50_000_000_000u64,
    }; 8];

    for i in 0..8 {
        prices[i].price = 3000.0 + (i as f64 * 10.0);
        prices[i].liquidity = 5_000_000_000_000_000u128 + (i as u128 * 1_000_000_000_000_000);
    }

    // Full hot-path simulation
    group.bench_function("full_hot_path_8_pools", |b| {
        b.iter(|| {
            let start = Instant::now();

            // Step 1: SIMD batch price adjustment
            let best = DexPriceState::process_batch(&mut prices);

            // Step 2: Solver convergence
            let (q_star, _) = calculate_optimal_size(10_000_000_000_000_000u128, 10_000_000_000_000_000u128, 0, true, 6);

            // Step 3: Yield estimation
            let _ = estimate_yield(q_star);

            // Step 4: Bayesian bribe
            let _ = calculate_bayesian_bribe(0.05, 0.99, 1.0);

            // Step 5: Sub-bundle splitter
            let _ = calculate_sub_bundles(q_star, 10_000_000_000_000_000u128, 0.5, 1.0, 4);

            start.elapsed().as_nanos()
        });
    });

    group.finish();
}

// ==============================================================================
// BENCHMARK 3: SIMD BATCH PROCESSING (KPI-02 / VELOCITY PILLAR)
// Target: AVX-512 8-lane throughput; scalar fallback documented
// ==============================================================================
fn bench_simd_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_batch");
    group.sample_size(5_000);

    for batch_size in [1, 4, 8, 16, 32].iter() {
        let mut prices = vec![
            DexPriceState {
                price: 3000.0,
                liquidity: 10_000_000_000_000_000u128,
                gas_cost_wei: 50_000_000_000u64,
            };
            *batch_size
        ];

        for i in 0..*batch_size {
            prices[i].price = 3000.0 + (i as f64 * 5.0);
            prices[i].liquidity = 5_000_000_000_000_000u128;
        }

        group.bench_with_input(
            BenchmarkId::new("process_batch", batch_size),
            &mut prices,
            |b, prices| {
                b.iter(|| {
                    let _ = DexPriceState::process_batch(prices);
                });
            },
        );
    }

    group.finish();
}

// ==============================================================================
// BENCHMARK 4: BLOOM FILTER (KPI-25 / SHIELD PILLAR)
// Target: O(1) insert + lookup, P50 ≤ 1 µs
// ==============================================================================
fn bench_bloom_filter(c: &mut Criterion) {
    let mut group = c.benchmark_group("bloom_filter");
    let addresses: Vec<String> = (0..1000)
        .map(|i| format!("0x{:040x}", i))
        .collect();

    // Pre-populate filter
    for addr in &addresses {
        add_to_filter(addr);
    }

    group.bench_function("ofac_lookup_hit", |b| {
        b.iter(|| {
            is_pool_compliant(&addresses[rand::thread_rng().gen_range(0..1000)])
        });
    });

    group.bench_function("ofac_lookup_miss", |b| {
        b.iter(|| {
            is_pool_compliant("0xdeadbeef00000000000000000000000000000000")
        });
    });

    group.bench_function("bloom_insert", |b| {
        b.iter(|| {
            add_to_filter(&format!("0xbench{:040x}", rand::thread_rng().gen::<u64>()));
        });
    });

    group.finish();
}

// ==============================================================================
// BENCHMARK 5: ETHICS AUTHORIZATION (KPI-26 / SHIELD PILLAR)
// Target: P50 ≤ 1 µs
// ==============================================================================
fn bench_ethics_authorize(c: &mut Criterion) {
    let mut group = c.benchmark_group("ethics_authorize");
    let mut engine = EthicsEngine::new();

    group.bench_function("authorize_trade_hot_path", |b| {
        b.iter(|| {
            let _ = engine.authorize_trade(0.01, 0.05, 0.01);
        });
    });

    // Cold path (after limits are hit)
    engine.set_daily_loss_limit(0.01);
    engine.record_loss(0.02);
    group.bench_function("authorize_trade_halt_path", |b| {
        b.iter(|| {
            let _ = engine.authorize_trade(0.001, 0.001, 0.001);
        });
    });

    group.finish();
}

// ==============================================================================
// BENCHMARK 6: CACHE-LINE FALSE SHARING (KPI-32 / EFFICIENCY PILLAR)
// Target: ≥10% throughput improvement for 64-byte aligned atomics
// ==============================================================================
fn bench_cache_alignment(c: &mut Criterion) {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::thread;

    let mut group = c.benchmark_group("cache_alignment");
    group.sample_size(1_000);

    // Aligned (64-byte cache line) — matches #[repr(align(64))] in monolith.rs
    group.bench_function("aligned_atomic_8_threads", |b| {
        b.iter(|| {
            let counter = Box::new(AtomicU64::new(0));
            let handles: Vec<_> = (0..8)
                .map(|_| {
                    let c = counter.clone();
                    thread::spawn(move || {
                        for _ in 0..10_000 {
                            c.fetch_add(1, Ordering::Relaxed);
                        }
                    })
                })
                .collect();
            for h in handles { h.join().unwrap(); }
            counter.load(Ordering::Relaxed)
        });
    });

    // Unaligned (false sharing induced by packing 8 atomics in one cache line)
    group.bench_function("false_shared_atomic_8_threads", |b| {
        b.iter(|| {
            #[repr(C)]
            struct Unaligned {
                counters: [AtomicU64; 8],
            }
            let shared = Box::new(Unaligned {
                counters: [
                    AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0),
                    AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0),
                ],
            });
            let handles: Vec<_> = (0..8)
                .map(|i| {
                    let s = unsafe { &*(shared.as_ref() as *const _ as *const AtomicU64).add(i) };
                    thread::spawn(move || {
                        for _ in 0..10_000 {
                            s.fetch_add(1, Ordering::Relaxed);
                        }
                    })
                })
                .collect();
            for h in handles { h.join().unwrap(); }
            shared.counters.iter().map(|c| c.load(Ordering::Relaxed)).sum::<u64>()
        });
    });

    group.finish();
}

// ==============================================================================
// BENCHMARK 7: SLIPPAGE MODEL (KPI-37 / EFFICIENCY PILLAR)
// ==============================================================================
fn bench_slippage_model(c: &mut Criterion) {
    let mut group = c.benchmark_group("slippage_model");

    for (amount, liquidity) in [
        (100.0, 1_000_000.0),
        (10_000.0, 10_000_000.0),
        (1_000_000.0, 100_000_000.0),
    ] {
        group.bench_with_input(
            BenchmarkId::new("constant_product", format!("{}/{}", amount, liquidity)),
            &(amount, liquidity),
            |b, &(a, l)| {
                b.iter(|| calculate_slippage_model(a, l));
            },
        );
    }

    group.finish();
}

// ==============================================================================
// BENCHMARK 8: GAS CYCLE MONITOR (KPI-38 / EFFICIENCY PILLAR)
// ==============================================================================
fn bench_gas_cycle_monitor(c: &mut Criterion) {
    let monitor = GasCycleMonitor {
        gas_ema: 30.0,
        sensitivity: 0.1,
    };

    c.bench_function("gas_bottom_detection", |b| {
        b.iter(|| {
            monitor.is_bottom_detected(25.0);
            monitor.is_bottom_detected(29.0);
        });
    });
}

// ==============================================================================
// BENCHMARK 9: ARB PRIORITIZATION (KPI-41 / EFFICIENCY PILLAR)
// ==============================================================================
fn bench_arb_prioritization(c: &mut Criterion) {
    c.bench_function("prioritize_triangular", |b| {
        b.iter(|| prioritize_arb_type("TRIANGULAR", 0.06));
    });
    c.bench_function("prioritize_cross_dex", |b| {
        b.iter(|| prioritize_arb_type("CROSS_DEX", 0.15));
    });
    c.bench_function("prioritize_jit", |b| {
        b.iter(|| prioritize_arb_type("JIT_LIQUIDITY", 0.01));
    });
}

// ==============================================================================
// BENCHMARK 10: BAYESIAN BRIBE (KPI-05 / EFFICIENCY PILLAR)
// ==============================================================================
fn bench_bayesian_bribe(c: &mut Criterion) {
    let mut group = c.benchmark_group("bayesian_bribe");

    for (profit, success_rate, congestion) in [
        (0.01, 0.99, 0.5),
        (0.10, 0.95, 1.0),
        (1.0, 0.85, 1.5),
    ] {
        group.bench_with_input(
            BenchmarkId::new("bribe_calc", format!("{:.2}_{:.2}", profit, success_rate)),
            &(profit, success_rate, congestion),
            |b, &(p, sr, cg)| {
                b.iter(|| calculate_bayesian_bribe(p, sr, cg));
            },
        );
    }

    group.finish();
}

// ==============================================================================
// BENCHMARK 11: SELF-HEALING (KPI-09 / CONTINUITY PILLAR)
// ==============================================================================
fn bench_self_healing(c: &mut Criterion) {
    let mut latency: f64 = 0.060;
    c.bench_function("self_heal_trigger", |b| {
        b.iter(|| {
            perform_self_healing(&mut latency);
        });
    });
}

// ==============================================================================
// BENCHMARK 12: RUNWAY VERIFICATION (KPI-08 / CONTINUITY PILLAR)
// ==============================================================================
fn bench_runway(c: &mut Criterion) {
    c.bench_function("runway_check_low_occupancy", |b| {
        b.iter(|| verify_runway(4500, 10000));
    });
    c.bench_function("runway_check_high_occupancy", |b| {
        b.iter(|| verify_runway(9500, 10000));
    });
}

// ==============================================================================
// BENCHMARK 13: SUB-BUNDLE SPLITTER (KPI-33 / profit PILLAR)
// ==============================================================================
fn bench_sub_bundles(c: &mut Criterion) {
    let mut group = c.benchmark_group("sub_bundles");

    for q_star in [1_000_000_000_000_000_000u128, 10_000_000_000_000_000_000u128] {
        group.bench_with_input(
            BenchmarkId::new("split", format!("{}", q_star)),
            &q_star,
            |b, &qs| {
                b.iter(|| {
                    calculate_sub_bundles(qs, 10_000_000_000_000_000u128, 0.5, 1.0, 4)
                });
            },
        );
    }

    group.finish();
}

// ==============================================================================
// CRITERION CONFIGURATION
// ==============================================================================
criterion_group! {
    name = kpi_benchmarks;
    config = Criterion::default()
        .warm_up_time(std::time::Duration::from_secs(2))
        .measurement_time(std::time::Duration::from_secs(5))
        .save_baseline("bench-results/baseline.json")
        .with_output_color(true);
    targets =
        bench_solver_convergence,
        bench_loop_latency,
        bench_simd_batch,
        bench_bloom_filter,
        bench_ethics_authorize,
        bench_cache_alignment,
        bench_slippage_model,
        bench_gas_cycle_monitor,
        bench_arb_prioritization,
        bench_bayesian_bribe,
        bench_self_healing,
        bench_runway,
        bench_sub_bundles,
}

criterion_main!(kpi_benchmarks);
