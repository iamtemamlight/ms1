# AllBright Shadow Execution Design

**Date:** 2026-07-13  
**Status:** DESIGN DOCUMENT — PENDING COMMANDER APPROVAL  
**Objective:** Validate performance under realistic market conditions without real transactions

---

## 1. Shadow Mode Overview

### 1.1 Definition
Shadow Mode is a simulation environment that:
- Receives real market data from live RPC endpoints
- Detects actual arbitrage opportunities
- Runs internal strategy evaluation
- Simulates execution (no real transactions)
- Records expected outcomes
- Compares predictions against what *would have* happened

### 1.2 Key Principle
**Zero real funds at risk.** Every action is simulated. No transactions are submitted to the blockchain.

### 1.3 Shadow Mode vs Live Mode

| Aspect | Shadow Mode | Live Mode |
|--------|-------------|-----------|
| Market Data | Real (live RPC) | Real (live RPC) |
| Opportunity Detection | Real | Real |
| Strategy Evaluation | Real | Real |
| Transaction Execution | **Simulated only** | Real |
| Gas Cost | Estimated | Actual |
| Profit/Loss | Theoretical | Real |
| Risk | **None** | Capital at risk |

---

## 2. System Architecture

### 2.1 High-Level Flow

```
┌──────────────────────────────────────────────────────────┐
│                    Shadow Execution Engine                 │
└──────────────────────────────────────────────────────────┘
  │
  ├─► Step 1: Fetch Market State
  │   ├─ Call RPC: eth_call (DEX reserves)
  │   ├─ Call RPC: eth_blockNumber
  │   └─ Call RPC: gas_price oracle
  │
  ├─► Step 2: Detect Opportunities
  │   ├─ Scan DEX pairs (Uniswap, SushiSwap, Balancer, etc.)
  │   ├─ Calculate spreads
  │   ├─ Filter by threshold
  │   └─ Generate opportunity candidates
  │
  ├─► Step 3: Strategy Evaluation
  │   ├─ Calculate optimal input (Newton-Raphson)
  │   ├─ Estimate gas cost
  │   ├─ Estimate slippage
  │   ├─ Calculate expected profit
  │   └─ Apply guardrails (loss limits, position limits)
  │
  ├─► Step 4: Simulation
  │   ├─ Simulate DEX swaps (read-only)
  │   ├─ Simulate flash loan (internal accounting)
  │   ├─ Simulate repayment
  │   ├─ Calculate net profit
  │   └─ Record simulation result
  │
  ├─► Step 5: Prediction vs Reality
  │   ├─ Wait for next block
  │   ├─ Check if MEV bot submitted similar tx
  │   ├─ Check if opportunity persisted
  │   ├─ Record what *actually* happened
  │   └─ Compare with prediction
  │
  └─► Step 6: Metrics Recording
      ├─ Internal latency (Group A, B, C)
      ├─ Market accuracy (profit, gas, slippage)
      ├─ Reliability (false positives, false negatives)
      └─ Store in database (Prisma)
```

### 2.2 Component Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                         Shadow Execution Engine                    │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│  ┌──────────────┐   ┌───────────────┐   ┌──────────────────┐   │
│  │ Market Data   │   │ Opportunity   │   │ Strategy Engine  │   │
│  │ Fetcher      │──►│ Scanner       │──►│ (M137)           │   │
│  │              │   │               │   │                  │   │
│  └──────────────┘   └───────────────┘   └────────┬─────────┘   │
│                          │                        │               │
│                          ▼                        ▼               │
│  ┌──────────────┐   ┌───────────────┐   ┌──────────────────┐   │
│  │ Simulation   │   │ Prediction   │   │ Latency          │   │
│  │ Engine       │◄──│ Comparator   │◄──│ Recorder         │   │
│  │              │   │               │   │                  │   │
│  └──────┬───────┘   └───────────────┘   └──────────────────┘   │
│         │                                                         │
│         ▼                                                         │
│  ┌──────────────┐                                                  │
│  │ Metrics DB   │                                                  │
│  │ (Postgres)   │                                                  │
│  └──────────────┘                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## 3. Implementation Requirements

### 3.1 Market Data Fetcher

```rust
// backend/src/shadow/market_fetcher.rs
pub struct MarketFetcher {
    rpc_client: RpcClient,
    dex_registry: DexRegistry,
}

impl MarketFetcher {
    pub async fn fetch_market_state(&self) -> Result<MarketState, Error> {
        let block_number = self.rpc_client.get_block_number().await?;
        let gas_price = self.rpc_client.get_gas_price().await?;
        
        // Fetch reserves for all registered DEX pairs
        let mut reserves = Vec::new();
        for pair in self.dex_registry.pairs() {
            let reserve = self.fetch_reserves(&pair).await?;
            reserves.push(reserve);
        }
        
        Ok(MarketState {
            block_number,
            gas_price,
            reserves,
            timestamp: Instant::now(),
        })
    }
    
    async fn fetch_reserves(&self, pair: &DexPair) -> Result<Reserves, Error> {
        // Call getReserves() on pair contract
        let data = eth_call(self.rpc_client, pair.address, GET_RESERVES_SELECTOR).await?;
        // Parse reserves from return data
        Ok(Reserves { reserve0, reserve1 })
    }
}
```

