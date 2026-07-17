# AllBright ACID Module Integration Architecture
**Proposal**: Embed the ACID-CHECKLIST.MD 10-layer audit framework as a first-class debugging system across AllBright modules.

---

## 1. Core Concept

AllBright's existing 60-module architecture provides the *trading intelligence* (arbitrage detection, pool routing, shadow replay, etc.). What it lacks is a unified *safety validation layer* that intercepts every critical operation and runs it through the ACID checklist before execution.

**Proposed Solution**: An `acid_core` subsystem with 10 pluggable `Check` modules — one per ACID layer — that can be invoked:
- **Proactively**: before any trade execution, deployment, or config change
- **Reactively**: when alerts trigger, to diagnose root cause against ACID layers
- **Continuously**: as a background daemon auditing fleet state

---

## 2. ACID Layer → Module Mapping

### Layer 1: Build Integrity → `backend/build_guard.rs` (NEW)
**Existing gap**: No reproducible-build verification, no binary hash audit, no cargo-audit gate at build time.
**Proposed module**:
- `verify_build_artifact()` — checks cargo.lock committed, binary SHA-256 matches, LTO/strip/panic=abort flags active
- `check_supply_chain()` — runs `cargo audit` and `cargo deny` programmatically
- `detect_unsafe()` — scans for `unsafe` blocks in project source (already done manually; automate)
- **Integration**: Called from `main.rs` startup; fails fast if build integrity violated.

### Layer 2: Secrets Management → `backend/secrets_vault.rs` (NEW) + enhance `key_manager.rs`
**Existing gap**: `.env` still contains plaintext keys (P0 fixed but not programmatically enforced). `key_manager.rs` only handles wallet keys, not API keys/RPC URLs.
**Proposed module**:
- `audit_secrets()` — scans `.env`, env vars, and process memory for plaintext key patterns
- `verify_vault_injection()` — confirms all secrets come from OS keyring / Vault, not files
- `check_key_rotation()` — flags keys older than rotation policy
- `detect_leaked_keys()` — cross-references known-leaked key prefixes
- **Integration**: Hooks into `key_manager.unlock_key()` and `module_62_alert_system` for real-time secret leak alerts.

### Layer 3: Transaction Safety → `backend/tx_safety.rs` (NEW) + enhance `module_57_pool_dispatcher.rs`
**Existing gap**: `module_57_pool_dispatcher.rs:308` has `simulate_swap` but uses `amountOut` (mock ABI), not `balanceOf` before/after. No pre-tx profit validation.
**Proposed module**:
- `validate_profit_above_gas()` — profit must exceed gas cost + 20% buffer (P1 fix, now centralized)
- `verify_balance_snapshot()` — `balanceOf` before/after check (P0 fix, now mandatory gate)
- `check_nonce_consistency()` — nonce from `nonce_manager` matches chain state
- `verify_block_freshness()` — block number from `rpc_consensus` is within staleness threshold
- **Integration**: Called in `module_57_pool_dispatcher::calculate_optimal_route()` and before any `sendRawTransaction`.

### Layer 4: MEV Defense → enhance `private_mempool.rs` + `backend/mev_defense.rs` (NEW)
**Existing gap**: `private_mempool.rs` has Flashbots bundle submission, but no sandwich detection, no mempool backlog simulation, no competitor monitoring integration.
**Proposed module**:
- `verify_private_relay()` — confirms `VITE_PRIVATE_RPC_URL` / `FLASHBOTS_RELAY_URL` is set before live trading
- `detect_sandwich_risk()` — simulates mempool backlog to check if tx would be sandwiched
- `check_competitor_pressure()` — uses `module_58_shadow_replay` competitor data to assess timing risk
- `verify_bundle_inclusion()` — post-submission check via `eth_getBundleStatus`
- **Integration**: Wraps `PrivateMempool::send_bundle()`; blocks submission if any check fails.

