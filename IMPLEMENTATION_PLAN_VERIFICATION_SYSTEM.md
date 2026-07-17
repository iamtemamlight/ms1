# ALLBRIGHT Verification & Remediation Implementation Plan

**Classification:** Engineering — Post-Audit Remediation & Verification System Build  
**Date:** 2026-06-25  
**Owner:** Chief External Auditor (this plan) → Engineering  
**Status:** Approved for Execution  
**Governance Model:** Local-first, no external CI/CD dependency

---

## EXECUTIVE SUMMARY

The independent audit identified four categories of failure:

1. **Evidence Fabrication** — every row in the 72-KPI comparison table's Verification column cites files that either do not exist or contain no version of the claimed measurement.
2. **Module Count Inflation** — the "119 modules" are a documentation artifact; the compiled system contains ~9 implemented, ~7 partial/stub, ~46 missing modules.
3. **Telemetry Stubbing** — KPI values are hardcoded atomic initializers, not measured telemetry (e.g., `CACHE_HIT_RATE_PCT: 9840` is a default, not a `perf stat` reading).
4. **Self-Contradicting Audit Documents** — at least 7 Markdown audit reports in the repo make claims ("10/10 DEPLOYMENT READY", "FULLY INTEGRATED") that are explicitly contradicted by other audit documents in the same repo.

This plan resolves all four categories through a **minimum 3-phase, 18-task** remediation program with enforced measurement gates, local verification pipelines, and a reproducible verification harness.

---

## PHILOSOPHY: LOCAL-FIRST GOVERNANCE

**No external CI/CD platform is required for verification enforcement.** All gates, measurements, and evidence generation run locally or on self-hosted infrastructure. The verification system is designed to be:

- **Deterministic** — same inputs produce same outputs on any machine
- **Reproducible** — any engineer can run `./scripts/verify_all.sh` and get identical results
- **Auditable** — every claim traces to a specific script, benchmark, or file hash
- **Self-contained** — no network dependency, no GitHub Actions, no third-party CI

This eliminates the "dirty logic" of trusting a third-party platform to enforce your own system's integrity.

---

## PHASE 1: KILL THE PAPER — REPRODUCIBLE MEASUREMENT INFRASTRUCTURE *(Week 1)*

**Objective:** Replace every hardcoded/claimed-but-absent KPI value with an independently reproducible measurement harness.

### Task 1.1 — Benchmark Harness Framework (`backend/benches/kpi_benchmarks.rs`)

| Attribute | Spec |
|-----------|------|
| **Location** | `backend/benches/kpi_benchmarks.rs` (new file) |
| **Framework** | `criterion` (add to `Cargo.toml` `[dev-dependencies]`) |
| **Output** | JSON/HTML benchmark report committed to `bench-results/` |
| **Execution** | `cargo bench --bench kpi_benchmarks` (local) |

**Required benchmark groups:**

| Benchmark | What it measures | Target from business plan | Pass criterion |
|-----------|-----------------|--------------------------|----------------|
| `bench_solver_convergence` | Newton-Raphson Q* solve time for 10,000 random `(res_x, res_y)` pairs | < 18.5 µs | P50 ≤ 18.5 µs on AVX-512 hardware |
| `bench_loop_latency` | Full hot-path: price fetch → solver → bribe calc → bundle → sign stub | 19,800 ns | P99 ≤ 19,800 ns (or documented hardware floor if AVX-512 unavailable) |
| `bench_simd_batch` | `DexPriceState::process_batch` for 8-pool batch | AVX-512 throughput | Must compile and run; result reported even without AVX-512 (scalar fallback) |
| `bench_bloom_filter` | 100K OFAC filter insert + lookup latency | O(1) | P50 ≤ 1 µs |
| `bench_ethics_authorize` | `EthicsEngine::authorize_trade` cold + hot path | Sub-µs | P50 ≤ 1 µs |
| `bench_cache_alignment` | False-sharing test: aligned vs unaligned atomics under 8 threads | 64-byte alignment wins | ≥ 10% throughput improvement for aligned |

**Acceptance gate:** All benchmarks produce a numerical result. No benchmark may return "N/A" or "stub." Hard failures (compilation without AVX-512) must document the hardware floor, not skip.

---

### Task 1.2 — Telemetry Collector (`backend/kpi_telemetry.rs`)

| Attribute | Spec |
|-----------|------|
| **Location** | `backend/kpi_telemetry.rs` (new file) |
| **Replace** | Hardcoded atomic default values in `monolith.rs` and `main.rs` |

**Design:**
```rust
// Every KPI in the 72-KPI matrix gets a MeasuredKpi entry:
pub struct MeasuredKpi {
    pub id: &'static str,         // e.g. "KPI-01"
    pub name: &'static str,       // e.g. "Loop Latency P50"
    pub pillar: Pillar,           // VELOCITY, ALPHA, ...
    pub unit: &'static str,       // "ns", "µs", "%", "ETH"
    pub value: AtomicF64,         // latest measured value
    pub sample_count: AtomicU64,  // for rolling statistics
    pub last_measured: AtomicU64, // unix millis
}
```

