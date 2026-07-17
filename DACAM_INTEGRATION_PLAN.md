# DACAM Integration Plan — AllBright V119
**Module:** Deep-Analytical Copilot Audit Module (DACAM)
**Source Specification:** `copioaudior`
**Target:** AllBright Arbitrage Flash Loan Engine
**Version:** V119.1
**Date:** 2026-07-08

---

## 1. Executive Summary

The `copioaudior` file is an architectural specification (ToR) for **DACAM** — an independent, read-only audit sidebar that monitors the Autonomous Copilot Engine in real-time, validates its math and data integrity in a clean-room environment, and provides the Commander with closed-loop override controls to force self-correction without downtime.

**Integration approach:** Implement DACAM as a new backend module (`m120_dacam_copilot_audit.rs`) and a frontend sidebar component in `apps/dashboard`, wired through existing gRPC/HTTP/WebSocket channels.

---

## 2. DACAM Functional Requirements → Allbright Mapping

| DACAM Requirement | Existing Allbright Component | Gap / Action |
|-------------------|------------------------------|--------------|
| Telemetry tap from `/audit/operational/` | `broadcast::Sender<FleetStatus>` in `CentralC2Server` | Wrap in dedicated audit stream |
| Shadow math calculation (SDI, Λ, ε_f) | `shield_guardrails.rs`, `metrics.rs` | Extract formulas into `m120` clean-room module |
| Standalone sidebar UI | `apps/dashboard/src/App.tsx` (11-tab) | Add sidebar panel + WebSocket/gRPC stream |
| Closed-loop override injection | `HotSwapRegistry` + `server.js` endpoints | Add DACAM override endpoint + directory watcher |
| Zero-trust data validity engine | `m099_zk_proof.rs`, `ai_agents.rs` | Add cross-examination layer in `m120` |
| Directory ledger schema | PostgreSQL via `sqlx` | Create `dacam_audit_log` table |
| Fail-safe protocols | `constitution_guard.rs` | Add DACAM-specific enforcement gates |
| Separation from Sovereign Auditor | `relationship_matrix.rs`, CGM | Enforce path isolation (`/audit/operational/` vs `/audit/sovereign/`) |

---

## 3. Backend Implementation Plan

### 3.1 New Backend Module: `backend/m120_dacam_copilot_audit.rs`

Create following the standalone module pattern (like `m054_auto_optimizer.rs`, `m057_pool_dispatcher.rs`):

**Responsibilities:**
1. **Telemetry interception** — Tap the `FleetStatus` broadcast channel
2. **Zero-trust data validation** — Multi-source oracle cross-examination with 0.5% drift tolerance, ±2s timestamp drift rejection
3. **Clean-room shadow math** — Recompute SDI, Λ, ε_f, α independently; bit-exact comparison (Δ = 0)
4. **Directory ledger persistence** — Write cryptographic audit records to PostgreSQL
5. **Override injection** — Write directives to `/allbright/directory/simulation/active_override` path (in-memory registry + future filesystem watcher)
6. **Fail-safe enforcement** — Auto-apply safety overrides when thresholds breached and Commander inactive

**Key structs and formulas:**

```rust
// Audit record matching the Directory Ledger Schema (§5)
pub struct DacamAuditRecord {
    pub audit_id: [u8; 32],
    pub block_height: u64,
    pub target_copilot_id: String,
    pub audit_classification: AuditClassification,
    pub data_integrity: DataIntegrity,
    pub calculation_integrity: CalculationIntegrity,
    pub analytical_benchmarks: AnalyticalBenchmarks,
    pub loop_reentry_state: LoopReentryState,
    pub governance_enforcement: GovernanceEnforcement,
}

// Thresholds from spec §4
pub struct DacamThresholds {
    pub sdi_max_pct: f64,           // 3.0
    pub parasitic_leakage_cutoff_pct: f64, // 6.0
    pub parasitic_leakage_target_pct: f64, // 4.0
    pub fleet_elasticity_min: f64,  // 0.15
    pub alpha_min_pct: f64,         // 15.0
    pub oracle_drift_max_pct: f64,  // 0.5
    pub timestamp_drift_max_ms: i64, // 2000
    pub safe_baseline_capacity_pct: f64, // 25.0
}

// Formula implementations
pub fn compute_sdi(simulated_roi: f64, actual_roi: f64) -> f64;
pub fn compute_parasitic_leakage(gas: f64, slippage: f64, mev: f64, total_value: f64) -> f64;
pub fn compute_fleet_elasticity(yield_delta_pct: f64, node_delta_pct: f64) -> f64;
pub fn compute_alpha(realized_yield: f64, passive_baseline: f64) -> f64;
```

