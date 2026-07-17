# AllBright Performance Instrumentation Design

**Date:** 2026-07-13  
**Status:** DESIGN DOCUMENT — PENDING COMMANDER APPROVAL  
**Objective:** Create trustworthy performance measurement framework with explicit boundaries

---

## 1. Design Principles

### 1.1 Core Principles
1. **Explicit Boundaries:** Every measurement must have clearly defined START and END points
2. **Identical Methodology:** Legacy and UPGRADE4 must be measured with identical boundaries
3. **Monotonic Clock:** Use `std::time::Instant` (Rust) or `process.hrtime.bigint()` (Node.js) exclusively
4. **No Optimization Without Measurement:** Do not claim improvement without instrumentation
5. **Transparent Methodology:** All measurement code must be reviewable

### 1.2 Anti-Patterns to Avoid
- Mixing internal and external latency in a single metric
- Using `SystemTime` (subject to NTP jumps)
- Measuring only "happy path" without cold-start data
- Reporting mean without variance
- Claiming end-to-end improvement without measuring end-to-end

---

## 2. KPI Group Definitions

### KPI Group A — Internal Engine Latency
**Boundary:** START = Opportunity signal received → END = Transaction signed and ready to submit  
**Excludes:** All RPC calls, submission, confirmation  
**Purpose:** Measure pure application logic performance

#### Sub-KPIs

| KPI ID | Name | Start Point | End Point | Unit |
|--------|------|-------------|-----------|------|
| A-01 | Opportunity Detection Latency | `on_opportunity_signal()` | `after_scan_complete()` | µs |
| A-02 | Strategy Calculation Latency | `before_strategy_calc()` | `after_gas_estimate()` | µs |
| A-03 | Simulation/Validation Latency | `before_simulate()` | `after_simulate()` | µs |
| A-04 | Transaction Preparation Latency | `before_prepare_tx()` | `after_sign_tx()` | µs |
| A-05 | **Group A Total (Mean)** | `START` | `END` | µs |
| A-06 | **Group A Total (P50)** | `START` | `END` | µs |
| A-07 | **Group A Total (P95)** | `START` | `END` | µs |
| A-08 | **Group A Total (P99)** | `START` | `END` | µs |
| A-09 | **Group A Total (P100)** | `START` | `END` | µs |

**Implementation:**
```rust
// Rust example
let t0 = Instant::now();
// ... opportunity detection ...
let detection_latency = t0.elapsed().as_micros() as u64;

let t1 = Instant::now();
// ... strategy calculation ...
let strategy_latency = t1.elapsed().as_micros() as u64;

let t2 = Instant::now();
// ... simulation ...
let sim_latency = t2.elapsed().as_micros() as u64;

let t3 = Instant::now();
// ... tx preparation ...
let prep_latency = t3.elapsed().as_micros() as u64;

let total = detection_latency + strategy_latency + sim_latency + prep_latency;
```

---

### KPI Group B — Blockchain Interaction Latency
**Boundary:** START = RPC call initiated → END = Response received  
**Excludes:** Internal processing between RPC calls  
**Purpose:** Measure external dependency latency

#### Sub-KPIs

| KPI ID | Name | Start Point | End Point | Unit |
|--------|------|-------------|-----------|------|
| B-01 | RPC Response Latency | `before_rpc_call()` | `after_rpc_response()` | ms |
| B-02 | Transaction Submission Latency | `before_sendRawTransaction()` | `after_mempool_ack()` | ms |
| B-03 | Network Propagation Latency | `tx_submitted` | `50%_network_aware()` | ms |
| B-04 | Block Confirmation Latency | `tx_submitted` | `block_included()` | seconds |
| B-05 | **Group B Total (End-to-End)** | `START` | `END` | seconds |

**Implementation:**
```rust
// Rust example
let t0 = Instant::now();
let response = rpc_client.call(method, params).await?;
let rpc_latency = t0.elapsed().as_millis() as u64;

let tx_hash = submit_transaction(signed_tx).await?;
let submission_latency = t0.elapsed().as_millis() as u64;

// Confirmation requires external event
listen_for_confirmation(tx_hash).await?;
let confirmation_latency = t0.elapsed().as_secs() as u64;
```

---

### KPI Group C — End-to-End Opportunity Capture Latency
**Boundary:** START = Opportunity detected (on-chain signal) → END = Execution confirmed on-chain  
**Includes:** Everything  
**Purpose:** Measure business-relevant latency from signal to confirmed profit

#### Sub-KPIs

