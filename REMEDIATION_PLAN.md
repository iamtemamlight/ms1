# ALLBRIGHT V119 — FULL REMEDIATION PLAN

**Prepared by**: Lead Architect  
**Date**: 2026-06-23  
**Status**: AWAITING APPROVAL  
**Target**: Production-ready, accurately-documented, security-clean system

---

## EXECUTIVE SUMMARY

The prior audit reports contain material inaccuracies that overstate system completeness. This plan corrects the record and sequences the work needed to reach genuine production readiness. It is organized into 4 phases, each with a clear **Approval Gate** that must be signed off before proceeding.

**Estimated total effort**: 6–8 developer-weeks  
**Risk if skipped**: Deploying without completing Phase 1 will result in runtime failures. Skipping Phase 2 leaves the system as a visually complete but functionally hollow shell.

---

## PHASE 1: UNBLOCK BUILDS (Week 1) — 5 Days

**Goal**: Resolve all issues that prevent `npm run build`, `cargo tauri build`, and `docker-compose up` from completing successfully.

### 1.1 TLS Certificate Fallback

| Item | Detail |
|------|--------|
| **File** | `backend/main.rs:476-478` |
| **Problem** | `fs::read_to_string(cert_path + "/server.crt")?` — `?` propagates as error, but there is no graceful fallback. If certs are absent, the gRPC server does not start. |
| **Fix** | Add conditional: if cert files not found, generate self-signed dev certs at startup using `rcgen` (already in `Cargo.toml`), or fall back to plaintext gRPC when `PILOT_MODE=true`. |
| **Effort** | 0.5 day |
| **Owner** | Backend Engineer |
| **Verification** | `cargo run` starts successfully with no certs present in `./certs/` |

### 1.2 Create Missing Infrastructure Artifacts

| Item | Detail |
|------|--------|
| **File** | `apps/dashboard/nginx.conf` (new) |
| **Problem** | `apps/dashboard/Dockerfile:25` does `COPY apps/dashboard/nginx.conf /etc/nginx/conf.d/default.conf` — file does not exist. Docker build fails. |
| **Fix** | Create nginx.conf with SPA routing (try_files $uri $uri/ /index.html), gzip, security headers. |
| **Effort** | 0.5 day |
| **Owner** | DevOps |

| Item | Detail |
|------|--------|
| **File** | `prometheus/alerts.yml` (new) |
| **Problem** | `prometheus.yml:15` references `alerts.yml` which does not exist. Prometheus logs a warning on every startup. |
| **Fix** | Create minimal alerts.yml with standard rules (high error rate, pod crash, high latency). If no alert rules are desired, remove the reference from `prometheus.yml`. |
| **Effort** | 0.5 day |
| **Owner** | DevOps |

### 1.3 Remove Hardcoded Private Keys

| Item | Detail |
|------|--------|
| **File** | `apps/dashboard/src/App.tsx:644-666` |
| **Problem** | Two Ethereum private keys hardcoded in component state initialization. Even though these are well-known Goerli testnet keys, their presence establishes a pattern that will be repeated in production code. |
| **Fix** | Remove the `privateKey` field from the wallet state structure. Wallets should be loaded from encrypted storage or entered via the manual-add modal only. The mock data should use placeholder values (`"0x..."`) or be loaded from a dev-seed fixture that is `.gitignore`d. |
| **Effort** | 0.5 day |
| **Owner** | Frontend Engineer |
| **Verification** | `grep -r "0xac0974" apps/dashboard/src/` returns no results |

### 1.4 Fix Root package.json

| Item | Detail |
|------|--------|
| **File** | `package.json` (root) |
| **Problem** | Minified to single line; old audit cited duplicate `devDependencies` key. Current state is valid JSON but unreadable for maintenance. |
| **Fix** | Pretty-print. Confirm only one `devDependencies` key exists. Ensure it contains `@tauri-apps/cli` (already present at `^2.11.3`). |
| **Effort** | 0.25 day |
| **Owner** | Any |

### 1.5 Version String Consolidation

