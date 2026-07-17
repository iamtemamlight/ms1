# ALLBRIGHT Master Six Pillars & 72 KPIs Table by Modules

**Document Version**: 2.0  
**Last Updated**: 2025-01-20  
**System Version**: APEX V119  
**Framework**: Internal (60) + External (12) = 72 Total KPIs

---

## Overview

| Attribute | Value |
|-----------|-------|
| Total Modules | 91 |
| Original Modules | 60 |
| New Modules (M63-M70) | 10 |
| Six Pillars | 6 |
| Total KPIs | 72 |
| **Internal KPIs** (Auto-Optimization) | **60** (5 pillars × 12) |
| **External KPIs** (Market Observation) | **12** (1 pillar × 12) |

---

## The Six Pillars Definition

### Internal Pillars (60 KPIs - Allbright CAN Control & Auto-Optimize)

| Pillar # | Pillar Name | Abbr | Description | Primary Focus | KPIs | Control Type |
|----------|------------|-----|-------------|----------------|------|-------------|
| 1 | VELOCITY | V | Performance | Latency, throughput, parallel execution | 12 | ✅ CONTROL |
| 2 | ALPHA | α | Profit Performance | Profit capture, ROI optimization, arbitrage detection | 12 | ✅ CONTROL |
| 3 | SHIELD | S | Risk Management | Guardrails, circuit breakers, safety limits | 12 | ✅ CONTROL |
| 4 | EFFICIENCY | E | Execution Optimization | Gas optimization, route finding, slippage control | 12 | ✅ CONTROL |
| 5 | CONTINUITY | C | Fleet Operations | Wallet management, state sync, copilot | 12 | ✅ CONTROL |

### External Pillar (12 KPIs - OBSERVE Only, NOT Controlled by Allbright)

| Pillar # | Pillar Name | Abbr | Description | Primary Focus | KPIs | Control Type |
|----------|------------|-----|-------------|----------------|------|-------------|
| 6 | MARKET | M | External Market Conditions | Market environment, regulatory, yield factors | 12 | ⚠️ OBSERVE |

**Key Distinction**: 
- **Internal (VELOCITY, ALPHA, SHIELD, EFFICIENCY, CONTINUITY)**: Allbright CAN control and auto-optimize
- **External (MARKET)**: Allbright CANNOT control - only OBSERVE and adapt strategy accordingly

---

## Master Module-to-Pillar Mapping

### Pillar 1: VELOCITY (Performance) - Internal KPIs 1-12

| Module | File | Function | KPI Range |
|--------|------|----------|----------|
| M09 | latency.rs | Latency Tracking | KPI 1-4 |
| M21 | regional_modules.rs | Cross-Region State Sync | KPI 5-8 |
| M57 | module_57_pool_dispatcher.rs | Pool Dispatcher | KPI 9-12 |

### Pillar 2: ALPHA (Profit Performance) - Internal KPIs 13-24

| Module | File | Function | KPI Range |
|--------|------|----------|----------|
| M02 | module_54_agent.rs | Auto-Optimization Agent | KPI 13-16 |
| M04 | module_58_shadow_replay.rs | Shadow Replay Engine | KPI 17-20 |
| M44 | auto_optimization.rs | DEX Optimization | KPI 21-24 |

### Pillar 3: SHIELD (Risk Management) - Internal KPIs 25-36

| Module | File | Function | KPI Range |
|--------|------|----------|----------|
| M61 | guardrails.rs | Daily Profit Cap | KPI 25-28 |
| M62 | guardrails.rs | Hourly Profit Cap | KPI 29-32 |
| M63 | guardrails.rs | Daily Loss Limit | KPI 33-36 |

### Pillar 4: EFFICIENCY (Execution Optimization) - Internal KPIs 37-48

| Module | File | Function | KPI Range |
|--------|------|----------|----------|
| M16 | engine_modules.rs | Liquidity Depth Assessment | KPI 37-40 |
| M17 | engine_modules.rs | Gas Cycle Timing | KPI 41-44 |
| M18 | engine_modules.rs | Solver Precision Tradeoff | KPI 45-48 |

### Pillar 5: CONTINUITY (Fleet Operations) - Internal KPIs 49-60

| Module | File | Function | KPI Range |
|--------|------|----------|----------|
| M01 | logic.rs | Wallet Management Engine | KPI 49-52 |
| M05 | module_59_state_synchronizer.rs | State Synchronizer | KPI 53-56 |
| M06 | logic.rs | Central C2 Server | KPI 57-60 |