**Migration rule:** Every `static X: AlignedAtomicU64 = AlignedAtomicU64(AtomicU64::new(HARDCODED_VALUE))` must be replaced with a `MeasuredKpi` that is actually written by a measurement function. The hardcoded initializer becomes the `default_value` only; it must not appear in any benchmark or production telemetry output without being overwritten by a real measurement.

**Telemetry emission points:**

| Current state | Required fix |
|---------------|--------------|
| `CACHE_HIT_RATE_PCT: 9840` | Must be overwritten by a real measurement (e.g., reading `perf stat`-style L1-dcache-loads via `perf_event` crate, or at minimum a sampled cache-line-access counter in the hot path). |
| `WIN_RATE_EMA: 9982` | Must be computed from actual trade outcomes in `guardrails.rs` (already has `trades_approved`/`trades_blocked` counters — wire them to the KPI emitter). |
| `BRIBE_EFFICIENCY_PCT: 9650` | Must be computed from actual `calculate_bayesian_bribe()` calls vs actual inclusion outcomes (requires Flashbots inclusion check, not possible in SIM mode — gate on `ENGINE_MODE`). |
| `LAST_MATH_LATENCY_NS: 18500` | Must be measured by `Instant::now()` wrapping the solver call. |

---

### Task 1.3 — Cache-Line False-Sharing Measurement

**Current state:** `#[repr(align(64))]` atomics exist but no measurement proves they help.  
**Action:**  
1. Add a `bench_cache_alignment` criterion (see Task 1.1).  
2. If AVX-512 hardware is present, the benchmark must report μs-per-operation for both aligned and unaligned versions.  
3. If AVX-512 is absent, the fallback scalar version must still be benchmarked to establish a hardware-class floor.

---

## PHASE 2: CLOSE THE EVIDENCE GAPS — PRODUCE THE ARTIFACTS *(Weeks 2–3)*

**Objective:** For each row in the 72-KPI comparison table, produce the measurement artifact that the Verification column claims already exists.

### Task 2.1 — Loop Latency Evidence (`monolith.rs` rdtsc benchmark)

| Current claim | "`monolith.rs` hot-path benchmark (rdtsc timer, 10M iterations)" |
|---------------|---------------------------------------------------------------|
| **Actual** | No rdtsc, no 10M-iteration loop. |
| **Fix** | Add `#[cfg(target_arch = "x86_64")]` rdtsc measurement in `bench_loop_latency` (Task 1.1). The benchmark must: 1) Run 10,000 iterations (10M is impractical for CI; 10K gives P99 confidence). 2) Use `core::arch::x86_64::_rdtsc()` with `_rdtscp()` barrier. 3) Output `p50_ns`, `p99_ns`, `mean_ns`. 4) The resulting `bench-results/loop_latency.json` becomes the authoritative evidence file. |

---

### Task 2.2 — Solver Convergence Evidence (`engine_modules.rs` profiling)

| Current claim | "`engine_modules.rs` Q* solver profiling (AVX-512 VNNI)" |
|---------------|----------------------------------------------------------|
| **Actual** | Solver is correct but unprofiled. No AVX-512 timing. |
| **Fix** | The `bench_solver_convergence` criterion (Task 1.1) must timestamp the solver for each iteration. Additionally: 1) Add `#[cfg(target_feature = "avx512f")]` inner-benchmark that times `_mm512_*` vector math vs scalar fallback. 2) Emit `bench-results/solver_convergence.json` with fields: `iterations_to_converge`, `total_time_ns`, `per_iteration_ns`, `avx512_available`. |

---

### Task 2.3 — Win Rate Evidence (`guardrails.rs` + `module_58_shadow_replay.rs`)

| Current claim | "`guardrails.rs` adverse selection filter logs (M58 shadow-replay validation)" |
|---------------|--------------------------------------------------------------------------------|
| **Actual** | `guardrails.rs` has zero log output for adverse selection. M58 is a struct stub. |
| **Fix** | Two deliverables: 1) **Adverse Selection Filter** (`backend/adverse_selection.rs`, new): Implement a `detect_toxic_flow()` function that checks: incoming trade price vs oracle mid-price deviation > threshold, post-trade price impact analysis, and mempool timing anomalies. Log each decision to a structured format (`tracing::warn!("ADVERSE_SELECTION: ...")`). 2) **M58 completion** (`module_58_shadow_replay.rs`): Implement a `replay_window(n_blocks)` that replays historical trades against the filter and computes win-rate statistics. This module only activates in SIMULATION mode. 3) Evidence artifact: `bench-results/win_rate_validation.json` from the shadow replay pass. |

---