| Item | Detail |
|------|--------|
| **Files** | `backend/Cargo.toml`, root `package.json`, `apps/dashboard/package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `IMPLEMENTATION_PLAN.md` |
| **Problem** | Six version 119.0` (workspace). Artifact naming will collide or confuse users. |
| **Fix** | Establish single source of truth. Recommendation: |
| | - Backend crate: `0.60.0` (semver, workspace-aligned) |
| | - Root workspace: `0.60.0` |
| | - Dashboard: inherit from workspace (remove independent version) |
| | - Desktop lib: `0.60.0` |
| | - Tauri bundle: `2026.1.0` (date-based is acceptable for consumer-facing installer) |
| **Effort** | 0.5 day |
| **Owner** | Any |
| **Verification** | `grep -r "version"` across all manifests shows aligned values |

### 1.6 Desktop Installer Build Chain Remediation

| Item | Detail |
|------|--------|
| **Source** | `IMPLEMENTATION_PLAN.md` |
| **Problem** | The desktop installer workflow is documented but not integrated into the remediation plan. Tauri CLI version confusion (audit reports claim v1.4/1.6; actual code is v2.11.3) will cause installer generation to fail if team follows audit guidance. |
| **Fix** | 1. Confirm `@tauri-apps/cli` v2.11.3 is installed globally or via npx. 2. Document exact commands for MSI and NSIS generation: `cd src-tauri && cargo tauri build` (both targets configured in `tauri.conf.json`). 3. Verify output paths: `src-tauri/target/release/bundle/msi/` and `src-tauri/target/release/bundle/nsis/`. 4. Add `certs/` directory creation to prerequisites (see 1.1). 5. Update `IMPLEMENTATION_PLAN.md` to reflect Tauri v2 and correct output paths. |
| **Effort** | 0.5 day |
| **Owner** | DevOps / Build Engineer |
| **Verification** | `cargo tauri build` produces both `.msi` and `.exe` (NSIS) in target bundle directories |

### 1.7 Frontend Build Verification

| Item | Detail |
|------|--------|
| **Problem** | No verified record of `npm run dashboard:build` completing in the current codebase state. Prior audit claimed success but that may have been on an older codebase. |
| **Fix** | Run `npm --prefix apps/dashboard run build` and capture output. Fix any TypeScript or Vite errors that surface. Confirm `dist/` assets are generated and match `tauri.conf.json` `frontendDist` path (`../apps/dashboard/dist`). |
| **Effort** | 0.5 day |
| **Owner** | Frontend Engineer |
| **Verification** | Build completes with exit code 0; `dist/index.html` exists |

### Phase 1 Approval Gate

```
[ ] 1.1 TLS fallback coded and tested
[ ] 1.2 nginx.conf created
[ ] 1.3 Private keys removed from source
[ ] 1.4 package.json readable and valid
[ ] 1.5 Version strings consolidated
[ ] 1.6 Desktop installer build chain verified (Tauri v2, MSI + NSIS)
[ ] 1.7 Frontend build verified clean

Approved by: _________________  Date: _______
```

---

## PHASE 2: COMPLETE STUB MODULES (Weeks 2–4) — 15 Days

**Goal**: Replace hardcoded returns and data-only structs with functional implementations across all 60 claimed modules.

### 2.1 CentralC2Server — Real Orchestration

| Stub | Current Code | Required Implementation |
|------|-------------|------------------------|
| `calculate_fleet_kpis()` | Returns hardcoded tuple | Query `runner_kpis` DashMap, compute weighted averages using `WEIGHT_ALPHA` through `WEIGHT_MARKET` constants (already defined at lines 109-114) |
| `calculate_single_runner_deflections()` | Returns hardcoded tuple | Pull per-runner metrics from `runner_kpis`, compute 7-dimension deflection vector |
| `execute_fleet_championship()` | Loops runners, discards results | Implement actual championship: load strategy config, execute `module_57_pool_dispatcher` route evaluation per runner, collect results, update `GlobalFleetState` |
| `run_copilot_decision_loop()` | Infinite tick calling stub | Add decision logic: evaluate `GlobalFleetState` against thresholds, invoke `opt_agent` for tuning, emit `RiskAlert` via broadcast when conditions warrant |
| `calibrate_simulation_fidelity()` | Sets one field | Wire to actual shadow replay comparison (M58) — compare simulated vs live deflection across last N trades |

**Effort**: 5 days  
**Owner**: Backend Lead

### 2.2 M01 — Wallet Management Engine (logic.rs)