### Layer 5: Infrastructure Hardening → `backend/infra_audit.rs` (NEW)
**Existing gap**: Docker hardening applied manually. No runtime check for seccomp, capabilities, filesystem permissions, or SSH exposure.
**Proposed module**:
- `audit_container_runtime()` — verifies `no-new-privileges`, `cap_drop: ALL`, `read_only`, non-root user
- `check_network_exposure()` — scans open ports, verifies firewall rules, checks for SSH on public interfaces
- `verify_seccomp()` — confirms seccomp-bpf profile is loaded
- `check_filesystem_permissions()` — `.env` is 600, certs are 600, no world-readable secrets
- **Integration**: Called from `main.rs` startup and from `module_62_alert_system` on health-check failures.

### Layer 6: Monitoring & Observability → enhance `module_62_alert_system.rs` + `backend/acid_monitor.rs` (NEW)
**Existing gap**: `module_62_alert_system` raises alerts but has no external webhook delivery, no Prometheus alert rules, no balance-drift monitoring.
**Proposed module**:
- `check_alert_delivery()` — verifies Slack/PagerDuty webhooks are reachable
- `monitor_balance_drift()` — periodic `balanceOf` check vs expected balance; alerts on discrepancy
- `check_rpc_health()` — uses `metrics::check_rpc_latency()`; alerts if latency > threshold
- `verify_prometheus_rules()` — ensures alert rules are loaded in Prometheus
- **Integration**: Runs as a tokio::interval task; feeds into `module_62_alert_system::raise_alert()`.

### Layer 7: Disaster Recovery → enhance `emergency_sweep.rs` + `backend/dr_audit.rs` (NEW)
**Existing gap**: `emergency_sweep.rs` has sweep config and backup records, but no offsite backup verification, no RTO/RPO metrics, no recovery drill automation.
**Proposed module**:
- `verify_backup_integrity()` — checks backup checksum, confirms offsite copy exists
- `check_rto_rpo_compliance()` — last backup age < RPO, last recovery drill age < RTO test interval
- `verify_emergency_sweep()` — confirms sweep tx is signed, timelock is valid, treasury address is correct
- `check_nonce_recovery()` — nonce DB can be restored from backup without gaps
- **Integration**: Daily audit task; feeds alerts to `module_62_alert_system`.

### Layer 8: Financial Controls → `backend/guardrails.rs` (ENHANCE) + `backend/financial_audit.rs` (NEW)
**Existing gap**: `guardrails.rs` has daily loss limits but no per-trade gas ceiling, no per-DEX exposure caps, no hourly limits, no atomic unwind.
**Proposed module**:
- `audit_financial_limits()` — verifies limits are initialized from capital size (not defaults)
- `check_per_trade_limits()` — gas cost ceiling, slippage tolerance, position size
- `verify_exposure_caps()` — per-DEX, per-chain, per-pool position limits
- `check_unwind_capability()` — confirms atomic unwind contract is deployed and funded
- **Integration**: Called from `guardrails::authorize_trade()` and after every trade execution.

### Layer 9: Chaos Engineering → `backend/chaos_lab.rs` (NEW)
**Existing gap**: No malformed RPC response handling, no reorg simulation, no gas price manipulation guard, no credential exposure scan.
**Proposed module**:
- `simulate_rpc_failures()` — injects timeouts, malformed responses, stale blocks
- `simulate_reorgs()` — uses foundry/anvil fork to test 1-10 block reorganizations
- `simulate_gas_spike()` — tests behavior when baseFee increases 10x
- `scan_credential_exposure()` — truffleHog-style scan of env, logs, and process memory
- **Integration**: Run as nightly CI job; results feed into `module_62_alert_system`.

### Layer 10: Preflight CI/CD → enhance `.github/workflows/security-gates.yml` + `backend/preflight.rs` (NEW)
**Existing gap**: CI checks compilation and linting but doesn't run foundry fork tests, balance assertions, or capital ramp validation.
**Proposed module**:
- `run_staging_simulation()` — foundry fork test with real RPC, asserts profit > gas
- `verify_balance_assertion()` — post-trade balance check in staging
- `validate_capital_ramp()` — confirms daily capital limits match staged ramp schedule
- `check_kill_switch()` — verifies kill switch triggers in <1s
- **Integration**: GitHub Actions workflow calls `preflight::run_all_checks()` before merge to main.