### Task 2.4 — Bribe Efficiency Evidence (`logic.rs` Bayesian optimization)

| Current claim | "`logic.rs` Bayesian tip optimization (Pimlico RPC integration)" |
|---------------|-----------------------------------------------------------------|
| **Actual** | `logic.rs` is WME only. `calculate_bayesian_bribe()` is in `monolith.rs`. No Pimlico client. |
| **Fix** | 1) **Extract `calculate_bayesian_bribe()`** from `monolith.rs` into a proper module `backend/bribe_optimizer.rs` with unit tests. 2) **Add Pimlico integration stub** (`backend/pimlico_gateway.rs`): A structured client that can query Pimlico paymaster sponsorship status. In SIM mode, return mock sponsorship. In PILOT/LIVE, require real credentials. 3) **Bribe efficiency measurement**: Instrument the bribe optimizer to log `(bribe_suggested_wei, actual_inclusion_bribe_wei, included: bool)` per transaction. Compute empirical efficiency = Σ(min(bribe, actual_include) / bribe) / N. 4) Evidence artifact: `bench-results/bribe_efficiency.json`. |

---

### Task 2.5 — Operational Cost Evidence (`fleet_controller.rs`)

| Current claim | "`fleet_controller.rs` automation cost analysis (K8s pod resource tracking)" |
|---------------|--------------------------------------------------------------------------------|
| **Actual** | `fleet_controller.rs` has scaling logic, zero cost tracking. |
| **Fix** | 1) Add `CostTracker` struct to `fleet_controller.rs` that records per-runner resource metrics from K8s `metrics-server` API: cpuRequestMillis, memoryRequestBytes, actual cpuUsageMillis, actual memoryUsageBytes. 2) Compute cost-per-hour using configurable per-resource pricing (cloud provider rate card). 3) Evidence artifact: `bench-results/operational_cost_breakdown.json`. 4) If K8s is unavailable (local dev), the tracker must fall back to a documented pricing model based on RunPod/raw-bare-metal rates. |

---

### Task 2.6 — Cache Hit Rate Evidence (`monolith.rs` perf stat)

| Current claim | "`monolith.rs` cache-line alignment (perf stat L1-dcache-loads)" |
|---------------|-------------------------------------------------------------------|
| **Actual** | 64-byte alignment exists; no measurement. |
| **Fix** | Use the `perf_event` crate (Linux) or `cpuid`-based cache-line sizing to sample L1-dcache-loads and L1-dcache-load-misses around the hot-path loop. On non-Linux (Windows/macOS), use the `cache-line-alignment` benchmark (Task 1.1) as a proxy and document the inability to read hardware perf counters in that environment. Evidence artifact: `bench-results/cache_hit_rate_validation.json`. |

---

### Task 2.7 — Audit Score 100/100 — Formal Verification

| Current claim | "`ACID_ANALYSIS_REPORT.md` formal verification (Coq proof assistant)" |
|---------------|-----------------------------------------------------------------------|
| **Actual** | File does not exist. |
| **Fix** | 1) **Create `verification/` directory** with two sub-artifacts: a) `verification/coq_proofs/`: A Coq proof that `calculate_optimal_size()` converges to within 1% of the true geometric mean for positive `res_x, res_y` (the Babylonian method used is a known fixed-point iteration; the theorem is mechanical). b) `verification/solver_proptest.rs`: A Rust proptest (using `proptest` crate) that generates 10,000 random `(res_x, res_y)` pairs and asserts `|q* - sqrt(res_x*res_y)| / sqrt(res_x*res_y) < 0.01`. 2) The `ACID_ANALYSIS_REPORT.md` is generated by a script that reads both verification outputs and produces a structured report. 3) If Coq toolchain is unavailable, the report must state "Coq not available on this runner — proptest-only score" instead of fabricating 100/100. |

---

### Task 2.8 — Ethical Compliance Evidence (`guardrails.rs` audit log)

| Current claim | "`guardrails.rs` ETHICAL_GUARDRAILS_ACTIVE flag (Module 53 audit log)" |
|---------------|-------------------------------------------------------------------------|
| **Actual** | Flag is named `ETHICS_ENABLED` in `guardrails.rs` and `ETHICAL_GUARDRAILS_ACTIVE` in `monolith.rs`. No audit-log-producing code exists. |
| **Fix** | 1) Rename/alias one flag to eliminate the naming discrepancy. 2) Implement `EthicsAuditLogger` in `guardrails.rs` that appends one JSON line per trade authorization decision to `audit-logs/ethics_{date}.jsonl`: `{timestamp, runner_id, approved, reason, position_size_eth, expected_profit_eth, risk_level}`. 3) The audit log must be machine-readable and queryable. 4) Evidence artifact: `bench-results/ethical_compliance_summary.json` counted from a simulated 10,000-trade replay. |

---

### Task 2.9 — Yield per Runner Evidence (Shadow-Fork Replay)

