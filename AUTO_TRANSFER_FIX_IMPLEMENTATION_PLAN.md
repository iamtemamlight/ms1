# Auto-Transfer Workflow — P0 Fix Implementation Plan

Status: **Implemented & type-checked** (2026-07-14).

This document records the P0 fix for the missing auto-transfer trigger in the C2 backend and
how the new module is wired into `backend/main.rs`.

## What was missing (P0)

| Issue | Before | After |
|-------|--------|-------|
| No auto-transfer trigger | ❌ not implemented | `check_and_trigger_auto_transfer()` |
| No threshold-check automation | ❌ not implemented | 30s `run_periodic_check()` background task |
| No smart contract integration | ⚠️ absent | `simulate_smart_contract_call()` placeholder (#PRODUCTION INTEGRATION POINT) |
| No real-time profit updates | ⚠️ absent | `GET /api/auto-transfer/stream` SSE endpoint |

## Files

- `backend/auto_transfer_scheduler.rs` — new module (the implementation).
- `backend/main.rs` — wired in (module decl, init + spawn, 3 HTTP routes).
- `AUTO_TRANSFER_FIX_IMPLEMENTATION_PLAN.md` — this file.

## Module API (`backend/auto_transfer_scheduler.rs`)

```rust
pub struct AutoTransferConfig {
    pub threshold_eth: f64,        // default 0.05
    pub check_interval_secs: u64,  // default 30
    pub cooldown_secs: u64,        // default 300 (anti-double-sweep)
    pub max_transfer_eth: f64,     // default 1.0 per sweep
    pub enabled: bool,             // default FALSE (must be armed explicitly)
}

impl AutoTransferScheduler {
    pub async fn check_and_trigger_auto_transfer(&self) -> AutoTransferEvent;
    pub async fn manual_trigger(&self) -> AutoTransferEvent;
    pub async fn run_periodic_check(&self);   // 30s loop; never returns
}

pub fn init_global(config: AutoTransferConfig) -> &'static AutoTransferScheduler;
pub fn global() -> Option<&'static AutoTransferScheduler>;
pub fn accumulated_profit_eth() -> f64;       // sums TRADE_RECORDS.net_profit_eth
```

Accumulated profit is read from the existing `TRADE_RECORDS` global (same source as
`get_profit_metrics` in `main.rs`), so no new data path was introduced.

## How it was wired into `main.rs`

1. **Module declaration** (near other `mod` statements):
   ```rust
   mod auto_transfer_scheduler;
   ```

2. **Init + spawn** (in `async fn main`, right after the HTTP/C2 server setup). The
   scheduler is **disabled by default** and only auto-runs when explicitly armed via env:
   ```rust
   let auto_transfer_enabled = std::env::var("AUTO_TRANSFER_ENABLED")
       .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
       .unwrap_or(false);
   let auto_transfer = auto_transfer_scheduler::init_global(
       auto_transfer_scheduler::AutoTransferConfig { enabled: auto_transfer_enabled, ..Default::default() },
   );
   if auto_transfer_enabled {
       let _auto_transfer_handle = tokio::spawn(auto_transfer.run_periodic_check());
   }
   ```

3. **HTTP routes** (added to the `Router::new()` chain):
   ```rust
   .route("/api/auto-transfer/status",  get(auto_transfer_scheduler::get_status))
   .route("/api/auto-transfer/trigger", post(auto_transfer_scheduler::post_trigger))
   .route("/api/auto-transfer/stream",  get(auto_transfer_scheduler::stream_events))
   ```

### Exposed endpoints

| Method | Path | Purpose |
|--------|------|---------|
| GET  | `/api/auto-transfer/status`  | JSON: threshold, accumulated/swept/unswept profit, totals, `next_check_in_secs`, `last_event`. |
| POST | `/api/auto-transfer/trigger` | Manual sweep of the **unswept** delta. Requires header `x-auto-transfer-key` matching env `AUTO_TRANSFER_ADMIN_KEY` (fail-closed: 403 if unset/mismatched). Enforces cooldown. |
| GET  | `/api/auto-transfer/stream`  | Server-Sent Events: `transfer` events on each sweep + `profit` heartbeat every 5s. |

### Sweep accounting (prevents double-sweep)

Accumulated profit is the sum of `TRADE_RECORDS.net_profit_eth`, which is **never decremented**.
To avoid re-sweeping already-moved funds, the scheduler tracks `swept_profit_eth` in
`AutoTransferState` and only acts on the **unswept delta** = `accumulated - swept`:
`unswept >= threshold_eth` gates a sweep, and each successful sweep adds its amount to
`swept_profit_eth`. The status endpoint exposes `swept_profit_eth` and `unswept_profit_eth`.

## Smart contract integration (production)

`simulate_smart_contract_call(amount_eth)` is the only placeholder. Replace its body with a real
on-chain sweep. The `ethers` crate (already a backend dependency) is the intended client:

```rust
// PRODUCTION INTEGRATION POINT (in simulate_smart_contract_call):
let provider = ethers::prelude::Provider::<Http>::try_from(rpc_url)?;
let signer = Arc::new(provider.clone().with_signer(key));
let contract = AutoTransfer::new(contract_addr, signer);
let tx = contract.sweep_profit(parse_eth(amount_eth)).send().await?;
Ok(format!("{:#x}", tx.tx_hash()))
```

The contract interface expected:

```solidity
interface IAutoTransfer {
    function sweep_profit(uint256 amountWei) external returns (bytes32 txHash);
}
```

Recommended guards before going live: multisig / `onlyRole(SWEEPER)` on the contract, amount
cap == `max_transfer_eth`, and a circuit breaker that halts sweeps when `risk_mode` is elevated
(consult `m015`/`shield_guardrails`).

## Frontend (real-time updates)

The backend exposes SSE at `GET /api/auto-transfer/stream`. On the dashboard, open an
`EventSource` and react to the two event types:

```ts
const es = new EventSource("/api/auto-transfer/stream");
es.addEventListener("profit", (e) => updateProfit(JSON.parse(e.data)));
es.addEventListener("transfer", (e) => {
  const ev = JSON.parse(e.data);
  if (ev.status === "Completed") toast(`Swept ${ev.amount_eth} ETH — ${ev.tx_hash}`);
});
```

`AutoTransferEvent` shape (over the wire):
```json
{ "at": "2026-07-14T..Z", "status": "Idle|InFlight|Completed|Failed",
  "profit_eth": 0.0, "amount_eth": 0.0, "tx_hash": null, "error": null }
```

## Verification

- The module was type-checked in isolation against the project's exact dependency versions
  (axum 0.7, tokio 1.32, tokio-stream 0.1, serde 1.0, chrono 0.4, hex 0.4, rand 0.8,
  futures 0.3) — compiles clean.
- A full `cargo check` of `backend/` could **not** be run in this environment: the repo's
  `Cargo.toml` has pre-existing invalid feature flags for the locked crate versions
  (`tonic` `tls-native-tls`, `ethers` `native-tls`). That is unrelated to this change — this
  module introduces **no new dependencies**. Fix those two feature flags (e.g. `tonic` →
  `tls-native-roots`, `ethers` → `rustls`) to restore a buildable tree, then run:
  ```sh
  cargo check --bin allbright-c2-backend
  ```

## Next steps for production

1. Replace `simulate_smart_contract_call` with the real `IAutoTransfer::sweep_profit` call.
2. Add the `IAutoTransfer` Solidity contract + deploy address to `contracts/`.
3. Wire dashboard `EventSource` against `/api/auto-transfer/stream`.
4. Gate sweeps behind `risk_mode` / multisig before enabling in LIVE mode.
5. Set `AUTO_TRANSFER_ADMIN_KEY` and require it on `POST /api/auto-transfer/trigger` (already fail-closed).
6. Arm the auto-loop only in LIVE mode via `AUTO_TRANSFER_ENABLED=true` (default disabled).
7. Fix the two pre-existing `Cargo.toml` feature flags so CI can build.