**Override types:**
```rust
pub enum OverrideType {
    None,
    RoutingShift,      // Inject L2 Arbitrum/Base constraint
    CapacityThrottle,  // Downscale to 50% or safe baseline
}
```

### 3.2 Registration in `backend/main.rs`

1. Add module declaration: `mod m120_dacam_copilot_audit;`
2. Add `use crate::m120_dacam_copilot_audit::DacamAuditModule;`
3. Add field to `CentralC2Server`:
   ```rust
   pub dacam: DacamAuditModule,
   pub dacam_tx: broadcast::Sender<DacamAuditAlert>,
   pub active_override: Arc<Mutex<Option<OverrideType>>>,
   ```
4. Instantiate in `CentralC2Server::new()`
5. Register in `HotSwapRegistry::register_core_modules()`:
   ```rust
   ("M120", "DACAM Copilot Audit", "m120_dacam_copilot_audit.rs"),
   ```
6. Wire DACAM as a listener on the fleet status broadcast channel at startup
7. Add DACAM startup check: scan `/allbright/directory/simulation/active_override` for pending directives (future: filesystem watcher)

### 3.3 New gRPC Endpoint (in `c2_service.proto`)

```protobuf
rpc StreamDacamAlerts(Empty) returns (stream DacamAlert) {}
rpc InjectDacamOverride(DacamOverrideRequest) returns (DacamOverrideResponse) {}
rpc GetDacamLedger(DacamLedgerQuery) returns (stream DacamAuditRecord) {}
```

Regenerate with `protoc` and implement in `main.rs`.

### 3.4 New HTTP Endpoints (in `server.js` + Axum router)

```javascript
GET  /api/dacam/status              // Current DACAM health, system status
GET  /api/dacam/alerts              // Active/pending alerts
GET  /api/dacam/ledger              // Audit log history (paginated)
POST /api/dacam/override/inject     // Commander injects override
GET  /api/dacam/benchmarks          // Latest SDI, Λ, ε_f, α
```

### 3.5 Database Schema (PostgreSQL via sqlx)

```sql
CREATE TABLE dacam_audit_records (
    audit_id BYTEA PRIMARY KEY,
    block_height BIGINT NOT NULL,
    target_copilot_id TEXT NOT NULL,
    audit_classification TEXT NOT NULL DEFAULT 'MICRO_OPERATIONAL_COPILOT_ONLY',
    oracle_consensus_status TEXT NOT NULL,
    max_price_feed_drift_pct DOUBLE PRECISION,
    timestamp_sync_drift_ms BIGINT,
    shadow_math_execution TEXT NOT NULL,
    simulation_drift_delta DOUBLE PRECISION DEFAULT 0.0,
    parasitic_leakage_delta DOUBLE PRECISION DEFAULT 0.0,
    fleet_elasticity_delta DOUBLE PRECISION DEFAULT 0.0,
    simulation_drift_index DOUBLE PRECISION NOT NULL,
    parasitic_value_leakage_index DOUBLE PRECISION NOT NULL,
    fleet_capital_elasticity DOUBLE PRECISION NOT NULL,
    alpha_vs_passive_baseline DOUBLE PRECISION NOT NULL,
    commander_directive_pending BOOLEAN DEFAULT FALSE,
    active_override_injected TEXT DEFAULT 'NONE',
    copilot_acknowledgement_status TEXT DEFAULT 'WAITING',
    boundary_violations INTEGER DEFAULT 0,
    system_health_status TEXT NOT NULL DEFAULT 'GREEN',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    INDEX idx_dacam_block (block_height),
    INDEX idx_dacam_copilot (target_copilot_id),
    INDEX idx_dacam_health (system_health_status)
);
```

### 3.6 Integration with Copilot Loop

The Copilot Engine (existing `ai/` subsystem or `trading_engine.rs`) must:

1. At the **start of every block header execution cycle**, check `CentralC2Server.active_override`
2. If override is active:
   - Parse the directive payload
   - Update operational limits (node count, routing constraints)
   - Wipe active memory cache
   - Re-route optimization path with new constraints
   - Continue execution (no downtime)