| Gap | Fix |
|-----|-----|
| `profit_cache` is an empty DashMap — no profit data is ever written to it | Implement profit recording in the trade execution path. `profit_cache.insert(wallet_id.clone(), cumulative_profit)` after each settled trade. |
| `start_air_engine()` uses `println!` — no error handling, no structured logging | Replace with `tracing::info!`/`tracing::error!`. Add try/catch around `monitor_and_extract_profit()`. Log to structured telemetry. |
| `execute_autonomous_optimization()` is an empty println | Implement the 4-agent loop described in the comment (Quant, Risk, Security, Strategic). Each agent reads current state, proposes parameter delta, risk agent stress-tests, security agent validates, strategic agent authorizes. |
| `calculate_performance_vitals()` returns fixed values | Compute from actual trade history in PostgreSQL. Query last 24h trades, compute expected value, risk score, confidence interval. |

**Effort**: 3 days  
**Owner**: Backend Engineer

### 2.3 M03/M57 — Pool Dispatcher (module_57_pool_dispatcher.rs)

| Gap | Fix |
|-----|-----|
| `DexProtocol` enum and `fee_bps()` are data-only — no routing logic | Implement `evaluate_routes(token_in, token_out, amount, max_routes)` that: (1) filters DEXes by `supports_flash_loans()` if flash loan mode, (2) computes estimated output = amount * (1 - fee_bps/10000) * price_ratio, (3) sorts by output descending, (4) returns top N routes. |
| No price integration | Add `reqwest`-based price fetch from DEX subgraph or on-chain quote. For stub, return deterministic mock with documented TODO. |
| No integration with `CentralC2Server` | Add `pool_shard_registry` population — register active pools per runner at startup. |

**Effort**: 3 days  
**Owner**: Backend Engineer

### 2.4 M04 — Shadow Replay Engine (module_58_shadow_replay.rs)

| Gap | Fix |
|-----|-----|
| Structs defined (`HistoricalTrade`, `AnomalyResult`, `ShadowReplayEngine`) but no execution methods | Implement: `ingest_trade(trade)` — push to VecDeque, `detect_anomalies()` — z-score analysis on last 100 blocks, `score_opportunity(route)` — multi-factor scoring (liquidity depth, historical win rate, competitor pressure). |
| No integration with `CentralC2Server` | Wire `perform_shadow_replay_audit()` to actually call `detect_anomalies()` and log results to telemetry. |

**Effort**: 2 days  
**Owner**: Backend Engineer

### 2.5 M06 — Auto-Optimization (auto_optimization.rs)

| Gap | Fix |
|-----|-----|
| Atomic statics and config structs only — no optimization loop | Implement `run_optimization_cycle()` that: (1) reads current fleet state, (2) identifies underperforming parameters (e.g., corridor width too tight), (3) proposes new values within configured bounds, (4) applies via `set_autonomy_settings` gRPC call. |
| `DEX_UNIVERSE` is a static list — no dynamic discovery | Add periodic refresh from DEX subgraphs or registry. Log new DEXes discovered. |
| Chain/region counts are static atomics | Make them computed from actual active runners and pool registries. |

**Effort**: 2 days  
**Owner**: Backend Engineer

### 2.6 M21-M25 — Regional Modules (regional_modules.rs)

**Action**: Full code review against `MODULE_AUDIT_REPORT.md` claims. Verify each function has an execution path that is reachable from `CentralC2Server`. If any are stubs, implement them.

**Effort**: 2 days  
**Owner**: Backend Engineer

### 2.7 M71-M80 — Infrastructure Modules

| File | Required Verification |
|------|----------------------|
| `fleet_controller.rs` | Confirm runner lifecycle management (spawn, health check, kill) is reachable from gRPC `global_kill_switch` |
| `k8s_manager.rs` | Confirm K8s pod scaling logic is implemented (not just structs) |
| `cert_utils.rs` | Confirm mTLS cert generation/rotation works |
| `key_manager.rs` | Confirm key rotation triggers actual rotation, not just logging |
| `learning/mod.rs` | Confirm model update triggers retraining pipeline, not just `println` |

**Effort**: 3 days  
**Owner**: Backend Lead

### Phase 2 Approval Gate

```
[ ] 2.1 CentralC2Server stubs replaced with real logic
[ ] 2.2 WME profit_cache populated from trade execution
[ ] 2.3 Pool dispatcher has working route evaluation
[ ] 2.4 Shadow replay has execute/detect/score methods
[ ] 2.5 Auto-optimization has runnable cycle
[ ] 2.6 Regional modules verified functional
[ ] 2.7 Infrastructure modules verified functional
[ ] All cargo tests pass (or documented why none exist)

Approved by: _________________  Date: _______
```