| Current claim | "`MODULE_AUDIT_REPORT.md` shadow-fork simulation results (30-day mainnet replay)" |
|---------------|-----------------------------------------------------------------------------------|
| **Actual** | `MODULE_AUDIT_REPORT.md` is a module inventory. No yield data exists. |
| **Fix** | 1) Build a **shadow-fork harness** (`backend/benches/shadow_fork_harness.rs`): a. Fork a JSON-RPC endpoint (use `foundry anvil` or a mock RPC in SIM mode). b. Replay a configurable block range. c. For each candidate arb opportunity, run the full engine (solver → bribe → bundle → sign) and record `(profitable, profit_eth, gas_eth, net_eth, included)`. 2) Compute P50, P99, mean profit per trade and per day. 3) Evidence artifact: `bench-results/shadow_fork_yield.json`. 4) The 100 ETH/day/runner claim must be explicitly annotated as a **simulation target** until real mainnet replay data is accumulated. |

---

## PHASE 3: LOCAL VERIFICATION PIPELINE — NO EXTERNAL DEPENDENCIES *(Week 3)*

**Objective:** Make the verification system self-contained: all gates run locally via command-line scripts with no network dependency.

### Task 3.1 — Local Benchmark Gate (`scripts/verify_benchmarks.sh`)

| Attribute | Spec |
|-----------|------|
| **Trigger** | Manual: `./scripts/verify_benchmarks.sh` or pre-commit hook |
| **Execution** | `cargo bench --bench kpi_benchmarks` |
| **Output** | Benchmark report in `bench-results/` |
| **Enforcement** | Script compares each metric against `benchmark-baseline.json`; exits non-zero if any metric regresses > 5% from baseline |

**Baseline policy:**
- Baselines are set by the **first passing local run** after Phase 2 completion.
- Baselines are stored in `benchmark-baseline.json` (committed to repo).
- On hardware where AVX-512 is unavailable, the baseline is annotated `"hardware_class: scalar_fallback"` and compared only against other scalar-fallback baselines.
- Baselines can only be updated by running `./scripts/update_baseline.sh` which requires manual confirmation.

---

### Task 3.2 — KPI Evidence Verification (`scripts/verify_kpi_evidence.sh`)

**Local script** that checks all evidence artifacts exist and are valid:

```bash
#!/usr/bin/env bash
set -euo pipefail

EVIDENCE_FILES=(
  "bench-results/solver_convergence.json"
  "bench-results/loop_latency.json"
  "bench-results/win_rate_validation.json"
  "bench-results/bribe_efficiency.json"
  "bench-results/operational_cost_breakdown.json"
  "bench-results/cache_hit_rate_validation.json"
  "bench-results/ethical_compliance_summary.json"
  "verification/coq_proofs/README.md"
  "bench-results/shadow_fork_yield.json"
)

for f in "${EVIDENCE_FILES[@]}"; do
  if [ ! -f "$f" ]; then
    echo "ERROR: Missing evidence file: $f"
    exit 1
  fi
done

# Validate JSON structure
for f in bench-results/*.json; do
  python3 scripts/validate_kpi_json.py "$f" || exit 1
done

echo "All KPI evidence artifacts present and valid."
```

**`scripts/validate_kpi_json.py`** (new): Validates that each JSON file contains the required fields (schema per artifact), has numeric values (not "N/A"), and timestamps are ≤ 30 days old.

---

### Task 3.3 — Local Proptest Fuzzing Gate

Add `proptest` to `Cargo.toml` `[dev-dependencies]`. Create `backend/tests/kpi_fuzz_tests.rs`:

| Test | What it validates |
|------|-------------------|
| `proptest_solver_accuracy` | 10,000 random positive `(res_x, res_y)` pairs → `|q* - sqrt(x*y)| / sqrt(x*y) < 0.01` |
| `proptest_bribe_bounds` | Bribe never exceeds 30% of profit and never exceeds `MAX_BRIBE_ETH` |
| `proptest_ethics_limits` | Position size, daily loss, consecutive losses always enforced regardless of input order |
| `proptest_bloom_no_fn` | No false negatives on OFAC filter (if address is added, lookup returns true) |
| `proptest_atomic_no_overflow` | Saturating counters never wrap; 2^64 increments on `TOTAL_TRADES` stop at max |

**Execution:** `cargo test --test kpi_fuzz_tests` (local). Add to `scripts/verify_all.sh`.

---

### Task 3.4 — Module Registry Enforcement (`scripts/verify_modules.sh`)

Create `backend/MODULE_REGISTRY.toml`:

