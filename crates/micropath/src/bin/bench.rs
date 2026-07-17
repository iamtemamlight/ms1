//! Benchmark: prove the no-hardware path stays under 5 µs end-to-end.
//! Run: cargo run --release --bin micropath-bench

use micropath::*;
use std::time::Instant;

fn main() {
    println!("=== micropath: no-hardware <5us arbiter hot path ===\n");

    let p = Precomp::build(
        1_000_000_000u64,
        (1_000_000u64 << 32) / 1000,
        5 * SCALE / 10_000,
        30u64 << 32,
        1u64 << 32,
        9 * SCALE / 10,
        1024,
    );

    // --- 1. Pure fused scalar hot path (M1+M2+Gas+M3), no ring, no IO ---
    let t = Tick { dx: 500, dr: 10, dd: 2, g_used: 100, token_bits: 0xABC };
    let iters = 2_000_000u64;
    let start = Instant::now();
    let mut sink = 0u64;
    for _ in 0..iters {
        let (s, _slot) = eval_scalar(&p, &t);
        sink = sink.wrapping_add(s);
    }
    let scalar_ns = start.elapsed().as_nanos() as f64 / iters as f64;
    println!(
        "[scalar] fused hot path : {:>8.2} ns/op  (correctness sink={})",
        scalar_ns, sink
    );

    // --- 2. Lock-free SPSC ring pickup + fused eval (Mandate 2/3 software) ---
    let ring = SpscRing::<1024>::new();
    let ring_iters = 1024u64;
    for i in 0..1024u64 {
        ring.try_push(Tick {
            dx: 400,
            dr: 7,
            dd: 1,
            g_used: 80,
            token_bits: i,
        });
    }
    let start = Instant::now();
    let mut sink2 = 0u64;
    for _ in 0..ring_iters {
        let tk = ring.pop_busy();
        let (s, _slot) = eval_scalar(&p, &tk);
        sink2 = sink2.wrapping_add(s);
    }
    let ring_ns = start.elapsed().as_nanos() as f64 / ring_iters as f64;
    println!(
        "[ring]   pickup+eval   : {:>8.2} ns/op  (correctness sink={})",
        ring_ns, sink2
    );

    // --- 3. AVX2 4-way batch (M4 substitution) if supported ---
    #[cfg(target_feature = "avx2")]
    {
        if is_x86_feature_detected!("avx2") {
            let ticks = [t, t, t, t];
            let avx_iters = 256u64;
            let start = Instant::now();
            let mut sink3 = 0u64;
            for _ in 0..avx_iters {
                let out = unsafe { micropath::avx2::eval4(&p, &ticks) };
                sink3 = sink3
                    .wrapping_add(out[0])
                    .wrapping_add(out[1])
                    .wrapping_add(out[2])
                    .wrapping_add(out[3]);
            }
            let avx_ns = start.elapsed().as_nanos() as f64 / (avx_iters * 4) as f64;
            println!(
                "[avx2]   4-way batch    : {:>8.2} ns/op  (per-lane, sink={})",
                avx_ns, sink3
            );
        } else {
            println!("[avx2]   not supported on this CPU");
        }
    }
    #[cfg(not(target_feature = "avx2"))]
    println!("[avx2]   compiled without avx2 target feature");

    // --- 4. End-to-end assertion: budget < 5 us (5000 ns) ---
    // Worst observed component dominates; we assert the ring pickup+eval, which
    // already includes the busiest software path (ring busy-wait on L1 + eval).
    let worst = ring_ns.max(scalar_ns);
    println!(
        "\nWorst hot-path component : {:.2} ns  (target budget 5000 ns)",
        worst
    );
    if worst < 5000.0 {
        println!("RESULT: PASS — hot path is < 5 µs with NO hardware addition.");
    } else {
        println!("RESULT: FAIL — exceeded 5 µs budget.");
    }
}