---

## 3. Central ACID Orchestrator

### New Module: `backend/acid_core.rs`

The central orchestrator that ties all 10 layers together:

```rust
pub struct AcidEngine {
    build_guard: BuildGuard,
    secrets_vault: SecretsVault,
    tx_safety: TxSafety,
    mev_defense: MevDefense,
    infra_audit: InfraAudit,
    monitor: AcidMonitor,
    dr_audit: DrAudit,
    financial_audit: FinancialAudit,
    chaos_lab: ChaosLab,
    preflight: Preflight,
}

impl AcidEngine {
    /// Run all 10 layers — used for startup self-check and operator debug commands
    pub async fn run_full_audit(&self) -> AcidReport { ... }

    /// Run only the layers relevant to a pre-trade gate
    pub async fn pre_trade_check(&self, trade: &Trade) -> AcidResult { ... }

    /// Run only layers relevant to deployment
    pub async fn pre_deploy_check(&self) -> AcidResult { ... }

    /// Diagnose an alert: which ACID layers are violated?
    pub async fn diagnose_alert(&self, alert: &Alert) -> Vec<AcidLayer> { ... }
}
```

**Key Design Decisions**:
1. **Pluggable checks**: Each layer is a trait `AcidCheck` with `check() -> Result<(), AcidViolation>`
2. **Fail-fast vs. warn**: Layers 1-5 (security) are fail-fast; Layers 6-10 (operational) are warn-and-continue
3. **Result aggregation**: `AcidReport` contains per-layer pass/fail + evidence + remediation steps
4. **gRPC exposure**: New `AcidDebug` service in `c2_service.proto` for operator queries

---

## 4. Integration Points with Existing 60 Modules

| Existing Module | ACID Integration | Trigger Point |
|----------------|-----------------|---------------|
| `module_57_pool_dispatcher` | L3: Tx Safety | `calculate_optimal_route()` — pre-trade gate |
| `module_58_shadow_replay` | L4: MEV Defense | `detect_anomalies()` — competitor pressure check |
| `module_59_state_synchronizer` | L3: Consensus | `update_state_root()` — dual-RPC verification |
| `module_62_alert_system` | L6: Monitoring | `raise_alert()` — ACID diagnosis on every alert |
| `guardrails` | L8: Financial | `authorize_trade()` — financial audit gate |
| `key_manager` | L2: Secrets | `unlock_key()` — vault verification |
| `emergency_sweep` | L7: DR | `create_emergency_sweep()` — sweep validation |
| `CentralC2Server` | L10: Preflight | `new()` — startup full ACID audit |
| `fleet_controller` | L5: Infra | `rebalance_fleet()` — infra audit before scale |
| `metrics` | L6: Monitoring | `check_rpc_latency()` — health data for ACID |

---

## 5. Operator Debugging Workflow

### 5.1 Startup Self-Check
```
main() → CentralC2Server::new() → acid_engine.run_full_audit()
  → If L1-L5 fail: HALT + alert
  → If L6-L10 fail: WARN + continue with degraded mode
```

### 5.2 Pre-Trade Gate
```
module_57::calculate_optimal_route()
  → acid_engine.pre_trade_check(trade)
    → L3: balance_snapshot, profit_above_gas, nonce_consistency
    → L4: private_relay_configured, sandwich_risk
    → L8: position_size, daily_loss_remaining
  → If any fail: BLOCK trade + log ACID violation
```

### 5.3 Alert Diagnosis
```
module_62::raise_alert(Critical, "RPC latency spike")
  → acid_engine.diagnose_alert(alert)
    → L5: check_network_exposure() → PASS
    → L6: check_rpc_health() → FAIL (primary RPC 3s > 2s threshold)
    → L3: check_block_freshness() → FAIL (stale block detected)
  → Operator sees: ACID L3+L6 violated, root cause = RPC lag + stale block
```