---

## PHASE 3: INFRASTRUCTURE HARDENING (Week 5) — 5 Days

**Goal**: Make the Docker/K8s/RunPod deployment stack actually runnable and observable.

### 3.1 Database Migration Tooling

| Item | Detail |
|------|--------|
| **Current state** | `wme_schema.sql` is a single init script. No versioned migrations. |
| **Fix** | Add `sqlx migrate` to project. Split `wme_schema.sql` into timestamped migrations (`202506230001_create_runner_wallets.up.sql`, etc.). Add migration runner to `backend/main.rs` startup. |
| **Effort** | 1 day |
| **Owner** | Backend Engineer |

### 3.2 Health Check Improvements

| Item | Detail |
|------|--------|
| **Current state** | Backend Dockerfile has `grpcurl` healthcheck. Dashboard has `wget`. |
| **Fix** | Add backend `/health` HTTP endpoint (separate from gRPC) that checks DB connectivity, Redis connectivity, and cert validity. Update Prometheus scrape to use it. |
| **Effort** | 1 day |
| **Owner** | Backend Engineer |

### 3.3 Secrets Management

| Item | Detail |
|------|--------|
| **Current state** | `DATABASE_URL=postgresql://apxuser:apxpass@postgres:5432/allbright` in `docker-compose.yml` and `backend/main.rs` defaults. |
| **Fix** | Move all secrets to `.env` file (already `.gitignore`d if present). Add `.env.example` with placeholder values. Document required env vars. Add startup validation — fail fast with clear message if `PRIVATE_KEY`, `RPC_URL`, or `DATABASE_URL` are missing in non-pilot mode. |
| **Effort** | 1 day |
| **Owner** | DevOps |

### 3.4 RunPod Configuration

| Item | Detail |
|------|--------|
| **Current state** | `SOVEREIGN_AUDIT_REPORT.md` references `runpod-fleet-config.yaml` — file does not exist in repo. |
| **Fix** | Create `runpod-fleet-config.yaml` with actual image reference (to be built and pushed to registry), resource specs, CPU feature requirements, and mTLS config. |
| **Effort** | 1 day |
| **Owner** | DevOps |

### 3.5 CI/CD Pipeline

| Item | Detail |
|------|--------|
| **Current state** | No CI/CD configuration visible. |
| **Fix** | Add GitHub Actions (or equivalent): |
| | 1. On PR: `cargo check`, `cargo clippy`, `tsc --noEmit`, `npm run lint` |
| | 2. On main: `cargo test`, `npm run dashboard:build`, `docker build` for backend + dashboard |
| | 3. On tag: `cargo tauri build` for MSI + NSIS, push to releases |
| **Effort** | 1.5 days |
| **Owner** | DevOps |

### 3.6 Observability

| Item | Detail |
|------|--------|
| **Current state** | `telemetry.rs` and `metrics.rs` exist but integration level unverified. Prometheus config exists. |
| **Fix** | Verify tonic `/metrics` endpoint is mounted. Confirm all 7 deflection KPIs are exported as Prometheus gauges. Add Grafana dashboard JSON (or import from `grafana/` directory if it exists). |
| **Effort** | 0.5 day |
| **Owner** | Backend Engineer |

### Phase 3 Approval Gate

```
[ ] 3.1 sqlx migrations replace init SQL
[ ] 3.2 /health endpoint returns 200 when all dependencies healthy
[ ] 3.3 .env.example documented, secrets out of docker-compose.yml
[ ] 3.4 runpod-fleet-config.yaml created with real image ref
[ ] 3.5 CI/CD pipeline runs on PR and push to main
[ ] 3.6 Prometheus scrapes real metrics, Grafana dashboard loads

Approved by: _________________  Date: _______
```

---

## PHASE 4: DOCUMENTATION & AUDIT CORRECTION (Week 6) — 5 Days

**Goal**: Ensure all documentation accurately reflects the codebase. Prior audits contained material inaccuracies that constitute a deployment risk.

### 4.1 Rewrite MODULE_AUDIT_REPORT.md