### Pillar 6: MARKET (External) - External KPIs 61-72

| Module | File | Function | KPI Range |
|--------|------|----------|----------|
| OBSERVE | External Data | Market Conditions | KPI 61-64 |
| OBSERVE | External Data | Regulatory Environment | KPI 65-68 |
| OBSERVE | External Data | Yield Factors | KPI 69-72 |

---

## 72 KPIs Detailed Breakdown by Pillar

### Pillar 1: VELOCITY KPIs (KPI 1-12) - Internal

| KPI # | KPI Name | Module | Target | Status |
|-------|---------|--------|--------|--------|
| 1 | Latency P50 | M09 | < 10ms | ✅ Active |
| 2 | Latency P99 | M09 | < 50ms | ✅ Active |
| 3 | Cross-Region Latency | M21 | < 150ms | ✅ Active |
| 4 | Validator Health Score | M22 | > 0.95 | ✅ Active |
| 5 | Jitter Score | M24 | < 5ms | ✅ Active |
| 6 | Gateway Latency | M67 | < 20ms | ✅ Active |
| 7 | Route Availability | M67 | > 99% | ✅ Active |
| 8 | Failover Time | M67 | < 1s | ✅ Active |
| 9 | Throughput Capacity | M57 | > 10K TPS | ✅ Active |
| 10 | Error Rate | M57 | < 0.1% | ✅ Active |
| 11 | Connection Pool Efficiency | M67 | > 90% | ✅ Active |
| 12 | Request Queuing Time | M67 | < 5ms | ✅ Active |

### Pillar 2: ALPHA KPIs (KPI 13-24) - Internal

| KPI # | KPI Name | Module | Target | Status |
|-------|---------|--------|--------|--------|
| 13 | Profit Capture Rate | M02 | > 95% | ✅ Active |
| 14 | Arbitrage Detection Latency | M04 | < 50ms | ✅ Active |
| 15 | Opportunity Conversion | M44 | > 80% | ✅ Active |
| 16 | ROI Optimization Factor | M02 | > 1.5x | ✅ Active |
| 17 | Historical Replay Accuracy | M04 | > 95% | ✅ Active |
| 18 | DEX Route Efficiency | M44 | > 92% | ✅ Active |
| 19 | Alpha Signal Freshness | M53 | < 100ms | ✅ Active |
| 20 | Pattern Recognition Score | M68 | > 90% | ✅ Active |
| 21 | Model Prediction Confidence | M71 | > 0.85 | ✅ Active |
| 22 | Learning Convergence Rate | M71 | < 10 epochs | ✅ Active |
| 23 | Dark Pool Signal Accuracy | M53 | > 85% | ✅ Active |
| 24 | Hidden Liquidity Detection | M68 | > 75% | ✅ Active |

### Pillar 3: SHIELD KPIs (KPI 25-36) - Internal

| KPI # | KPI Name | Module | Target | Status |
|-------|---------|--------|--------|--------|
| 25 | Daily Profit Cap Compliance | M61 | 150K ETH | ✅ Active |
| 26 | Hourly Profit Cap Compliance | M62 | 12.5K ETH | ✅ Active |
| 27 | Daily Loss Limit Compliance | M63 | 50K ETH | ✅ Active |
| 28 | Max Position Enforcement | M64 | 100 ETH | ✅ Active |
| 29 | Circuit Breaker Trigger | M65 | 5 consecutive | ✅ Active |
| 30 | Alert Trigger Rate | M62 | 100% | ✅ Active |
| 31 | Response Mitigation Time | M62 | < 30s | ✅ Active |
| 32 | False Positive Rate | M62 | < 5% | ✅ Active |
| 33 | Escalation Success | M62 | > 95% | ✅ Active |
| 34 | Notification Delivery | M62 | 100% | ✅ Active |
| 35 | Severity Classification | M62 | 100% accurate | ✅ Active |
| 36 | Alert Correlation | M62 | > 80% | ✅ Active |

### Pillar 4: EFFICIENCY KPIs (KPI 37-48) - Internal