### 5.4 Pre-Deploy Gate
```
CI: security-gates.yml
  → acid_engine.pre_deploy_check()
    → L1: build_guard.verify()
    → L2: secrets_vault.audit()
    → L9: chaos_lab.simulate_reorgs()
    → L10: preflight.run_staging_simulation()
  → All must pass before merge to main
```

### 5.5 Continuous Audit Daemon
```
acid_engine.run_background_audit()
  → Every 5 minutes: L3, L4, L6, L8
  → Every hour: L7 (backup verification)
  → Every 24 hours: L1, L2, L5, L9, L10
  → Violations → module_62::raise_alert()
```

---

## 6. ACID Report Format (Standardized)

Every ACID check produces a structured report:

```json
{
  "timestamp": "2026-06-24T03:42:00Z",
  "mode": "startup_full | pre_trade | pre_deploy | background",
  "layers": [
    {
      "layer": 3,
      "name": "Blockchain Execution & Transaction Safety",
      "status": "PASS | WARN | FAIL | BLOCKED",
      "checks": [
        {
          "name": "balance_based_simulation",
          "status": "PASS",
          "evidence": "balanceOf before=1.5 ETH, after=1.499 ETH, delta=-0.001 ETH"
        },
        {
          "name": "profit_above_gas",
          "status": "FAIL",
          "evidence": "profit=0.0005 ETH, gas_cost=0.001 ETH, buffer_required=0.0012 ETH",
          "remediation": "Increase position size or find higher-yield route"
        }
      ]
    }
  ],
  "overall": "FAIL",
  "blocking_layers": [3],
  "recommended_action": "HALT — do not execute trade until L3 profit check passes"
}
```

---

## 7. Implementation Phases

### Phase 1: Core Engine (Week 1)
- Create `acid_core.rs` with `AcidEngine` struct and `AcidCheck` trait
- Implement `run_full_audit()` aggregation
- Add gRPC `AcidDebug` service to `c2_service.proto`

### Phase 2: Security Layers (Week 1-2) — Fail-Fast
- `build_guard.rs` (L1)
- `secrets_vault.rs` (L2)
- `tx_safety.rs` (L3)
- `mev_defense.rs` (L4)
- `infra_audit.rs` (L5)

### Phase 3: Operational Layers (Week 2-3) — Warn-Continue
- `acid_monitor.rs` (L6)
- `dr_audit.rs` (L7)
- `financial_audit.rs` (L8)

### Phase 4: CI/CD Integration (Week 3)
- `chaos_lab.rs` (L9)
- `preflight.rs` (L10)
- Update `security-gates.yml` to call ACID preflight

### Phase 5: Existing Module Hooks (Week 3-4)
- Add `acid_engine.pre_trade_check()` to `module_57`
- Add `acid_engine.diagnose_alert()` to `module_62`
- Add startup audit to `main.rs`

---

## 8. Files to Create/Modify

### New Files
| File | ACID Layers | Purpose |
|------|------------|---------|
| `backend/acid_core.rs` | All | Central orchestrator + trait definitions |
| `backend/build_guard.rs` | L1 | Build integrity verification |
| `backend/secrets_vault.rs` | L2 | Secrets audit and leak detection |
| `backend/tx_safety.rs` | L3 | Transaction safety gates |
| `backend/mev_defense.rs` | L4 | MEV/sandwich/bundle checks |
| `backend/infra_audit.rs` | L5 | Container/network hardening audit |
| `backend/acid_monitor.rs` | L6 | Monitoring and alert delivery |
| `backend/dr_audit.rs` | L7 | Disaster recovery verification |
| `backend/financial_audit.rs` | L8 | Financial controls audit |
| `backend/chaos_lab.rs` | L9 | Chaos engineering tests |
| `backend/preflight.rs` | L10 | CI/CD preflight gates |
| `backend/acid_report.rs` | All | Structured report types |
| `backend/acid_debug.rs` | All | gRPC debug service |

