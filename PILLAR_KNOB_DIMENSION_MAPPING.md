# ALLBRIGHT Pillar-to-Knob-to-Dimension Mapping

## Layer Hierarchy Alignment
Layer 1: Enterprise Mission (Autonomous Optimization)
Layer 2: Six Strategic Control Knobs (Commander Interface)
Layer 3: Seven KPI Pillars → 78 KPIs (Measurement Layer)
Layer 4: 25 Control Dimensions (AI Intervention Layer)

---

## Pillar-to-Knob Mapping

| Current Pillar | New Knob | Rationale |
|----------------|----------|-----------|
| APEX | Profit SubSystem | Captures revenue, margin, profit optimization, arbitrage capture |
| ALPHA | Growth SubSystem | Identifies opportunities, expands markets, drives adoption |
| VELOCITY | Velocity SubSystem | Measures execution speed, cycle time, throughput |
| SHIELD | Security SubSystem | Risk protection, fraud detection, continuity assurance |
| EFFICIENCY | Efficiency SubSystem | Resource utilization, waste reduction, cost optimization |
| CONTINUITY | Quality SubSystem | System reliability, uptime, service consistency |
| UPGRADE4 | Latency Extension | Ultra-fast pipeline latency verification (KPI-73..78, 0% APEX weight) |

---

## Enterprise Score Calculation

```
Enterprise Performance Score = Σ(PillarScore_i × KnobWeight_i) / Σ(KnobWeight_i)

Where:
- Each PillarScore is 0-100 (normalized from KPIs)
- KnobWeights sum to 1.0 (configurable by Commander)
```

---

## 25 Control Dimensions with Pillar Mappings

### Profit SubSystem (APEX→Profit SubSystem)
| No | Dimension | Unit | Controls |
|----|-----------|------|----------|
| 1 | Pricing Power Index | Score (0–100) | Revenue Growth Rate, Gross Profit Margin, Net Profit Margin, Pricing Effectiveness Index |
| 2 | Cost Efficiency Ratio | % | Operating Cost Ratio, Cost per Unit, Rework Rate, Waste Percentage |
| 3 | Revenue Optimization Rate | % | Revenue Growth Rate, Revenue per Customer |
| 4 | Customer Value Density | USD/customer | Customer Lifetime Value, Revenue per Customer |

### Growth SubSystem (ALPHA→Growth SubSystem)
| No | Dimension | Unit | Controls |
|----|-----------|------|----------|
| 5 | Acquisition Intensity | Spend index | Customer Acquisition Rate, Acquisition Cost |
| 6 | Conversion Optimization Strength | % | Conversion Rate, New User Growth Rate |
| 7 | Retention Stability Factor | % | Retention Rate, Churn Rate |
| 8 | Market Expansion Velocity | Score | Market Share, Geographic Expansion Index |

### Velocity SubSystem (VELOCITY→Velocity SubSystem)
| No | Dimension | Unit | Controls |
|----|-----------|------|----------|
| 9 | Process Automation Level | % | Cycle Time, Throughput Rate, Lead Time |
| 10 | Decision Latency Index | ms/min | Decision Latency, System Response Time |
| 11 | System Throughput Capacity | units/time | Throughput Rate, Automation Rate, Processing Latency |
| 12 | Workflow Optimization Score | Score | Queue Backlog Size, Workflow Completion Rate |

### Efficiency SubSystem (EFFICIENCY→Efficiency SubSystem)
| No | Dimension | Unit | Controls |
|----|-----------|------|----------|
| 13 | Resource Utilization Rate | % | Asset Utilization Rate, Capacity Utilization, Labor Efficiency Ratio |
| 14 | Waste Reduction Index | % | Waste Percentage, Cost Optimization Index |
| 15 | Asset Optimization Level | % | Asset Utilization Rate, Productivity Index, Energy Efficiency Index |
| 16 | Operational Density Score | output/input | Productivity Index, Process Yield, Operational Efficiency Score |

### Security/Continuity Knob (SHIELD+CONTINUITY→Security)
| No | Dimension | Unit | Controls |
|----|-----------|------|----------|
| 17 | Risk Exposure Control Index | Score | Security Incident Rate, Vulnerability Exposure Index |
| 18 | System Resilience Factor | % | System Uptime, MTTR, MTBF |
| 19 | Compliance Enforcement Level | % | Compliance Score, Audit Trail Completeness |
| 20 | Data Integrity Strength | Score | Data Integrity Score, Backup Success Rate |

### Quality SubSystem (QUALITY→Quality SubSystem)
| No | Dimension | Unit | Controls |
|----|-----------|------|----------|
| 21 | Experience Friction Index | Score (↓ better) | CSAT, UX Friction Score |
| 22 | Defect Suppression Rate | % | Defect Rate, First-Time-Right Rate |
| 23 | Service Consistency Level | Score | Service Reliability Index, Consistency Index |
| 24 | Resolution Effectiveness Rate | % | Resolution Effectiveness, Complaint Rate |
| 25 | Continuous Improvement Velocity | % | Continuous Improvement Rate, Improvement Rate |

---

## AI Learning Loop Integration

```
1. Adjust dimensions (not KPIs directly)
2. Observe KPI response (ΔKPI)
3. Calculate causal strength: Impact(d) = ΔKPI / ΔDimension
4. Strengthen useful dimensions, weaken ineffective ones
5. Evolve system configuration
```

---

## Next Steps

- [ ] Map existing KPI implementations to new standardized definitions
- [ ] Create dimension adjustment APIs in backend/
- [ ] Add Enterprise Performance Score to Command Center
- [ ] Build admin UI for dimension management