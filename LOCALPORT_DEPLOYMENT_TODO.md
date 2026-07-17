# AllBright LOCALPORT Deployment Plan

## Executive Summary
Deploy AllBright arbitrage flash loan app to localport using apps/dashboard/src/components as web-based dashboard.

---

## Information Gathered

### Current Architecture
- **Backend**: Rust gRPC server on port 50051, HTTP REST API on port 3000
- **Docker Compose**: Backend, Postgres (5432), Redis (6379), Prometheus (9090), Dashboard (5173)
- **EngineControl.tsx**: 6 engine modes with complete mode progression workflow
- **91 AISE Agents**: Registered and executable via copilot loop
- **LOCALPORT Protocol**: 5 ports (8545-8549) reserved for LOCALPORT mode

### Engine Modes Progression (Part-Based)

```
CONNECT_ENDPOINTS (Required First)
    ↓
DEBUG: Part 1 - Independent verification
    │   - 72 KPIs + Security Layers 1-10 + Silicon
    │   - Zero Checksum = 0 required
    ↓
PREFLIGHT: Part 2 - Independent re-verification  
    │   - Security Layers + Profit Logic + Silicon
    │   - Zero Checksum = 0 required
    ↓
    ├─→ SIMULATION: Part 3 - Risk-Free Validation
    │       - Shadow-fork strategy
    │       - Risk < 0.45, Deflection ≥ 0
    │       - Zero Checksum = 0 required
    │   ↓
    ├─→ PILOT: Part 4 - Controlled Live
    │       - 1-1000 nodes, Live RPC
    │       - Deflection ≥ 0.8, Zero Checksum = 0 required
    │   ↓
    └─→ LIVE: Full Production
            - Chief Architect + YubiKey approval
            - Deflection ≥ 0.8 required
```

### Part-Based Verification Checklist

**Part 1 (DEBUG)**:
- [ ] 72 KPIs verified independently (curl /kpi/KPI-XX endpoints)
- [ ] Security Layers 1-5 verified
- [ ] Security Layers 6-10 verified  
- [ ] Silicon integration verified (91 agents, 5s copilot)
- [ ] Zero Checksum = 0 validated

**Part 2 (PREFLIGHT)**:
- [ ] Security Layers 1-10 re-verified
- [ ] Profit Logic validated (M001 → M057 → M054)
- [ ] Wallet/Withdrawal validated
- [ ] Security Gate (HSM/YubiKey) confirmed
- [ ] AISE stack (M051-M053) active
- [ ] ZK Proof (M099) active
- [ ] Agent Matrix (91 agents) confirmed
- [ ] Silicon re-verification complete
- [ ] Zero Checksum = 0 validated

**Part 3 (SIMULATION)**:
- [ ] Shadow-fork strategy validated
- [ ] Backtesting >99% success rate
- [ ] Risk < 0.45 threshold
- [ ] Deflection ≥ 0 achieved
- [ ] Zero Checksum = 0 validated

**Part 4 (PILOT)**:
- [ ] Node count (1-1000) configured
- [ ] Live RPC data flow confirmed
- [ ] Profit path active
- [ ] Gasless enabled
- [ ] Deflection ≥ 0.8 achieved
- [ ] Zero Checksum = 0 validated

---

## Implementation Plan

### Phase 1: LocalPort Infrastructure Setup

- [ ] 1.1 Configure Docker Compose for localport mode
  - [ ] Map ports 8545-8549 for RPC endpoints
  - [ ] Ensure backend on 50051 (gRPC), 3000 (HTTP)
  - [ ] Ensure Postgres on 5432, Redis on 6379
- [ ] 1.2 Verify backend builds successfully
  - [ ] `cd backend && cargo build --release`
- [ ] 1.3 Start infrastructure services
  - [ ] `docker compose up -d postgres redis`

### Phase 2: Backend Deployment

- [ ] 2.1 Start backend server
  - [ ] `cargo run` (or `docker compose up -d backend`)
- [ ] 2.2 Verify gRPC endpoint
  - [ ] `grpcurl -plaintext localhost:50051 list`
- [ ] 2.3 Verify HTTP health
  - [ ] `curl http://localhost:3000/healthz`

### Phase 3: Dashboard Deployment

- [ ] 3.1 Install dependencies
  - [ ] `npm run install:dashboard`
- [ ] 3.2 Configure environment
  - [ ] Set VITE_API_URL=http://localhost:3000
  - [ ] Set VITE_BACKEND_API_URL=http://localhost:3001
- [ ] 3.3 Start dashboard
  - [ ] `npm run dashboard:dev`
- [ ] 3.4 Access dashboard
  - [ ] http://localhost:3000

### Phase 4: Engine Control Integration

- [ ] 4.1 Connect endpoints (CONNECT_AND_SECURE)
  - [ ] Import .env file
  - [ ] Click CONNECT button
  - [ ] Verify endpoint status shows connected
- [ ] 4.2 Run PREFLIGHT validation
  - [ ] Click PREFLIGHT button
  - [ ] Wait for attestation PASSED
- [ ] 4.3 Select operational mode
  - [ ] SIMULATION (zero risk testing)
  - [ ] PILOT (controlled deployment)
  - [ ] LIVE (full production - requires YubiKey)

### Phase 5: LOCALPORT-Specific Configuration

- [ ] 5.1 Port allocation
  - [ ] 8545 - Primary Fleet RPC
  - [ ] 8546 - Secondary RPC (Backup)
  - [ ] 8547 - Shadow-Fork Simulation
  - [ ] 8548 - Testing/QA
  - [ ] 8549 - Arbitrum One RPC Mirror