```toml
[meta]
version = "V119"
total_modules = 119
last_audit = "2026-06-25"

[[module]]
id = "M01"
name = "Sovereign Solver"
file = "backend/engine_modules.rs"
status = "IMPLEMENTED"   # or PARTIAL, STUB, MISSING, EXTERNAL
kpis = ["KPI-13", "KPI-14", "KPI-15", "KPI-16"]
notes = "NewtonRaphsonSolver struct present and tested"

[[module]]
id = "M53"
name = "Ethical Guardrails"
file = "backend/guardrails.rs"
status = "PARTIAL"
kpis = ["KPI-25", "KPI-26", "KPI-27", "KPI-28"]
notes = "Trade authorization works; not wired into hot path per trade"

# ... all 91 entries
```

**Enforcement script** `scripts/verify_modules.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

# 1. Every file listed in registry exists
# 2. Every IMPLEMENTED module has at least one non-trivial function (≥ 3 lines)
# 3. Every STUB/MISSING module is flagged in output
python3 scripts/verify_module_registry.py backend/MODULE_REGISTRY.toml
```

**`scripts/verify_module_registry.py`**: Parses TOML, checks file existence, counts non-trivial functions per file (simple heuristic: lines of code minus comments/blank lines).

This replaces the "CI job that blocks PRs" with a **local script that must pass before any commit is considered valid.**

---

## PHASE 4: MODULE IMPLEMENTATION GAPS — CLOSE OR RECLASSIFY *(Weeks 4–6)*

**Objective:** Every module claimed in the blueprint must be either implemented, properly stubbed with a `// STUB:` comment, or removed from the blueprint.

### Task 4.1 — Priority Implementations (Stub → Partial)

| Module | Current | Required minimum for PARTIAL |
|--------|---------|------------------------------|
| **M06 Security Shield** | Atomic atomics only | `perform_self_destruct` must zero all critical state; add unit test |
| **M09 Auto-Healer** | Not in compile | Implement `perform_self_healing` with documented heuristic vectors (cache purge, core affinity, context reset) |
| **M38 Bloom Filter** | Missing | Implement in `monolith.rs` or new file; true Bloom filter with 3 hash functions, calibrated false-positive rate |
| **M42 Signature Obfuscation** | Missing | Implement `apply_stealth_signature` with documented non-cryptographic purpose (metadata obfuscation only, NOT security) |
| **M43 Sub-Bundle Splitter** | In monolith, not wired | Caller-visible function returning split amounts; unit test |
| **M47 RPC Multiplexer** | Missing | Implement with `reqwest`-based health check + failover counter |
| **M48 Champion Discovery** | Partial (avg logging) | Implement propagation: champion runner's parameters written to a shared config that other runners read |
| **M49 Predictive Shield** | Atomic stub only | Wire to `module_58_shadow_replay.rs` anomaly detector; update atomic from replay results |
| **M54 Auto Optimization Agent** | Struct stub | Implement `check_npm_compliance` + `optimization_cycle` with measurable output |
| **M58 Shadow Replay** | Struct stub | Implement `replay_window` with anomaly detection producing measurable KPI output |
| **M59 State Synchronizer** | Struct stub | Implement `sync_regions` with measurable latency |
| **M73 Graph Route Optimizer** | File exists | Verify it compiles and produces measurable route cost output |

---

### Task 4.2 — Monolith Integration Decision

**Current state:** `monolith.rs` exists at repo root, contains ~1,000 lines of integrated module code, and is **NOT compiled** (`mod monolith;` is absent from `backend/main.rs`).

**Decision gate:**

| Option | Action | Outcome |
|--------|--------|---------|
| A | Add `mod monolith;` to `backend/main.rs` and resolve compile errors | Monolith becomes the primary engine; multi-file modules must not conflict |
| B | Extract monolith content into proper `backend/` modules and delete `monolith.rs` at root | Cleaner architecture; eliminates duplicate atomics and type definitions |

**Enforcement:** Whichever option is chosen, `MODULE_IMPLEMENTATION_AUDIT.md` must be updated to reflect actual compilation status. `cargo build --release` must succeed with `-D warnings`.

---

## PHASE 5: DOCUMENTATION RECONCILIATION *(Parallel with Phases 2–4)*

**Objective:** Eliminate self-contradicting audit documents and ensure all business plan claims are traceable to code or measurement artifacts.

### Task 5.1 — Audit Document Hierarchy

Establish a single source-of-truth hierarchy:

| Tier | File | Authority | Usage |
|------|------|-----------|-------|
| 1 | `MODULE_IMPLEMENTATION_AUDIT.md` | **ONLY authoritative audit document** | All other audits refer to it |
| 2 | `MODULE_REGISTRY.toml` | Machine-readable module status | Local script enforcement |
| 3 | `bench-results/*.json` | Evidence artifacts | Cited by business plan |
| 4 | `BUSINESS_PLAN_FULL.md` | Marketing / strategic | Must cite Tier 3 artifacts, not self-referential |
| 5 | All other `*_AUDIT*.md`, `*_REPORT*.md`, `*_READINESS*.md` | **SUPERSEDED** | Must contain a header: `# DEPRECATED — See MODULE_IMPLEMENTATION_AUDIT.md` |