### Modified Files
| File | Changes |
|------|---------|
| `main.rs` | Add `mod acid_core`, startup audit, integrate into CentralC2Server |
| `module_57_pool_dispatcher.rs` | Pre-trade ACID gate in `calculate_optimal_route` |
| `module_62_alert_system.rs` | ACID diagnosis on `raise_alert` |
| `guardrails.rs` | Hook `financial_audit` into `authorize_trade` |
| `key_manager.rs` | Hook `secrets_vault` into `unlock_key` |
| `emergency_sweep.rs` | Hook `dr_audit` into sweep creation |
| `c2_service.proto` | Add `AcidDebug` service + `AcidReport` message |
| `.github/workflows/security-gates.yml` | Add ACID preflight job |

---

## 9. Why This Architecture

1. **Single Responsibility**: Each ACID layer has its own module — no 1000-line god module
2. **Pluggable**: New checks can be added per layer without touching orchestrator
3. **Fail-Fast for Security**: L1-L5 halt execution on failure (non-negotiable safety)
4. **Warn-Continue for Operations**: L6-L10 log and alert but don't block (availability)
5. **Operator-Friendly**: Standardized `AcidReport` format means every debug session follows the same workflow
6. **CI-Native**: ACID checks run automatically in GitHub Actions — no manual audit needed
7. **Leverages Existing Code**: Reuses `RpcConsensus`, `BalanceSimulator`, `PrivateMempool`, `NonceManager`, `Guardrails` — no reinvention

---

## 10. Result: AllBright's ACID Debugging System

```
┌─────────────────────────────────────────────────────┐
│                  ACID CORE ORCHESTRATOR              │
│  ┌───────┐ ┌───────┐ ┌───────┐ ┌───────┐ ┌───────┐ │
│  │  L1   │ │  L2   │ │  L3   │ │  L4   │ │  L5   │ │
│  │Build  │ │Secrets│ │  Tx   │ │ MEV   │ │ Infra │ │
│  │Guard  │ │Vault  │ │Safety │ │Defense│ │ Audit │ │
│  └───┬───┘ └───┬───┘ └───┬───┘ └───┬───┘ └───┬───┘ │
│  ┌───┴───┐ ┌───┴───┐ ┌───┴───┐ ┌───┴───┐ ┌───┴───┐ │
│  │  L6   │ │  L7   │ │  L8   │ │  L9   │ │  L10  │ │
│  │Monitor│ │  DR   │ │Finance│ │ Chaos │ │Preflight│ │
│  └───┬───┘ └───┬───┘ └───┬───┘ └───┬───┘ └───┬───┘ │
│      └─────────┴─────────┴─────────┴─────────┘     │
│                    │                                 │
│  ┌─────────────────┼─────────────────┐              │
│  │                 ▼                 │              │
│  │         AcIdReport (JSON)         │              │
│  │     per-layer pass/fail +         │              │
│  │     evidence + remediation        │              │
│  └─────────────────┬─────────────────┘              │
│                    │                                 │
│  ┌─────────────────┼─────────────────┐              │
│  │                 ▼                 │              │
│  │  gRPC: AcidDebug::RunAudit       │              │
│  │  gRPC: AcidDebug::DiagnoseAlert  │              │
│  │  gRPC: AcidDebug::PreTradeCheck  │              │
│  └──────────────────────────────────┘              │
└─────────────────────────────────────────────────────┘
         │              │              │
         ▼              ▼              ▼
    ┌─────────┐  ┌─────────┐  ┌─────────┐
    │ module_ │  │ module_ │  │  CI/CD  │
    │   57    │  │   62    │  │  Gates  │
    │(pool)   │  │(alerts) │  │         │
    └─────────┘  └─────────┘  └─────────┘
```

This gives AllBright a **self-documenting, self-debugging safety system** where every critical path is guarded by the full ACID checklist, and every alert can be traced to specific checklist violations.