3. After processing, write acknowledgement status back to the override record

**Implementation location:** Add override check in `CentralC2Server::execute_block_cycle()` or the `trading_engine.rs` execution loop.

### 3.7 Fail-Safe Enforcement Logic

```rust
pub fn check_fail_safes(&mut self, metrics: &CopilotMetrics) -> Vec<DacamOverride> {
    let mut overrides = Vec::new();

    if metrics.fleet_elasticity < self.thresholds.fleet_elasticity_min {
        overrides.push(DacamOverride {
            override_type: OverrideType::CapacityThrottle,
            payload: json!({"action": "lock_node_count", "new_limit": "read_only"}),
            auto: true,
            reason: format!("ε_f {} < {}", metrics.fleet_elasticity, self.thresholds.fleet_elasticity_min),
        });
    }

    if metrics.parasitic_leakage > self.thresholds.parasitic_leakage_cutoff_pct {
        overrides.push(DacamOverride {
            override_type: OverrideType::RoutingShift,
            payload: json!({"action": "force_fallback_rpc", "cluster": "isolated_fallback"}),
            auto: true,
            reason: format!("Λ {} > {}", metrics.parasitic_leakage, self.thresholds.parasitic_leakage_cutoff_pct),
        });
    }

    if metrics.shadow_math_delta != 0.0 {
        overrides.push(DacamOverride {
            override_type: OverrideType::CapacityThrottle,
            payload: json!({"action": "safe_baseline", "capacity_pct": 25.0}),
            auto: true,
            reason: "Math boundary violation (Δ ≠ 0)".to_string(),
        });
    }

    overrides
}
```

---

## 4. Frontend Implementation Plan

### 4.1 New Component: `apps/dashboard/src/components/DacamPanel.tsx`

Standalone sidebar panel following the existing component patterns (see `CopilotPanel.tsx`, `OperationsCenter.tsx`):

**Layout:** Fixed sidebar (300-400px width) docked to the right of the main console
**Data sources:**
- WebSocket stream from `/ws/dacam` (or gRPC-web if WebSocket not ready — fallback to HTTP polling on `/api/dacam/status`)
- Override injection via `POST /api/dacam/override/inject`

**Rendering sections:**

| Section | Content |
|---------|---------|
| Data Integrity Bar | Oracle consensus status, drift %, sync latency |
| Math Integrity Bar | Shadow math result: MATCH / MISMATCH, delta |
| Benchmark Forensics | SDI, Λ, ε_f, α with color-coded status (✓ ✓Nominal / ❌ FAULT) |
| Closed-Loop Control Panel | Diagnosis text + Action A/B override buttons |
| System Health | GREEN / AMBER / RED indicator |

**Zero-trust visual indicators:**
- Pulsing red border when data is COMPROMISED
- "GUILTY UNTIL PROVEN INNOCENT" badge for unverified packets
- Ephemeral trust countdown timer (resets per block)

**Override buttons:**
```tsx
// Action A: RE-ROUTE MATRIX
<button onClick={() => injectOverride('ROUTING_SHIFT')}>
  ROUTE TO L2 ARBITRUM/BASE
</button>

// Action B: THROTTLE FLEET
<button onClick={() => injectOverride('CAPACITY_THROTTLE', 50)}>
  THROTTLE TO 50%
</button>

// Auto fail-safe: SAFE BASELINE (triggered by system)
<button onClick={() => injectOverride('CAPACITY_THROTTLE', 25)} disabled={!autoTriggered}>
  EMERGENCY 25% BASELINE
</button>
```

### 4.2 Integration into `apps/dashboard/src/App.tsx`

Add DACAM as a collapsible sidebar alongside the existing main tab interface:

```tsx
// New tab or persistent sidebar (recommended: sidebar for real-time monitoring)
<div className="flex">
  <main className="flex-1">
    {/* Existing tab content */}
  </main>
  <aside className="w-96 border-l">
    <DacamPanel />
  </aside>
</div>
```

### 4.3 WebSocket / gRPC-Web Client

Create `apps/dashboard/src/hooks/useDacamStream.ts`:

```tsx
export function useDacamStream() {
  const [alerts, setAlerts] = useState<DacamAlert[]>([]);
  const [status, setStatus] = useState<DacamStatus | null>(null);
  const [benchmarks, setBenchmarks] = useState<Benchmarks | null>(null);
  const wsRef = useRef<WebSocket | null>(null);

  useEffect(() => {
    const ws = new WebSocket(`${WS_URL}/dacam`);
    ws.onmessage = (event) => {
      const data = JSON.parse(event.data);
      switch (data.type) {
        case 'status': setStatus(data.payload); break;
        case 'alert': setAlerts(prev => [...prev, data.payload]); break;
        case 'benchmark': setBenchmarks(data.payload); break;
      }
    };
    wsRef.current = ws;
    return () => ws.close();
  }, []);

  const injectOverride = async (type: OverrideType, payload: any) => {
    await fetch('/api/dacam/override/inject', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ override_type: type, payload }),
    });
  };

  return { alerts, status, benchmarks, injectOverride };
}
```

---

## 5. Governance & Separation Concerns

### 5.1 Path Isolation

| Path | Owner | DACAM Access |
|------|-------|-------------|
| `/audit/operational/` | DACAM (this module) | Read + Write overrides |
| `/audit/sovereign/` | Sovereign Auditor (CGM core) | **No access** |
| `/allbright/directory/simulation/active_override` | DACAM override channel | Write only |
| `/allbright/directory/constitution/` | ConstitutionGuard | **No access** |

### 5.2 CGM Law Compliance

DACAM must not violate CGM Laws. Review `relationship_matrix.rs` for constraints:

- **Law V (Quality):** DACAM enhances Quality (5%) by providing audit integrity
- **Law VI (Security):** Zero-trust approach strengthens Security
- DACAM does **not** modify Profit, Growth, Velocity, or Efficiency directly — only through approved override channels governed by the Copilot Engine's own enforcement loop

### 5.3 Separation from Sovereign Auditor

The DACAM sidebar is **micro-operational only**. It must NOT:
- Access platform-wide asset protection policies
- Modify constitutional state
- Touch identity/authentication protocols
- Issue enterprise-risk directives

These remain the Sovereign Auditor's domain via `/audit/sovereign/`.

---

## 6. Module Registry Update (`MODULE_REGISTRY.toml`)

```toml
[[module]]
id = "M120"
name = "DACAM Copilot Audit Module"
file = "backend/m120_dacam_copilot_audit.rs"
status = "PLANNED"
kpis = ["KPI-15", "KPI-16", "KPI-17", "KPI-18"]
notes = "Deep-analytical Copilot Audit Module with standalone sidebar UI and closed-loop override pipeline. Audits SDI, parasitic leakage (Λ), fleet elasticity (ε_f), and alpha generation. Zero-trust data validity engine with clean-room shadow math verification."

[[module]]
id = "M121"
name = "DACAM Frontend Panel"
file = "apps/dashboard/src/components/DacamPanel.tsx"
status = "PLANNED"
kpis = ["KPI-15"]
notes = "DACAM sidebar UI component. WebSocket stream for real-time telemetry. Closed-loop Commander control panel with override injection."
```

---

## 7. Implementation Sequence (Recommended Order)

| Phase | Task | Est. | Priority |
|-------|------|------|----------|
| **P1** | Create `m120_dacam_copilot_audit.rs` — core audit engine with shadow math formulas | 4h | P0 |
| **P2** | Register M120 in `main.rs` + `HotSwapRegistry` + wire to fleet broadcast | 1h | P0 |
| **P3** | Add PostgreSQL table `dacam_audit_records` and sqlx model | 1h | P1 |
| **P4** | Add gRPC endpoints for DACAM streams | 1h | P1 |
| **P5** | Add HTTP endpoints (`/api/dacam/*`) in `server.js` + Axum | 1h | P1 |
| **P6** | Implement override injection channel + active_override registry | 2h | P0 |
| **P7** | Wire Copilot Engine to check overrides at block cycle start | 2h | P0 |
| **P8** | Create `DacamPanel.tsx` frontend component | 3h | P1 |
| **P9** | Add WebSocket/gRPC-web client hook `useDacamStream.ts` | 1h | P1 |
| **P10** | Integrate DACAM sidebar into `App.tsx` | 1h | P1 |
| **P11** | Implement fail-safe enforcement logic (auto-override) | 2h | P2 |
| **P12** | Update `MODULE_REGISTRY.toml` | 15min | P1 |
| **P13** | Testing — unit tests for shadow math formulas | 2h | P1 |
| **P14** | Integration testing — override injection → Copilot ack cycle | 2h | P1 |
| **P15** | Docker compose updates (if needed) | 30min | P2 |