**Action:** Every file in Tier 5 must be edited to add the deprecation header. Local script `scripts/verify_docs.sh` must fail if any deprecated audit file makes a claim that contradicts `MODULE_REGISTRY.toml`.

---

### Task 5.2 — 55-KPI → 72-KPI Cleanup

`IMPLEMENTATION_PLAN_72KPI_UNIFICATION.md` lists these as "🔲 NEEDS ACTION":

| Action | Owner | Verification |
|--------|-------|-------------|
| Remove `ANNEX B: 55-KPI Matrix` from `BUSINESS_PLAN_FULL.md` | Docs engineer | `grep -c "55-KPI" BUSINESS_PLAN_FULL.md` returns 0 |
| Replace all "60 modules" → "119 modules" | Docs engineer | `grep -c "60 modules" BUSINESS_PLAN_FULL.md` returns 0 |
| Update module census to V119 | Docs engineer | Cross-checked against `MODULE_REGISTRY.toml` |
| Add new ANNEX with full 72-KPI listing by measured value (not target) | Engineering + Docs | Each KPI entry has a `source` field pointing to a `MeasuredKpi` ID in `kpi_telemetry.rs` |

---

### Task 5.3 — Business Plan Verification Table Rewrite

The current 72-KPI comparison table in `BUSINESS_PLAN_FULL.md` must be re-issued with this column structure:

| KPI | Industry Baseline | Allbright Target | Delta% | Measurement Method | Evidence Artifact | Last Measured | Status |
|-----|-------------------|-----------------|--------|-------------------|-------------------|---------------|--------|
| Loop Latency | 25,000,000 ns | 19,800 ns | −99.92% | `cargo bench bench_loop_latency` | `bench-results/loop_latency.json` | 2026-06-25 | ✅ Measured / ❌ Target only |

**Rule:** The "Allbright Target" column may contain aspirational values, but they must be clearly labeled `[TARGET]` and the "Last Measured" column must show the most recent benchmark date. A value with no measurement date is invalid.

---

## PHASE 6: VERIFICATION SYSTEM — ONGOING LOCAL GOVERNANCE *(Week 6 onward)*

### Task 6.1 — Master Verification Script (`scripts/verify_all.sh`)

The single entrypoint for all verification:

```bash
#!/usr/bin/env bash
set -euo pipefail

echo "=== ALLBRIGHT Verification Pipeline ==="
echo ""

echo "[1/4] Running benchmarks..."
cargo bench --bench kpi_benchmarks || { echo "BENCHMARKS FAILED"; exit 1; }

echo "[2/4] Verifying evidence artifacts..."
./scripts/verify_kpi_evidence.sh

echo "[3/4] Running proptest fuzz tests..."
cargo test --test kpi_fuzz_tests || { echo "FUZZ TESTS FAILED"; exit 1; }

echo "[4/4] Verifying module registry..."
./scripts/verify_modules.sh

echo ""
echo "=== ALL VERIFICATION GATES PASSED ==="
```

This script is the **only enforcement mechanism**. It runs locally. It has no external dependencies. It produces a binary pass/fail result.

---

### Task 6.2 — Cryptographic Evidence Chaining

To prevent evidence tampering:

1. Every local run that produces `bench-results/*.json` computes a SHA-256 hash of each file.
2. Hashes are committed to `bench-results/HASHES_{timestamp}.toml`.
3. `scripts/verify_hashes.py` checks that current file contents match their committed hashes.
4. If a benchmark JSON file is modified without a new local verification run, the script fails with "Evidence file modified outside verification — possible tampering."

---

### Task 6.3 — Quarterly Local Re-Audit Script (`scripts/quarterly_reaudit.sh`)

```bash
#!/usr/bin/env bash
set -euo pipefail

DATE=$(date +%Y-%m-%d)
echo "=== Quarterly Re-Audit: $DATE ==="

# Run full benchmark suite with extended iterations
cargo bench --bench kpi_benchmarks -- --n 100000

# Run proptest with 100,000 cases (vs. 10,000 in CI)
cargo test --test kpi_fuzz_tests -- --test-threads=1 --cases 100000

# Update module registry if new modules detected
python3 scripts/update_module_registry.py

# Generate updated audit report
python3 scripts/generate_audit_report.py > MODULE_IMPLEMENTATION_AUDIT_V2.md

# Compare all 72-KPI values against business plan targets
python3 scripts/compare_kpi_targets.py

# Update baselines if improvements confirmed (requires --force flag)
./scripts/update_baseline.sh --review-diff

echo "Quarterly re-audit complete: bench-results/quarterly_$DATE.json"
```

---

## SUMMARY: TASK DEPENDENCY GRAPH

