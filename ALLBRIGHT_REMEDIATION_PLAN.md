# ALLBRIGHT Remediation Plan
**Basis:** ALLBRIGHT_VALIDATION_EVALUATION.md against NEWREFINE checklist  
**Constraint:** Fixes are edits to existing files; no new runtime dependencies beyond what is already declared.

---

## PHASE 0 — Enable Real Enforcement (2–3h)
Do this first because the rest of the system depends on it.

| # | Action | File | Change |
|---|--------|------|--------|
| 1 | Wire RelationshipMatrix into Copilot loop | `backend/main.rs:run_copilot_decision_loop` | Replace the `_impact = matrix_guard.evaluate_impact(...)` discard with a real decision branch: if impact_score > 0.7, skip optimization cycle; else proceed. |
| 2 | Replace dummy EthicsEngine gate with real pre-trade gate | `backend/main.rs:840` | Replace `self.ethics_engine.authorize_trade(0.01, 0.05, 0.01)` with `authorize_trade(profit_eth, gas_eth, size_eth)` pulled from the runner’s KPI/market data. Treat rejected result as hard stop before simulator/copilot AI call. |
| 3 | Convert ConstitutionGuard from PROPOSED to live gate | `backend/constitution_guard.rs` | Instantiate in `CentralC2Server::new` and call `constitution_guard.validate_objective(...)` before any KPI broadcast. Return early on violation. |

---

## PHASE 1 — Core Trading Stubs → Functional Modules (4–6h)
The biggest deployment blocker is that 6+ trading modules report success without doing work.

### 1.1 Arbitrage Detector (M022)
**File:** `backend/m022_arbitrage_detector.rs`
- Replace `execute()` success-only path with logic that reads the last N entries from `PoolDispatcher::pool_cache` and emits opportunities only when `expected_output - gas_estimate > profit_threshold`.
- Return structured data: `{detected: bool, pairs: [...], ev: f64}`.

### 1.2 Price Monitor (M024)
**File:** `backend/m024_price_monitor.rs`
- Implement periodic RPC polling via `RpcConsensus` for configured token pairs.
- Cache last price with timestamp; emit stale flag if > 2 blocks old.

### 1.3 Trade Executor (M025)
**File:** `backend/m025_trade_executor.rs`
- Wire executor to `m001_wallet_management::WmeService::execute_profit_sweep` and `private_mempool::PrivateMempool::submit_bundle`.
- Record outcome into `profit_cache` and `RunnerKpiMatrix`.

### 1.4 Slippage Calculator (M027)
**File:** `backend/m027_slippage_calculator.rs`
- Implement `calculate_slippage_model(amount_q, liquidity_l)` using constant-product formula from `trading_engine.rs:11`.
- Expose as callable function for pre-trade simulation.

### 1.5 Liquidity Analyzer (M023)
**File:** `backend/m023_liquidity_analyzer.rs`
- Query `eth_getBalance` + ERC-20 `balanceOf` for top pools via `BalanceSimulator::fetch_balance`.
- Flag pools where `reserve_usd < trade_size * 2`.

---

## PHASE 2 — Simulation Wired Into Hot Path (2–3h)

### 2.1 BalanceSimulator pre-trade gate
**File:** `backend/main.rs` or new `backend/simulation_gate.rs`
- Before any opportunity is passed to executor, call:
  ```
  let sim = BalanceSimulator::new(primary_rpc)
      .with_profit_buffer(0.20)
      .simulate_arbitrage(wallet, tokens_in, tokens_out, amounts_in, gas_limit, gas_price_gwei, block)
      .await
  ```
- If `sim.profitable == false` or `sim.net_profit_eth < 0`, drop opportunity immediately.
- Record `sim.warnings` into M132 Copilot Auditor.

### 2.2 Gas Oracle live update
**File:** `backend/m007_gas_oracle.rs`
- Add `update_from_rpc(&mut self, rpc_url: &str)` that queries `eth_gasPrice` and updates `current_price` every 15s.
- Hook this into `run_copilot_decision_loop` 5s tick so gas data is never stale.

---

## PHASE 3 — Settlement Verification & PnL Attribution (3–4h)