| KPI ID | Name | Start Point | End Point | Unit |
|--------|------|-------------|-----------|------|
| C-01 | Opportunity Capture Latency | `opportunity_detected` | `execution_confirmed` | seconds |
| C-02 | Signal-to-Prepared Latency | `opportunity_detected` | `tx_signed` | ms |
| C-03 | Prepared-to-Submitted Latency | `tx_signed` | `tx_submitted` | ms |
| C-04 | Submitted-to-Confirmed Latency | `tx_submitted` | `execution_confirmed` | seconds |
| C-05 | **Group C Total (P50)** | `START` | `END` | seconds |
| C-06 | **Group C Total (P95)** | `START` | `END` | seconds |
| C-07 | **Group C Total (P99)** | `START` | `END` | seconds |
| C-08 | **Group C Total (P100)** | `START` | `END` | seconds |

**Implementation:**
```rust
// Rust example
let global_t0 = Instant::now();

// Phase 1: Internal processing
let internal_end = process_opportunity(signal).await?;

// Phase 2: Submission
let submitted = submit_transaction().await?;

// Phase 3: Confirmation (async)
let confirmed = wait_for_confirmation(submitted.tx_hash).await?;

let total_latency = global_t0.elapsed().as_secs_f64();
```

---

## 3. Measurement Protocol

### 3.1 Clock Source
- **Rust:** `std::time::Instant` (monotonic, not subject to NTP)
- **Node.js:** `process.hrtime.bigint()` (nanosecond precision, monotonic)
- **Solidity:** `block.timestamp` (seconds granularity, for on-chain events only)
- **Never use:** `SystemTime`, `Date.now()`, `time.time()`

### 3.2 Timestamp Correlation
```rust
// Every measurement must include:
struct LatencyMeasurement {
    kpi_id: &'static str,
    boundary: Boundary, // A, B, or C
    start_ns: u64,      // monotonic clock
    end_ns: u64,        // monotonic clock
    duration_ns: u64,   // end_ns - start_ns
    clock_source: &'static str,
    measurement_location: &'static str,
    warm_cache: bool,
    load_level: LoadLevel,
}
```

### 3.3 Warm-Cache vs Cold-Start Protocol

| Test Type | Procedure | Report Separately |
|-----------|-----------|-------------------|
| **Cold Start** | Fresh process, empty caches, no prior execution | Yes |
| **Warm Cache** | Run 100 warm-up transactions, then measure | Yes |
| **Steady State** | Continuous load for 60 seconds, report P50/P95/P99 | Yes |

### 3.4 Load Level Protocol

| Load Level | Concurrent Opportunities | Measurement Duration |
|------------|-------------------------|----------------------|
| L1 (Idle) | 1 | 100 transactions |
| L2 (Normal) | 10 | 100 transactions |
| L3 (Peak) | 100 | 100 transactions |
| L4 (Stress) | 1000 | 60 seconds |

---

## 4. Statistical Requirements

### 4.1 Sample Size
- **Minimum:** 1,000 transactions per test condition
- **Preferred:** 10,000 transactions for statistical significance
- **Report:** Mean, median, std dev, min, max, P50, P95, P99, P100

### 4.2 Statistical Tests
- **Normality Test:** Shapiro-Wilk test on latency distribution
- **Outlier Detection:** IQR method (1.5x Q1-Q3)
- **Confidence Interval:** 95% CI for mean
- **Effect Size:** Cohen's d for group comparisons

### 4.3 Reporting Format
```json
{
  "kpi_id": "A-05",
  "n": 1000,
  "mean_us": 116.83,
  "median_us": 100.00,
  "std_dev_us": 45.23,
  "min_us": 23.60,
  "max_us": 500.00,
  "p50_us": 100.00,
  "p95_us": 200.00,
  "p99_us": 300.00,
  "p100_us": 500.00,
  "ci_95_lower": 115.50,
  "ci_95_upper": 118.16,
  "normality_p": 0.03,
  "outliers_removed": 5
}
```

---

## 5. Measurement Implementation

### 5.1 Rust Instrumentation