```
Phase 1 (Measurement Infrastructure)
  ├── Task 1.1 (Benchmark Harness)
  ├── Task 1.2 (Telemetry Collector) ─────┐
  └── Task 1.3 (Cache-Sharing Measurement) │
                                           ▼
Phase 2 (Evidence Artifacts) ◄────────────┘
  ├── Task 2.1 (Loop Latency)
  ├── Task 2.2 (Solver Convergence)
  ├── Task 2.3 (Win Rate)
  ├── Task 2.4 (Bribe Efficiency)
  ├── Task 2.5 (Operational Cost)
  ├── Task 2.6 (Cache Hit Rate)
  ├── Task 2.7 (Audit Score / Formal Verification)
  └── Task 2.8 (Yield per Runner)
          │
          ▼
Phase 3 (Local Verification Pipeline) ◄──────────────────┘
  ├── Task 3.1 (Benchmark Regression Gate)
  ├── Task 3.2 (KPI Evidence Verification)
  ├── Task 3.3 (Proptest Fuzzing Gate)
  └── Task 3.4 (Module Registry Enforcement)

Phase 4 (Module Gaps) ◄──────────────────┘
  ├── Task 4.1 (Priority Implementations)
  └── Task 4.2 (Monolith Integration Decision)

Phase 5 (Documentation) ◄─────────────────┐ (parallel with 2–4)
  ├── Task 5.1 (Audit Hierarchy)
  ├── Task 5.2 (55→72 KPI Cleanup)
  └── Task 5.3 (BP Table Rewrite)

Phase 6 (Governance) ◄────────────────────┘ (starts after Phase 2 complete)
  ├── Task 6.1 (Master Verification Script)
  ├── Task 6.2 (Cryptographic Evidence Chaining)
  └── Task 6.3 (Quarterly Re-Audit Script)
```

---

## CRITICAL PATH & ESTIMATES

| Phase | Duration | Critical dependency | Risk |
|-------|----------|---------------------|------|
| Phase 1 | 3–5 days | `criterion` integration, AVX-512 hardware for benchmarks | Medium — AVX-512 hardware may be unavailable |
| Phase 2 | 2–3 weeks | Phase 1 complete; M58 implementation | High — M58 shadow-replay requires historical block data |
| Phase 3 | 3–5 days | Phase 2 artifacts committed | Low — scripting is mechanical |
| Phase 4 | 2–3 weeks | Module registry approved | Medium — monolith decision could be contentious |
| Phase 5 | 1 week | Phase 3 operational | Low — documentation work |
| Phase 6 | 2 days | Phase 2 complete | Low — automation scripting |

**Total critical path: ~6–8 weeks** from approval to full verification system operational.

---

## FILES TO CREATE / MODIFY (COMPLETE LIST)

### New files (24)

| Path | Purpose |
|------|---------|
| `backend/benches/kpi_benchmarks.rs` | Criterion benchmark suite |
| `backend/benches/shadow_fork_harness.rs` | Shadow-fork replay yield measurement |
| `backend/benches/cache_alignment.rs` | False-sharing benchmark |
| `backend/kpi_telemetry.rs` | Runtime KPI measurement collector |
| `backend/adverse_selection.rs` | Toxic flow detection filter |
| `backend/bribe_optimizer.rs` | Bayesian bribe with measurement hooks |
| `backend/pimlico_gateway.rs` | Pimlico paymaster client (stub for SIM, real for LIVE) |
| `backend/tests/kpi_fuzz_tests.rs` | Proptest fuzz tests |
| `verification/coq_proofs/q_star_convergence.v` | Coq formal verification |
| `verification/solver_proptest.rs` | Proptest for solver accuracy |
| `verification/README.md` | Formal verification report generator |
| `scripts/validate_kpi_json.py` | JSON schema validator for evidence artifacts |
| `scripts/verify_all.sh` | Master verification pipeline (entrypoint) |
| `scripts/verify_benchmarks.sh` | Benchmark regression gate |
| `scripts/verify_kpi_evidence.sh` | Evidence artifact existence + schema check |
| `scripts/verify_modules.sh` | Module registry enforcement |
| `scripts/verify_docs.sh` | Documentation consistency check |
| `scripts/verify_hashes.py` | Cryptographic evidence chain verification |
| `scripts/update_baseline.sh` | Baseline updater (manual confirmation required) |
| `scripts/quarterly_reaudit.sh` | Quarterly re-audit automation |
| `scripts/update_module_registry.py` | Auto-detect new modules |
| `scripts/generate_audit_report.py` | Generates updated audit document |
| `scripts/compare_kpi_targets.py` | KPI vs target comparison |
| `bench-results/README.md` | Evidence directory documentation |

### Files to modify (16)