**Total estimated effort:** ~22 hours

---

## 8. Key Technical Decisions

| Decision | Recommendation | Rationale |
|----------|---------------|-----------|
| WebSocket vs gRPC-web for frontend | **WebSocket** (existing pattern) | Simpler, already used elsewhere; gRPC-web fallback if needed |
| Override persistence | **In-memory + PostgreSQL** | Fast atomic reads from Copilot loop; durable store for ledger |
| Directory path abstraction | **Wrap `/allbright/directory/simulation/active_override` as an in-memory registry** now, add filesystem watcher later | No file I/O needed for MVP; maintains future compatibility |
| Shadow math location | **Dedicated clean-room in M120** | Must be completely isolated from Copilot's own math per spec |
| Fail-safe override timing | **Automatic with Commander notification** | Per spec: auto-activate but surface to sidebar immediately |
| Data freshness | **Block-height-anchored** | Every trust assertion expires at next block per "100% Doubt" rules |

---

## 9. Open Questions to Resolve During Implementation

1. **Where exactly does the Copilot Engine's block execution cycle live?** Need to locate the exact `execute_block` / `process_block` function to wire the override check. Likely in `trading_engine.rs` or a dedicated execution loop.
2. **Does the system already have a WebSocket server?** The spec mentions port 50052 but exploration showed partial implementation. Need to confirm or add one.
3. **What is the exact Copilot ID format?** The ledger schema references `target_copilot_id` — need to confirm the naming convention for AI agents (AI101-AI106).
4. **Should override injection be transactional?** If the Copilot is mid-execution, how to ensure atomic override + cache wipe + re-route without race conditions?
5. **Does the Sovereign Auditor already have a `/audit/sovereign/` path implementation?** Need to verify no namespace collision.

---

## 10. Risk Assessment

| Risk | Severity | Mitigation |
|------|----------|------------|
| Copilot Engine has no clean override hook | High | May need to refactor execution loop; allow 4h buffer |
| WebSocket server missing | Medium | HTTP polling fallback available |
| Shadow math diverges from Copilot's own formulas | Medium | Need cross-reference with existing metrics in `shield_guardrails.rs` |
| Override injection creates race conditions | High | Use `Arc<Mutex<Option<OverrideType>>>` with atomic swap pattern |
| DACAM accidentally touches sovereign paths | High | Enforce path isolation in code review + CI lint rule |
| Performance overhead from shadow calculations | Low | DACAM runs asynchronously; not on critical path |

---

## 11. Files to Create / Modify

### New Files
| File | Purpose |
|------|---------|
| `backend/m120_dacam_copilot_audit.rs` | Core audit module |
| `backend/m120_dacam_audit_models.rs` | (Optional) Separate models if file exceeds 500 lines |
| `apps/dashboard/src/components/DacamPanel.tsx` | Frontend sidebar |
| `apps/dashboard/src/hooks/useDacamStream.ts` | WebSocket hook |
| `apps/dashboard/src/types/dacam.ts` | TypeScript interfaces |

### Files to Modify
| File | Changes |
|------|---------|
| `backend/main.rs` | Module decl, field, registration, wiring |
| `backend/c2_service.proto` | New gRPC endpoints |
| `backend/server.js` | New HTTP endpoints |
| `backend/Cargo.toml` | (Usually no new deps needed) |
| `MODULE_REGISTRY.toml` | M120 + M121 entries |
| `apps/dashboard/src/App.tsx` | Add DACAM sidebar |
| `docker-compose.yml` | (Probably no changes) |

---

## 12. Consistency Check

- **Tech stack:** Rust backend + React/TypeScript frontend — matches existing patterns ✓
- **Module pattern:** Standalone .rs file + main.rs registration + HotSwapRegistry — matches ✓
- **Frontend pattern:** React component in `apps/dashboard/src/components/` — matches ✓
- **Communication:** HTTP REST + gRPC + (WebSocket optional) — matches existing ✓
- **Data persistence:** PostgreSQL via sqlx — matches existing ✓
- **Governance:** Enforces CGM path separation, no constitutional violations ✓
- **DACAM spec compliance:** All 5 sections of the spec are addressed in the plan above ✓