| KPI # | KPI Name | Module | Target | Status |
|-------|---------|--------|--------|--------|
| 37 | Slippage Model Accuracy | M16 | > 95% | ✅ Active |
| 38 | Gas Cycle Detection | M17 | 100% | ✅ Active |
| 39 | Solver Convergence | M18 | 99% | ✅ Active |
| 40 | Multi-hop Efficiency | M19 | > 90% | ✅ Active |
| 41 | Arbitrage Priority Score | M20 | > 85% | ✅ Active |
| 42 | Compliance Score | M45 | 100% | ✅ Active |
| 43 | Audit Trail Completeness | M45 | 100% | ✅ Active |
| 44 | Rule Adherence | M45 | > 98% | ✅ Active |
| 45 | Violation Detection Rate | M45 | 100% | ✅ Active |
| 46 | Auto-Remediation Success | M45 | > 90% | ✅ Active |
| 47 | Policy Update Frequency | M45 | Real-time | ✅ Active |
| 48 | Evidence Collection | M45 | 100% | ✅ Active |

### Pillar 5: CONTINUITY KPIs (KPI 49-60) - Internal

| KPI # | KPI Name | Module | Target | Status |
|-------|---------|--------|--------|--------|
| 49 | Wallet Operational Uptime | M01 | > 99.9% | ✅ Active |
| 50 | State Sync Latency | M05 | < 100ms | ✅ Active |
| 51 | Fleet Command Success | M06 | > 99% | ✅ Active |
| 52 | Regional Failover Time | M08 | < 5s | ✅ Active |
| 53 | Fleet Health Score | M66 | > 0.95 | ✅ Active |
| 54 | Node Distribution | M66 | Global | ✅ Active |
| 55 | Active Node Count | M66 | Real-time | ✅ Active |
| 56 | Node Failure Rate | M66 | < 1% | ✅ Active |
| 57 | Recovery Time | M66 | < 30s | ✅ Active |
| 58 | Load Distribution | M66 | Balanced | ✅ Active |
| 59 | Command Success Rate | M72 | > 95% | ✅ Active |
| 60 | Session Continuity | M72 | > 99% | ✅ Active |

### Pillar 6: MARKET KPIs (KPI 61-72) - External (OBSERVE Only)

| KPI # | KPI Name | Source | Target | Status |
|-------|---------|--------|--------|--------|
| 61 | ETH Gas Price | External API | Monitor | ⚠️ OBSERVE |
| 62 | Network Congestion | External API | Monitor | ⚠️ OBSERVE |
| 63 | Market Volatility | External API | Monitor | ⚠️ OBSERVE |
| 64 | TVL Changes | External API | Monitor | ⚠️ OBSERVE |
| 65 | Regulatory Changes | External News | Monitor | ⚠️ OBSERVE |
| 66 | Yield Curve | External API | Monitor | ⚠️ OBSERVE |
| 67 | Liquidity Events | External API | Monitor | ⚠️ OBSERVE |
| 68 | Competitor Activity | External API | Monitor | ⚠️ OBSERVE |
| 69 | Flash Crash Events | External API | Monitor | ⚠️ OBSERVE |
| 70 | MEV Activity | External API | Monitor | ⚠️ OBSERVE |
| 71 | Oracle Price Deviation | External API | Monitor | ⚠️ OBSERVE |
| 72 | Market Anomalies | External API | Monitor | ⚠️ OBSERVE |

---

## Module Summary Table

| Module Range | Pillar | Module Count | Status |
|-------------|--------|-------------|--------|
| M01-M06 | CONTINUITY | 6 | ✅ Active |
| M08-M10 | VELOCITY | 3 | ✅ Active |
| M16-M20 | EFFICIENCY | 5 | ✅ Active |
| M21-M24 | VELOCITY | 4 | ✅ Active |
| M40-M45 | ALPHA/EFFICIENCY | 6 | ✅ Active |
| M53-M57 | ALPHA/VELOCITY | 5 | ✅ Active |
| M61-M65 | SHIELD | 5 | ✅ Active |
| M66-M70 | NEW (Continuity/Velocity/Market) | 5 | ✅ Active |
| M71-M72 | NEW (Alpha/Continuity) | 2 | ✅ Active |

---

## KPI Aggregation by Control Type

| Control Type | Pillars | KPIs | Status |
|--------------|--------|-----|--------|
| **INTERNAL** | VELOCITY, ALPHA, SHIELD, EFFICIENCY, CONTINUITY | 60 | ✅ All Active |
| **EXTERNAL** | MARKET | 12 | ⚠️ OBSERVE Only |

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2025-01-20 | Initial master table with 70 modules and 72 KPIs |
| 2.0 | 2025-01-20 | Restructured: Internal (60 KPIs) + External (12 KPIs) |

---

**Document Owner**: ALLBRIGHT System Architecture  
**Classification**: INTERNAL - CONFIDENTIAL