| Path | Change |
|------|--------|
| `backend/Cargo.toml` | Add `criterion`, `proptest` to dev-deps |
| `backend/main.rs` | Replace hardcoded atomic defaults with `MeasuredKpi` reads |
| `backend/guardrails.rs` | Add `EthicsAuditLogger`; wire `trades_approved`/`blocked` to telemetry |
| `backend/monolith.rs` | Remove hardcoded `CACHE_HIT_RATE_PCT: 9840`, `WIN_RATE_EMA: 9982`, etc.; replace with measurement hooks |
| `backend/logic.rs` | Extract bribe optimizer; add cost tracking hooks |
| `backend/fleet_controller.rs` | Add `CostTracker` |
| `backend/module_58_shadow_replay.rs` | Implement `replay_window()` and anomaly detection |
| `backend/module_59_state_synchronizer.rs` | Implement `sync_regions()` with latency measurement |
| `backend/module_54_agent.rs` | Implement optimization cycle with measurable output |
| `BUSINESS_PLAN_FULL.md` | Rewrite 72-KPI comparison table; remove 55-KPI references; add evidence artifact links |
| `ALLRIGHT_DIRECTORY_MASTER_BLUEPRINT_TABLE.md` | Reconcile module count with `MODULE_REGISTRY.toml` |
| `IMPLEMENTATION_PLAN_72KPI_UNIFICATION.md` | Mark all "NEEDS ACTION" items complete or update owners |
| `MASTER_PILLARS_KPI_TABLE.md` | Each KPI gets `source = MeasuredKpi ID` |
| `.github/workflows/ci-cd.yml` | **DEPRECATE** — mark as superseded by local scripts |
| `.github/workflows/security-gates.yml` | **DEPRECATE** — mark as superseded by local scripts |
| `MODULE_AUDIT_REPORT.md` | Add deprecation header |

### Files to mark deprecated (5)

| Path | Action |
|------|--------|
| `.github/workflows/ci-cd.yml` | Add header: `# DEPRECATED — Superseded by scripts/verify_all.sh (local verification pipeline)` |
| `.github/workflows/security-gates.yml` | Add deprecation header |
| `MODULE_AUDIT_REPORT.md` | Add deprecation header |
| `DESKTOP_INSTALLATION_AUDIT_REPORT.md` | Add deprecation header (or substantiate with build artifacts) |
| `SOVEREIGN_AUDIT_DEPLOYMENT_READINESS.md` | Add deprecation header (or substantiate claims) |

---

## VERIFICATION SIGN-OFF CRITERIA

The implementation plan is **complete** when all of the following are true:

| # | Criterion | Evidence |
|---|-----------|----------|
| 1 | All 9 comparison-table rows have a corresponding `bench-results/*.json` file with a real numeric value (not "N/A", not "stub") | `scripts/verify_kpi_evidence.sh` passes |
| 2 | `./scripts/verify_all.sh` passes green 3 consecutive times on clean checkout | Local execution log |
| 3 | `MODULE_REGISTRY.toml` exists and every claimed module is classified | `scripts/verify_modules.sh` passes |
| 4 | `monolith.rs` is either compiled into the build or deleted; `cargo build --release` succeeds with `-D warnings` | Local build log |
| 5 | All deprecated audit documents carry the deprecation header | `scripts/verify_docs.sh` passes |
| 6 | All "NEEDS ACTION" items in `IMPLEMENTATION_PLAN_72KPI_UNIFICATION.md` are checked | Manual review + script |
| 7 | `ACID_ANALYSIS_REPORT.md` is either generated by `scripts/generate_audit_report.py` or removed from the repo | File existence + content provenance |
| 8 | `ANNEX B` in `BUSINESS_PLAN_FULL.md` is the 72-KPI matrix, not the 55-KPI matrix | `grep "55-KPI" BUSINESS_PLAN_FULL.md` returns 0 |
| 9 | Cryptographic evidence hashes in `bench-results/HASHES_{timestamp}.toml` match current `bench-results/*.json` contents | `scripts/verify_hashes.py` passes |
| 10 | The 72-KPI comparison table's "Allbright Target" column distinguishes `[MEASURED]` from `[TARGET]` values | Business plan review |

---

## IMMEDIATE NEXT STEPS (THIS WEEK)

1. **Engineer lead approves this plan** — no implementation begins before sign-off.
2. **Create `backend/benches/` directory** and add `criterion` to `Cargo.toml`.
3. **Task 1.1 (Benchmark Harness)** is the unblocking prerequisite for all of Phase 2. Prioritize it.
4. **Monolith decision** (Task 4.2, Option A vs B) must be made before Phase 4 begins — it affects all module wiring.
5. **Hardware inventory:** Document the available hardware's AVX-512 status. If unavailable, budget for a cloud instance (AWS `c7i.4xlarge` or equivalent) for Phase 1 benchmarks. Document the hardware class in every benchmark output.

---

**Plan Version:** 2.0  
**Plan Author:** Chief External Auditor (independent)  
**Governance Model:** Local-first, no external CI/CD dependency  
**Plan Review:** Engineering Lead + CTO  
**Next Review:** Upon Phase 2 completion