```rust
// src-tauri/src/latency_measurement.rs
use std::time::Instant;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyMeasurement {
    pub kpi_id: &'static str,
    pub boundary: Boundary,
    pub start_ns: u64,
    pub end_ns: u64,
    pub duration_ns: u64,
    pub clock_source: &'static str,
    pub measurement_location: &'static str,
    pub warm_cache: bool,
    pub load_level: LoadLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Boundary {
    A, // Internal only
    B, // Submission ready
    C, // Execution confirmed
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadLevel {
    L1, // 1 concurrent
    L2, // 10 concurrent
    L3, // 100 concurrent
    L4, // 1000 concurrent
}

pub struct LatencyRecorder {
    measurements: Vec<LatencyMeasurement>,
}

impl LatencyRecorder {
    pub fn new() -> Self {
        Self {
            measurements: Vec::new(),
        }
    }

    pub fn record(&mut self, measurement: LatencyMeasurement) {
        self.measurements.push(measurement);
    }

    pub fn calculate_statistics(&self, kpi_id: &str) -> Statistics {
        let values: Vec<u64> = self.measurements
            .iter()
            .filter(|m| m.kpi_id == kpi_id)
            .map(|m| m.duration_ns)
            .collect();

        if values.is_empty() {
            return Statistics::default();
        }

        let mut sorted = values.clone();
        sorted.sort();

        let sum: u64 = values.iter().sum();
        let mean = sum / values.len() as u64;
        let median = sorted[sorted.len() / 2];
        let min = sorted[0];
        let max = sorted[sorted.len() - 1];

        let variance: u64 = values.iter()
            .map(|&x| (x as i64 - mean as i64).pow(2) as u64)
            .sum::<u64>() / values.len() as u64;

        let std_dev = (variance as f64).sqrt() as u64;

        Statistics {
            n: values.len(),
            mean,
            median,
            std_dev,
            min,
            max,
            p50: percentile(&sorted, 50),
            p95: percentile(&sorted, 95),
            p99: percentile(&sorted, 99),
            p100: sorted[sorted.len() - 1],
        }
    }
}

fn percentile(sorted: &[u64], p: usize) -> u64 {
    let idx = (sorted.len() as f64 * (p as f64 / 100.0)) as usize;
    sorted[idx.min(sorted.len() - 1)]
}

#[derive(Debug, Clone, Default)]
pub struct Statistics {
    pub n: usize,
    pub mean: u64,
    pub median: u64,
    pub std_dev: u64,
    pub min: u64,
    pub max: u64,
    pub p50: u64,
    pub p95: u64,
    pub p99: u64,
    pub p100: u64,
}
```

### 5.2 Node.js Instrumentation

```javascript
// backend/src/latency-measurement.ts
class LatencyRecorder {
  constructor() {
    this.measurements = [];
  }

  record({ kpiId, boundary, startNs, endNs, warmCache, loadLevel }) {
    this.measurements.push({
      kpi_id: kpiId,
      boundary,
      start_ns: startNs,
      end_ns: endNs,
      duration_ns: endNs - startNs,
      clock_source: 'process.hrtime.bigint()',
      measurement_location: kpiId,
      warm_cache: warmCache,
      load_level: loadLevel,
    });
  }

  calculateStatistics(kpiId) {
    const values = this.measurements
      .filter(m => m.kpi_id === kpiId)
      .map(m => m.duration_ns);

    if (values.length === 0) return null;

    values.sort((a, b) => a - b);

    const sum = values.reduce((a, b) => a + b, 0n);
    const mean = sum / BigInt(values.length);

    return {
      n: values.length,
      mean_us: Number(mean) / 1000,
      median_us: Number(values[Math.floor(values.length / 2)]) / 1000,
      std_dev_us: this.calculateStdDev(values, mean),
      min_us: Number(values[0]) / 1000,
      max_us: Number(values[values.length - 1]) / 1000,
      p50_us: Number(this.percentile(values, 50)) / 1000,
      p95_us: Number(this.percentile(values, 95)) / 1000,
      p99_us: Number(this.percentile(values, 99)) / 1000,
      p100_us: Number(values[values.length - 1]) / 1000,
    };
  }

  percentile(sorted, p) {
    const idx = Math.floor(sorted.length * (p / 100));
    return sorted[Math.min(idx, sorted.length - 1)];
  }

  calculateStdDev(values, mean) {
    const squaredDiffs = values.map(v => {
      const diff = BigInt(v) - mean;
      return diff * diff;
    });
    const variance = squaredDiffs.reduce((a, b) => a + b, 0n) / BigInt(values.length);
    return Number(variance ** 0.5n) / 1000;
  }
}

export { LatencyRecorder };
```

---

## 6. Measurement Boundaries Verification

### 6.1 Boundary A Verification Checklist
- [ ] START is exactly `on_opportunity_signal()` — no earlier
- [ ] END is exactly `after_sign_tx()` — no later
- [ ] Zero RPC calls between START and END
- [ ] Zero network I/O between START and END
- [ ] Measurement includes allocation/deallocation overhead

### 6.2 Boundary B Verification Checklist
- [ ] START is exactly `on_opportunity_signal()`
- [ ] END is exactly `after_rpc_response()` for final RPC
- [ ] All RPC calls are included
- [ ] Transaction submission RPC is included
- [ ] No network I/O after final RPC response