- [ ] 5.2 Wallet auto-detection
  - [ ] Verify MetaMask detection works
  - [ ] Test manual wallet add
- [ ] 5.3 Vault configuration
  - [ ] Set ALLBRIGHT_VAULT_PASSWORD
  - [ ] Enable encrypted .env

---

## Port Mapping Summary (With Redundancy - Minimum 2 Backups Per Service)

| Service | Primary Port | Backup Ports | Protocol |
|---------|-------------|--------------|----------|
| Backend gRPC | 50051 | 51051, 51052 | gRPC |
| Backend WS | 50052 | 51053, 51054 | WebSocket |
| Backend HTTP | 3001 | 3002, 3003 | HTTP |
| Postgres | 5432 | 5433, 5434 | TCP |
| Redis | 6379 | 6380, 6381 | TCP |
| Redis Backup 1 | 6382 | - | TCP |
| Redis Backup 2 | 6383 | - | TCP |
| Dashboard | 5173 | 5174, 5175 | HTTP |
| Dashboard Backup 1 | 5176 | - | HTTP |
| Dashboard Backup 2 | 5177 | - | HTTP |
| Prometheus | 9090 | 9091, 9092 | HTTP |
| Fleet RPC 1 | 8545 | 8550-8553 | HTTP |
| Fleet RPC 2 | 8546 | 8554, 8555 | HTTP |
| Fleet RPC 3 | 8547 | - | HTTP |
| Fleet RPC 4 | 8548 | - | HTTP |
| Fleet RPC 5 | 8549 | - | HTTP |
| LocalPort RPC Backup 1 | 8560, 8561 | - | HTTP |
| LocalPort RPC Backup 2 | 8562, 8563 | - | HTTP |

**Total Exposed Ports**: 40+ ports for maximum redundancy

---

## Redundancy Architecture

Each critical service has minimum 2 backup instances:
- **Backend**: 3 gRPC + 3 WebSocket + 3 HTTP endpoints
- **Database**: Primary + 2 replicas
- **Cache**: Primary + 2 replicas + 2 backup services
- **Dashboard**: Primary + 2 replicas + 2 backup services
- **RPC Endpoints**: 9 total (Primary + 4 backups x 2 chains)
- **Metrics**: Primary + 2 replicas

---

## Verification Commands

```bash
# 1. Check all services
docker compose ps

# 2. Backend health
curl http://localhost:3000/healthz

# 3. gRPC endpoints
grpcurl -plaintext localhost:50051 list

# 4. Dashboard
curl http://localhost:3000

# 5. Prometheus
curl http://localhost:9090/metrics

# 6. Database
docker compose exec postgres pg_isready -U apxuser -d allbright

# 7. Redis
docker compose exec redis redis-cli ping
```

---

## Engine Mode Actions

| Mode | Phase | Action | Required | Risk | Zero Checksum |
|------|-------|--------|----------|------|---------------|
| CONNECT_ENDPOINTS | Phase 0 | CONNECT_AND_SECURE | Yes (first) | None | N/A |
| DEBUG | Part 1 | INITIALIZE_ARCH_AUDIT | Recommended | None | Must = 0 |
| PREFLIGHT | Part 2 | RUN_PREFLIGHT | Yes (before progression) | None | Must = 0 |
| SIMULATION | Part 3 | START_SIMULATION | Optional | Zero | Must = 0 |
| PILOT | Part 4 | DEPLOY_PILOT | Optional | Controlled | Must = 0 |
| LIVE | Production | AUTHORIZE_APEX | No | Full | Must = 0 |

---

## User Override Protocol

| Override Level | Permission | When Available |
|----------------|------------|----------------|
| Commander | Dashboard approval | DEBUG/PREFLIGHT/SIMULATION |
| Commander Override | Manual trigger | SIMULATION→PILOT (if Deflection ≥ 0, Risk < 0.45, Zero Checksum = 0) |
| Chief Architect | YubiKey + Dashboard | PILOT→LIVE only |

---

## Deployment Status

**Phase 0**: CONNECT_ENDPOINTS - [ ] Pending
**Part 1**: DEBUG - [ ] Pending (72 KPIs + Security + Silicon)
**Part 2**: PREFLIGHT - [ ] Pending (Dual-verification)
**Part 3**: SIMULATION - [ ] Pending (Shadow-fork)
**Part 4**: PILOT - [ ] Pending (Controlled deployment)

---

## Tauri Desktop Deployment

### Option: Standalone Windows Desktop App

For Windows-only deployment without Docker:

| Step | Command | Description |
|------|---------|-------------|
| 1 | `deploy-tauri-standalone.bat` | Automated build script |
| 2 | `npm run dashboard:build` | Build frontend assets |
| 3 | `npm run tauri:build` | Create MSI installer |
| 4 | Desktop Icon | `AllBright Dashboard.exe` on Desktop |

**Tauri Configuration**:
- Frontend: `apps/dashboard/dist/`
- Backend: Embedded Rust FFI calls
- LocalPort Ports: 8545-8549 for RPC

**First Run**:
1. Double-click Desktop icon
2. CONNECT_ENDPOINTS → DEBUG → PREFLIGHT
3. Proceed through Part 1-4 mode verification

- Always start with CONNECT_ENDPOINTS, then DEBUG, then PREFLIGHT
- Zero Checksum = 0 required before each mode transition
- Use hardware wallet (YubiKey) for production
- Review security checklist in CHIEF_ARCHITECT_DEPLOYMENT_PLAN.md

**Created**: 2026-06-28
**Chief Architect**: AllBright DeFi Software Engineering PLC