| Issue | Correction |
|-------|-----------|
| Claims "FULLY INTEGRATED" for M01, M02, M03, M06, M58, M60 | Reclassify to "PARTIAL — stub implementation present, execution logic pending" |
| Claims "EXCEEDS TARGET" for DEX coverage (58 DEX) | Clarify: 58 DEX protocols defined in enum, 0 have working price/routing integration |
| Claims "EXCELLENT" rating | Downgrade to "ARCHITECTURE SOUND, IMPLEMENTATION INCOMPLETE" |
| Missing | Add section on stub-to-real implementation gap per module |

### 4.2 Rewrite SOVEREIGN_AUDIT_REPORT.md

| Issue | Correction |
|-------|-----------|
| References Tauri CLI 1.4 / 1.6 | Update to Tauri 2.11.3 |
| Claims "9.5/10 deployment ready" | Downgrade to "4/10 — critical blockers present" |
| Claims TypeScript issues "VERIFIED RESOLVED" | Verify against current `App.tsx` — `apexDeflection` appears in vitals state but `FleetState` interface in `types.ts` does not define it. Re-verify or fix. |
| References non-existent `runpod-fleet-config.yaml` | Remove or mark as pending |
| Claims "3 TypeScript blocking issues resolved" | Re-audit current state |

### 4.3 Rewrite DEPLOYMENT_READINESS_AUDIT.md

| Issue | Correction |
|-------|-----------|
| Score: "10/10" | Downgrade to actual score |
| Claims "all resolved" for TypeScript issues | Re-verify |
| References Tauri 1.4 | Update to Tauri 2.11.3 |

### 4.4 Rewrite IMPLEMENTATION_PLAN.md

| Issue | Correction |
|-------|-----------|
| Only covers desktop installer build | Expand to full-stack: backend, frontend, docker, tauri, k8s, runpod |
| version 119.0.0` inconsistency | Align with Phase 1 version decision |
| Missing phases for backend logic completion | Add Phase 2 reference |

### 4.5 Create AUDIT_HISTORY.md

| Purpose | Detail |
|---------|--------|
| Record | Document that prior audits (MODULE_AUDIT_REPORT, SOVEREIGN_AUDIT_REPORT, DEPLOYMENT_READINESS_AUDIT) contained material inaccuracies |
| Lessons | What went wrong: audits rated presence of files and structs as "integrated" rather than testing execution paths |
| Process | Define new audit standard: every "INTEGRATED" claim must have a verified execution path with at least one integration test or manual test procedure |

### Phase 4 Approval Gate

```
[ ] 4.1 MODULE_AUDIT_REPORT.md rewritten with accurate ratings
[ ] 4.2 SOVEREIGN_AUDIT_REPORT.md corrected (Tauri version, score, missing files)
[ ] 4.3 DEPLOYMENT_READINESS_AUDIT.md corrected
[ ] 4.4 IMPLEMENTATION_PLAN.md covers full stack
[ ] 4.5 AUDIT_HISTORY.md created with lessons learned
[ ] All documentation reviewed by second engineer

Approved by: _________________  Date: _______
```

---

## RISK REGISTER

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| Phase 2 takes longer than estimated (stubs are deeper than assessed) | Medium | Schedule slip | Time-box each stub implementation. If a module is too complex, mark it as "deferred to V119.1" and ship without it rather than block release. |
| Hardcoded keys found in additional locations | Low | Security | Run `grep -r "0x[0-9a-f]{64}"` across entire repo before Phase 1 close |
| TLS cert generation adds unwanted complexity for devs | Low | Build friction | Make cert generation opt-in via env var. Default to plaintext in pilot mode. |
| sqlx migrations conflict with existing wme_schema.sql | Medium | DB breakage | Test migrations on clean Postgres instance before committing |
| Team resists rewriting audit docs (sunk cost) | Medium | Continued misinformation | Frame as "audit maturity improvement" — same standard we'd apply to any vendor audit |

---

## APPROVAL REQUEST

I request approval to proceed with **Phase 1 (Unblock Builds)** beginning immediately. Each subsequent phase requires its own approval gate before work begins.

Please indicate your decision:

**OPTIONS**:
- **Approve Phase 1** — Begin work on unblocking builds immediately
- **Approve All Phases** — Begin Phase 1, proceed through all phases with per-phase check-ins
- **Request Changes** — Modify this plan (specify below)
- **Defer** — Do not begin remediation at this time

**Selected option**: _________________

**Additional instructions**:
_________________________________________________________________
_________________________________________________________________