### 6.3 Boundary C Verification Checklist
- [ ] START is exactly `on_opportunity_signal()`
- [ ] END is exactly `execution_confirmed` (on-chain event)
- [ ] All phases included: internal + RPC + submission + confirmation
- [ ] Timestamp correlation confirmed (monotonic clock throughout)

---

## 7. Anomaly Detection

### 7.1 P100 < Mean Anomaly
**Detection Rule:** If `P100 < Mean`, measurement is invalid.  
**Possible Causes:**
1. Timer overflow (unlikely with 64-bit nanosecond counters)
2. Measurement switched from system clock to monotonic mid-run
3. Outliers were removed before P100 calculation but included in mean
4. P100 and mean values were swapped in report

**Resolution:**
```rust
fn validate_statistics(stats: &Statistics) -> Result<(), String> {
    if stats.p100 < stats.mean {
        return Err(format!(
            "Invalid statistics: P100 ({}) < Mean ({})",
            stats.p100, stats.mean
        ));
    }
    if stats.p99 < stats.median {
        return Err(format!(
            "Invalid statistics: P99 ({}) < Median ({})",
            stats.p99, stats.median
        ));
    }
    if stats.min > stats.p50 {
        return Err(format!(
            "Invalid statistics: Min ({}) > P50 ({})",
            stats.min, stats.p50
        ));
    }
    Ok(())
}
```

### 7.2 Unit Conversion Errors
**Detection Rule:** All units must be explicitly labeled.  
**Common Errors:**
- Reporting µs as ms (1000x error)
- Reporting ms as µs (0.001x error)
- Mixing units within a single table

**Resolution:**
```rust
enum LatencyUnit {
    Nanoseconds,
    Microseconds,
    Milliseconds,
    Seconds,
}

impl LatencyUnit {
    fn conversion_factor(&self, target: LatencyUnit) -> f64 {
        match (self, target) {
            (LatencyUnit::Nanoseconds, LatencyUnit::Microseconds) => 0.001,
            (LatencyUnit::Nanoseconds, LatencyUnit::Milliseconds) => 0.000001,
            (LatencyUnit::Milliseconds, LatencyUnit::Microseconds) => 1000.0,
            // ... etc
        }
    }
}
```

---

## 8. Reporting Requirements

### 8.1 Mandatory Fields
Every latency report must include:
1. `clock_source` — exact function used
2. `measurement_boundary` — A, B, or C
3. `start_timestamp_ns` — monotonic clock value
4. `end_timestamp_ns` — monotonic clock value
5. `duration_ns` — calculated difference
6. `warm_cache` — boolean
7. `load_level` — L1, L2, L3, or L4
8. `sample_size` — number of measurements
9. `statistical_validity` — pass/fail based on normality test

### 8.2 Prohibited Claims
Do not claim:
- "X times faster" without specifying boundary
- "P100 improvement" if P100 < mean in either dataset
- "End-to-end latency reduced" without measuring end-to-end
- "Production-ready" without live validation

---

## 9. Implementation Timeline

### Phase 1: Instrumentation (Week 1)
- [ ] Add `LatencyRecorder` to Rust backend
- [ ] Add `LatencyRecorder` to Node.js backend
- [ ] Add tracepoints at all measurement boundaries
- [ ] Validate clock source (monotonic)
- [ ] Run cold-start vs warm-cache baseline

### Phase 2: Validation (Week 2)
- [ ] Run 1,000-transaction simulation with instrumentation
- [ ] Measure all three boundaries (A, B, C)
- [ ] Run at L1, L2, L3 load levels
- [ ] Verify P100 >= Mean for all KPIs
- [ ] Generate statistical reports with confidence intervals

### Phase 3: Comparison (Week 3)
- [ ] Measure Legacy with identical boundaries
- [ ] Measure UPGRADE4 with identical boundaries
- [ ] Calculate delta with statistical significance
- [ ] Re-score confidence after instrumentation

### Phase 4: Live Validation (Week 4)
- [ ] Shadow-fork test with live RPC
- [ ] Compare simulation vs shadow-fork results
- [ ] Document simulation-to-live gap
- [ ] Final confidence assessment

---

## 10. Approval Required

| Item | Required For |
|------|--------------|
| Instrumentation design | Adding measurement code to production |
| Boundary definitions | Any latency comparison |
| Statistical methodology | Publishing performance claims |
| Live validation plan | Deployment decisions |

---

*Design document produced by AllBright Performance Architect. No code changes without Commander approval.*