### 3.2 Opportunity Scanner

```rust
// backend/src/shadow/opportunity_scanner.rs
pub struct OpportunityScanner {
    min_profit_threshold: f64,
    max_gas_price: f64,
    slippage_tolerance: f64,
}

impl OpportunityScanner {
    pub fn scan(&self, market_state: &MarketState) -> Vec<Opportunity> {
        let mut opportunities = Vec::new();
        
        for pair in self.pairs.iter() {
            // Get prices from different DEXes
            let price_a = self.get_price(&pair.dex_a, &pair.token_a, &pair.token_b);
            let price_b = self.get_price(&pair.dex_b, &pair.token_a, &pair.token_b);
            
            // Calculate spread
            let spread = (price_b - price_a) / price_a;
            
            if spread > self.min_profit_threshold {
                let opportunity = Opportunity {
                    token_pair: pair.clone(),
                    buy_dex: pair.dex_a.clone(),
                    sell_dex: pair.dex_b.clone(),
                    price_buy: price_a,
                    price_sell: price_b,
                    spread_pct: spread * 100.0,
                    estimated_profit: self.estimate_profit(spread, pair),
                    estimated_gas: self.estimate_gas(pair),
                    timestamp: market_state.timestamp,
                };
                opportunities.push(opportunity);
            }
        }
        
        opportunities
    }
}
```

### 3.3 Simulation Engine

```rust
// backend/src/shadow/simulation_engine.rs
pub struct SimulationEngine {
    strategy: Arc<Mutex<M137>>,
    dex_router: DexRouter,
}

impl SimulationEngine {
    pub async fn simulate(&self, opportunity: &Opportunity) -> SimulationResult {
        let start = Instant::now();
        
        // Step 1: Calculate optimal input
        let optimal_input = self.strategy.calculate_optimal_input(
            opportunity.buy_price,
            opportunity.sell_price,
            opportunity.slippage_tolerance,
        ).await?;
        
        // Step 2: Estimate gas
        let gas_estimate = self.estimate_gas_cost(
            optimal_input,
            &opportunity.token_pair,
        ).await?;
        
        // Step 3: Simulate swap A (buy)
        let swap_a_result = self.dex_router.simulate_swap(
            &opportunity.buy_dex,
            optimal_input,
            opportunity.token_pair.token_b,
        ).await?;
        
        // Step 4: Simulate swap B (sell)
        let swap_b_result = self.dex_router.simulate_swap(
            &opportunity.sell_dex,
            swap_a_result.output_amount,
            opportunity.token_pair.token_a,
        ).await?;
        
        // Step 5: Calculate net profit
        let gross_profit = swap_b_result.output_amount - optimal_input;
        let net_profit = gross_profit - gas_estimate - swap_a_result.fee - swap_b_result.fee;
        
        let simulation_latency = start.elapsed();
        
        SimulationResult {
            opportunity: opportunity.clone(),
            optimal_input,
            gas_estimate,
            swap_a: swap_a_result,
            swap_b: swap_b_result,
            gross_profit,
            net_profit,
            simulation_latency,
            would_execute: net_profit > 0.0,
        }
    }
}
```

### 3.4 Prediction vs Reality Comparator

```rust
// backend/src/shadow/reality_comparator.rs
pub struct RealityComparator {
    db: Database,
}

impl RealityComparator {
    pub async fn compare(&self, simulation: SimulationResult) -> ComparisonReport {
        // Wait for next block (or timeout after 15 seconds)
        let next_block = self.wait_for_next_block().await;
        
        // Check if MEV bot submitted similar transaction
        let mev_activity = self.check_mev_activity(&simulation).await;
        
        // Check if opportunity persisted
        let actual_market = self.fetch_current_market().await;
        let actual_spread = self.calculate_spread(&actual_market, &simulation.opportunity);
        
        // Record comparison
        let comparison = Comparison {
            predicted_profit: simulation.net_profit,
            actual_profit: if simulation.would_execute {
                // If we would have executed, what was the real outcome?
                self.estimate_actual_profit(&simulation, &actual_market).await
            } else {
                0.0 // Would not have executed
            },
            predicted_spread: simulation.opportunity.spread_pct,
            actual_spread,
            gas_estimate: simulation.gas_estimate,
            actual_gas: self.get_actual_gas_price().await,
            slippage_estimate: simulation.slippage,
            actual_slippage: self.calculate_actual_slippage(&simulation, &actual_market).await,
            mev_competition: mev_activity,
            missed_opportunity: !simulation.would_execute && actual_spread > self.threshold,
            false_positive: simulation.would_execute && actual_spread < self.threshold,
            timestamp: Instant::now(),
        };
        
        self.db.store_comparison(comparison).await;
        comparison
    }
}
```