### 3.1 Settlement Verifier
**File:** `backend/m001_wallet_management.rs` — extend `WmeService`
- Add `verify_settlement(tx_hash: &str, expected_profit: f64)` that polls `eth_getTransactionReceipt` and post-trade balances, then emits PASS/FAIL to M132 and M133.

### 3.2 Gas Accounting per Trade
**File:** `backend/main.rs` — extend `RunnerKpiMatrix` or add `TradeRecord`
- Store `gas_used`, `gas_price_gwei`, `strategy_id`, `dex_id`, `builder_id` per trade.
- Aggregate into per-strategy / per-DEX / per-builder gas cost KPIs.

### 3.3 PnL Attribution
**File:** `backend/balance_simulator.rs` — add `TradeAttribution` struct
- Fields: `opportunity_id`, `strategy`, `dex`, `builder`, `gross_profit_eth`, `gas_cost_eth`, `flash_loan_fee_eth`, `net_profit_eth`, `timestamp`.
- Emit attribution to fleet KPI stream every trade.

---

## PHASE 4 — Latency Telemetry & Execution Measurement (2–3h)

### 4.1 Pipeline stage timers
**File:** `backend/m009_latency.rs` (extend or create `LatencyTracker`)
- Add microsecond stage markers: `detection → decision → simulation → signing → bundle → relay → inclusion`.
- Persist P50/P99/Max per stage to `RollingWindow` (M131).

### 4.2 Builder/Relay health scoring
**File:** `backend/m067_rpc_consensus.rs`
- Track per-builder inclusion rate and per-relay latency.
- Emit builder score (0-100) used by bundle pricing.

---

## PHASE 5 — Failure Analysis & Continuous Improvement (3–4h)

### 5.1 Failed trade replay
**File:** `backend/m058_shadow_replay.rs`
- After any execution failure, snapshot the opportunity state and replay deterministically through `BalanceSimulator`.
- Compare simulated vs actual; log drift > 5% as anomaly to M132.

### 5.2 Automatic parameter tuning
**File:** `backend/m054_auto_optimizer.rs` (extend existing 25-D matrix)
- Add Bayesian-style budget reallocation: if strategy X wins > 60% over 1h window, increase capital allocation; if loses > 60%, decrease.
- Gate changes through ConstitutionGuard.

### 5.3 Regime detection
**File:** `backend/relationship_matrix.rs` or new `backend/regime_detector.rs`
- Classify current market as LOW_VOL / NORMAL / HIGH_VOL using volatility EWMA.
- Adjust `M009` solver config and `M007` gas strategy automatically.

---

## PHASE 6 — Live Data Integration (4–6h)

### 6.1 Market feeds
**Files to modify:** `backend/m024_price_monitor.rs`, `backend/m022_arbitrage_detector.rs`
- Add V3 subgraph or TheGraph API calls for Uniswap/SushiSwap pools.
- Add Curve pool endpoint integration.
- Merge private mempool (M123) with public mempool in `m028_mempool_watcher.rs`.

### 6.2 Builder feed
**File:** `backend/ai_agents.rs` or new `backend/m028_block_builder.rs`
- Poll `https://relay.flashbots.net/relay/v1/builder/bids` or equivalent.
- Update builder reputation scores used by bundle selection.

---

## Implementation Order
```
Phase 0: Enforcement gates       (2–3h)  ← unblocks safety
Phase 1: Trading stubs           (4–6h)  ← unblocks opportunity detection
Phase 2: Simulation gate         (2–3h)  ← unblocks validation
Phase 3: Settlement/PnL          (3–4h)  ← unblocks treasury
Phase 4: Latency telemetry       (2–3h)  ← unblocks execution QA
Phase 5: Failure analysis        (3–4h)  ← unblocks continuous optimization
Phase 6: Live feeds              (4–6h)  ← final production readiness
Total estimated effort: 20–29h
```

---

## Immediate Next Step
Start in this order:
1. Edit `backend/main.rs:840` to make `EthicsEngine::authorize_trade` a real gate.
2. Edit `backend/main.rs:run_copilot_decision_loop` to wire `RelationshipMatrix`.
3. Edit `backend/m022_arbitrage_detector.rs` to replace stub with real pool-cache scanning.