---

## 4. Metrics to Record

### 4.1 Internal Metrics (Group A)

| Metric | Measurement | Target |
|--------|--------------|--------|
| Opportunity Detection Latency | Time from market fetch to opportunity list | < 50 ms |
| Strategy Calculation Latency | Time from opportunity to simulation complete | < 100 ms |
| Simulation Latency | Time to simulate swaps and calculate profit | < 20 ms |
| Transaction Preparation Latency | Time to prepare unsigned tx | < 5 ms |

### 4.2 Market Metrics

| Metric | Measurement | Target |
|--------|--------------|--------|
| Opportunity Frequency | Opportunities per hour | > 100 |
| Profit Estimate Accuracy | (predicted - actual) / predicted | < 10% error |
| Gas Estimate Accuracy | (estimated - actual) / actual | < 15% error |
| Slippage Prediction Accuracy | (predicted - actual) / actual | < 5% error |

### 4.3 Reliability Metrics

| Metric | Measurement | Target |
|--------|--------------|--------|
| False Positive Rate | Simulations that would profit but didn't | < 5% |
| False Negative Rate | Missed opportunities that would profit | < 10% |
| Failed Simulations | Simulations that error/crash | < 1% |
| Missed Opportunities | Profitable opportunities not detected | < 5% |

---

## 5. Test Protocol

### 5.1 Test Scenarios

| Scenario | Description | Duration | Success Criteria |
|----------|-------------|----------|------------------|
| **S1: Normal Market** | Typical conditions, moderate volatility | 24 hours | > 90% accuracy |
| **S2: High Volatility** | Gas spikes, price swings | 6 hours | > 85% accuracy |
| **S3: Low Liquidity** | Thin markets, high slippage | 12 hours | > 80% accuracy |
| **S4: MEV Competition** | Active bot activity | 12 hours | < 20% missed opportunities |
| **S5: Flash Crash** | Rapid price drop (>10% in 1 block) | 1 hour | Correctly identifies no-trade |

### 5.2 Data Collection

```json
{
  "test_id": "shadow_exec_2026_07_13",
  "scenario": "S1_Normal_Market",
  "start_block": 21847000,
  "end_block": 21847100,
  "duration_seconds": 900,
  "opportunities_detected": 150,
  "simulations_run": 150,
  "would_execute": 45,
  "false_positives": 2,
  "false_negatives": 5,
  "avg_profit_prediction_accuracy": 0.06,
  "avg_gas_accuracy": 0.12,
  "avg_slippage_accuracy": 0.03,
  "mev_competition_rate": 0.15,
  "missed_opportunity_rate": 0.08
}
```

---

## 6. Safety Controls

### 6.1 Hard Stops
- No transaction submission code path is enabled
- All swap calls are `eth_call` (read-only), not `eth_sendRawTransaction`
- Flash loan simulation uses internal accounting only
- No private key required for Shadow Mode

### 6.2 Validation Gates
- Pre-flight check: verify `SHADOW_MODE=true` in environment
- Runtime check: assert no `send_raw_transaction` calls in call stack
- Post-run check: verify zero on-chain transactions sent

### 6.3 Monitoring
- Real-time dashboard showing:
  - Shadow Mode indicator (always green)
  - Opportunities detected per minute
  - Simulation accuracy trends
  - Alert if real transaction is attempted (should never happen)

---

## 7. Implementation Timeline

### Week 1: Core Engine
- [ ] Implement `MarketFetcher`
- [ ] Implement `OpportunityScanner`
- [ ] Implement `SimulationEngine`
- [ ] Implement `RealityComparator`
- [ ] Add SHADOW_MODE flag to config

### Week 2: Metrics & Recording
- [ ] Instrument latency recording (Group A, B, C)
- [ ] Implement `MetricsDB` (Prisma)
- [ ] Create comparison report generator
- [ ] Add unit tests for simulation accuracy

### Week 3: Test Execution
- [ ] Run Scenario S1 (Normal Market, 24h)
- [ ] Run Scenario S2 (High Volatility, 6h)
- [ ] Run Scenario S3 (Low Liquidity, 12h)
- [ ] Generate initial accuracy reports

### Week 4: Analysis & Reporting
- [ ] Compile `SHADOW_EXECUTION_RESULTS.md`
- [ ] Generate `OPPORTUNITY_ACCURACY_REPORT.md`
- [ ] Update `KPI_VALIDATION_REPORT.md` with live data
- [ ] Commander review

---

## 8. Approval Required

| Item | Required For |
|------|--------------|
| Shadow Mode design | Implementing simulation framework |
| Test execution plan | Running 24/48-hour simulations |
| Metrics definition | Publishing accuracy claims |
| Deployment recommendation | Any production deployment |

---

*Design document produced by AllBright Shadow Execution Architect. No code changes without Commander approval.*