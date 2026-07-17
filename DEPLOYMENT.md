# AllBright DeFi V119 - Localhost Deployment Guide

This guide provides step-by-step instructions for deploying the AllBright arbitrage flash loan application on localhost.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Quick Start](#quick-start)
3. [Docker Compose Deployment](#docker-compose-deployment)
4. [Frontend-Only Development](#frontend-only-development)
5. [Tauri Desktop App](#tauri-desktop-app)
6. [Backend-Only Development](#backend-only-development)
7. [Configuration](#configuration)
8. [Engine Modes Guide](#engine-modes-guide)
9. [Troubleshooting](#troubleshooting)
10. [Security Checklist](#security-checklist)

---

## Prerequisites

### Required Software

- **Node.js** 20+ and npm 9+ ([Download](https://nodejs.org/))
- **Docker** and Docker Compose ([Download](https://www.docker.com/get-started))
- **Rust** 1.88+ (for backend development) - Only if building backend from source
- **Git** - For cloning the repository

### Optional (for Tauri Desktop Development)

- **WebView2** (Windows) - Usually pre-installed on Windows 10/11
- **webkit2gtk** (Linux) - `sudo apt install libwebkit2gtk-4.1-dev`
- **Xcode Command Line Tools** (macOS) - `xcode-select --install`

### Verify Installation

```bash
# Check Node.js
node --version  # Should be 20.12.0+
npm --version   # Should be 9+

# Check Docker
docker --version
docker compose version

# Check Rust (optional, for backend)
rustc --version  # Should be 1.88+
cargo --version
```

---

## Quick Start

### 1. Clone and Setup

```bash
# Clone the repository (if not already done)
git clone <your-repo-url>
cd ALLBRIGHT

# Install frontend dependencies
npm run install:dashboard

# Copy environment template
cp .env.example .env

# Edit .env with your actual values
nano .env  # or use your preferred editor
```

### 2. Start All Services (Recommended)

```bash
# Start PostgreSQL, Redis, and Backend
docker compose up -d postgres redis backend

# Wait 10-15 seconds for services to initialize
# Check backend health
curl http://localhost:50051/health

# In a new terminal, start the frontend
npm run dashboard:dev
```

### 3. Access the Application

- **Frontend Dashboard**: http://localhost:3000
- **Backend gRPC**: localhost:50051
- **Backend HTTP**: localhost:3001
- **PostgreSQL**: localhost:5432
- **Redis**: localhost:6379
- **Prometheus**: http://localhost:9090

---

## Docker Compose Deployment

### Full Stack (All Services)

```bash
# Start all services
docker compose up -d

# View logs
docker compose logs -f

# Stop all services
docker compose down

# Stop and remove volumes (WARNING: deletes data)
docker compose down -v
```

### Individual Services

```bash
# Start only database services
docker compose up -d postgres redis

# Start only backend
docker compose up -d backend

# Start only frontend (requires backend running)
docker compose up -d dashboard
```

### Rebuild After Code Changes

```bash
# Rebuild specific service
docker compose up -d --build backend

# Rebuild all services
docker compose up -d --build
```

---

## Frontend-Only Development

### Development Mode (Hot Reload)

```bash
# Install dependencies
npm run install:dashboard

# Start Vite dev server
npm run dashboard:dev
# or from root:
npm run dev

# Access at http://localhost:3000
```

### Production Build

```bash
# Build for production
npm run dashboard:build

# Preview production build
npm run dashboard:preview
```

### Using Docker (Production)

```bash
# Build and run just the dashboard
docker compose up -d --build dashboard

# Check if running
curl http://localhost:5173
```

---

## Tauri Desktop App

### Prerequisites

Install Tauri CLI dependencies:

**Windows:**
```powershell
# Install WebView2 (usually pre-installed)
# Install Visual Studio C++ Build Tools
# Install Microsoft Visual Studio C++ (MSVC) compiler
```

**macOS:**
```bash
xcode-select --install
```

**Linux:**
```bash
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libssl-dev \
  libappindicator3-dev \
  librsvg2-dev
```

### Development

```bash
# Install dependencies
npm run install:dashboard

# Start Tauri dev mode
npm run desktop:dev

# This will:
# 1. Build the frontend
# 2. Open a desktop window running the app
# 3. Enable hot reload for changes
```

### Production Build

```bash
# Build for current platform
npm run desktop:build

# Build Windows installers (MSI + NSIS)
npm run desktop:build:msi-nsis

# Installers will be in:
# src-tauri/target/release/bundle/nsis/
# src-tauri/target/release/bundle/msi/
```

---

## Backend-Only Development

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify installation
rustc --version
cargo --version
```

### Build and Run

```bash
cd backend

# Build release binary
cargo build --release

# Run the server
cargo run --release

# Or run directly
./target/release/allbright-c2-backend
```

### Using Docker

```bash
# Build backend Docker image
docker compose build backend

# Run backend
docker compose up -d backend

# View logs
docker compose logs -f backend
```

### Backend Health Check

```bash
# gRPC health check
grpcurl -plaintext localhost:50051 list

# Expected output: allbright.c2.FleetCommand
```

---

## Configuration

### Environment Variables

**Critical Files:**
- `.env` - Main environment configuration (DO NOT COMMIT)
- `.env.example` - Template with all available options
- `apps/dashboard/.env` - Frontend-specific variables (uses VITE_* prefix)

### Key Variables to Configure

```bash
# Required for AI features
OPENAI_API_KEY=sk-...
GROQ_API_KEY=gsk_...
OPENROUTER_API_KEY=sk-or-v1-...

# Required for blockchain interaction
PRIVATE_KEY=0x...
WALLET_ADDRESS=0x...
CHAIN_ID=1

# Database
DATABASE_URL=postgresql://apxuser:apxpass@localhost:5432/allbright

# Modes
PAPER_TRADING_MODE=true  # Use true for testing!
VITE_DEMO_MODE=true     # Use true for testing!
```

### Port Configuration

Default ports (can be customized in docker-compose.yml):

| Service       | Port  | Protocol | Description                    |
|---------------|-------|----------|--------------------------------|
| Frontend      | 3000  | HTTP     | Vite dev server / Dashboard    |
| Dashboard     | 5173  | HTTP     | Docker Nginx production        |
| Backend gRPC  | 50051 | gRPC     | Main backend API               |
| Backend HTTP  | 3001  | HTTP     | Backend REST API               |
| Backend WS    | 50052 | WS       | WebSocket real-time updates    |
| PostgreSQL    | 5432  | TCP      | Database                       |
| Redis         | 6379  | TCP      | Cache                          |
| Prometheus    | 9090  | HTTP     | Metrics                        |

---

## Engine Modes Guide

The AllBright system provides **6 engine modes** for different operational states. These modes must be executed in a specific order for safe operation.

### Mode Progression Workflow

```
CONNECT_ENDPOINTS (Required First)
    ↓
DEBUG (Architecture Audit - Optional but Recommended)
    ↓
PREFLIGHT (Agent Attestation - Required Before Trading)
    ↓
    ├─→ SIMULATION (Testing - No Private Keys)
    ├─→ PILOT (Controlled Deployment - Private Keys Enabled)
    └─→ LIVE (Full Production - Complete Autonomy)
```

---

### 1. CONNECT AND SECURE (System Binding & HSM Activation)

**Action**: `CONNECT_AND_SECURE`  
**Security Level**: Foundation (Required)  
**Private Key Access**: No  

**Purpose:**  
Establishes secure connections to all configured RPC endpoints, API services, and AI providers from your `.env` configuration. Activates system security and HSM/TSS readiness.

**What It Does:**
- Reads all `VITE_*` and backend environment variables
- Establishes gRPC-web telemetry streams with configured providers
- Validates HSM (Hardware Security Module) / TSS (Threshold Signature Scheme) readiness
- Binds RPC/API endpoints for subsequent engine modes
- Activates system security protocols

**When to Use:**
- **MUST be run first** before any other engine mode
- After changing `.env` configuration
- After network connectivity changes
- On first startup after installation

**Prerequisites:**
- Backend service running (localhost:50051)
- All RPC endpoints configured in `.env`
- Valid API keys for AI services

**Security Notes:**
- No private keys required or used
- Safe to run multiple times
- Validates endpoint authentication

---

### 2. DEBUG (CEIO Architecture Audit)

**Action**: `INITIALIZE_ARCH_AUDIT`  
**Security Level**: Diagnostic (Read-Only)  
**Private Key Access**: No  

**Purpose:**  
Comprehensive verification of the Unified Intelligence Ecosystem (Modules M01-M60). Ensures 100% control over the specialized agent matrix.

**What It Does:**
- Invokes Phase 1-5 validation protocols
- Runs bit-perfect SMC (State Machine Controller) parity checks
- Validates all silicon pathways and agent registrations
- Checks Module 60 census completeness
- Verifies architecture integrity

**When to Use:**
- After system updates or upgrades
- When troubleshooting agent issues
- To verify system integrity before trading
- After modifying agent configurations

**Safety:**
- **Read-only mode** - no trading or state changes
- Safe to run at any time
- Does not require private keys

**Output:**
- Architecture validation report
- Agent matrix status
- Module census verification
- Bit-perfect parity check results

---

### 3. PREFLIGHT (Agent Attestation)

**Action**: `RUN_PREFLIGHT`  
**Security Level**: Validation (Required Before Trading)  
**Private Key Access**: No  

**Purpose:**  
Comprehensive system validation across all 8 specialist agents before capital deployment. Ensures system integrity and readiness.

**What It Does:**
- Invokes agent attestation across all pillars
- Validates cryptographic posture
- Verifies risk parameters
- Checks HSM/TSS operational readiness
- Validates multi-agent coordination
- Confirms system security baseline

**When to Use:**
- **Required before** SIMULATION, PILOT, or LIVE modes
- After DEBUG mode completes
- Before any capital deployment
- When switching between trading modes

**Safety:**
- **Mandatory gate** - cannot execute trading modes without passing
- Does not require private keys
- Validates but does not execute trades

**Output:**
- Status: `PENDING` | `RUNNING` | `PASSED` | `FAILED`
- Agent readiness status (all 8 agents)
- Cryptographic posture validation
- Risk parameter verification

---

### 4. SIMULATION (Shadow-Fork)

**Action**: `START_SIMULATION`  
**Security Level**: Testing (Zero Risk)  
**Private Key Access**: **NO** (Explicitly excluded)  

**Purpose:**  
High-fidelity Shadow-Fork environment for strategy validation without risking master collateral.

**What It Does:**
- Uses full `.env` context **excluding private keys**
- Determines sample size and epoch duration based on target confidence score
- Validates multi-hop routing robustness
- Calculates Expected Value (EV) without real funds
- Shadow-forks blockchain state for realistic testing

**When to Use:**
- Initial strategy testing
- After PREFLIGHT passes
- When testing new configurations
- Before PILOT mode
- For strategy optimization

**Safety:**
- **Zero financial risk** - no private keys used
- Uses simulated/sandbox environment
- Cannot execute real transactions
- Safe for unlimited testing

**Required Configuration:**
```bash
PRIVATE_KEY=0x...  # Will be IGNORED in simulation
PAPER_TRADING_MODE=true
VITE_DEMO_MODE=true
```

---

### 5. PILOT (Gated Node Orchestration)

**Action**: `DEPLOY_PILOT`  
**Security Level**: Controlled Risk (Private Keys Enabled)  
**Private Key Access**: **YES** (Gated to 1-1000 nodes)  

**Purpose:**  
Strategic orchestration across gated nodes for incremental grid saturation testing with real capital movement.

**What It Does:**
- Commander specifies node count (1-1000) and duration
- Private keys enabled for controlled capital deployment
- Monitors real-time multi-agent performance vitals
- Allows incremental testing with limited exposure
- Gathers performance metrics for LIVE mode

**When to Use:**
- After successful SIMULATION testing
- When ready to test with real funds (small scale)
- To validate live trading with limited nodes
- Before committing to LIVE mode

**⚠️ CRITICAL SECURITY NOTES:**
- Private keys are active - **real funds at risk**
- Always start with minimum nodes (1-10) to test
- Set profit wallet to monitor actual gains
- Enable MEV protection
- Monitor continuously during execution

**Required Configuration:**
```bash
PRIVATE_KEY=0x...  # Required - REAL FUNDS
PAPER_TRADING_MODE=false  # Real trading
VITE_DEMO_MODE=false  # Real trading
```

---

### 6. LIVE (Sovereign Intelligence Apex)

**Action**: `AUTHORIZE_APEX`  
**Security Level**: Full Production (Uncapped Risk)  
**Private Key Access**: **YES** (Full access, uncapped)  

**Purpose:**  
Uncapped autonomous execution powered by the full Unified Intelligence Ecosystem with complete private key access.

**What It Does:**
- Full AVX-512 utilization
- Persistent YubiKey FIDO2 heartbeat authentication
- Private keys fully enabled for all transactions
- Autonomous decision-making across all 91 AI agents
- Unrestricted grid saturation
- Maximum capital deployment

**When to Use:**
- **Only after** extensive SIMULATION and PILOT testing
- When all security validations pass
- With hardware wallet (YubiKey) configured
- With comprehensive monitoring in place
- When ready for full production

**⚠️ MAXIMUM RISK WARNING:**
- **Uncapped autonomous execution** - full control over funds
- Requires YubiKey FIDO2 heartbeat (hardware wallet)
- All 91 AI agents operate autonomously
- No manual intervention during execution
- Only use after extensive testing in PILOT mode

**Required Configuration:**
```bash
PRIVATE_KEY=0x...  # Required - Hardware wallet recommended
PAPER_TRADING_MODE=false  # Real trading
VITE_DEMO_MODE=false  # Real trading
# YubiKey FIDO2 must be configured and connected
```

---

## Safe Testing Progression

**Recommended Path to Production:**

```
1. CONNECT_ENDPOINTS
   ↓
2. DEBUG (verify system integrity)
   ↓
3. PREFLIGHT (validate all agents)
   ↓
4. SIMULATION (test strategies, no risk)
   ↓ (repeat until confident)
5. PILOT - Start with 1-10 nodes
   ↓ (monitor for 24-48 hours)
6. PILOT - Scale to 100-1000 nodes
   ↓ (monitor for 1-2 weeks)
7. LIVE (full production)
```

**Time Recommendations:**
- SIMULATION: 1-2 weeks of testing
- PILOT (1-10 nodes): 3-7 days minimum
- PILOT (100-1000 nodes): 2-4 weeks minimum
- LIVE: Only after 1+ month of successful PILOT testing

---

## Quick Reference Table

| Mode | Private Keys | Risk Level | Use Case | Required |
|------|-------------|------------|----------|----------|
| CONNECT_ENDPOINTS | No | None | System binding | Yes (first) |
| DEBUG | No | None | Architecture audit | Recommended |
| PREFLIGHT | No | None | Agent validation | Yes (before trading) |
| SIMULATION | No | Zero | Strategy testing | No |
| PILOT | Yes | Controlled | Limited deployment | No |
| LIVE | Yes | Maximum | Full production | No |

---

## Troubleshooting Engine Modes

**Problem**: Cannot proceed past CONNECT_ENDPOINTS
```bash
# Check backend is running
curl http://localhost:50051/health

# Check .env configuration
cat .env | grep VITE_

# Verify RPC endpoints
curl $RPC_ENDPOINT
```

**Problem**: PREFLIGHT keeps failing
```bash
# Check agent logs
docker compose logs backend | grep -i "agent.*fail"

# Verify HSM/TSS status
# Check if security module is initialized
```

**Problem**: SIMULATION not using shadow-fork
```bash
# Ensure demo mode is enabled
echo $VITE_DEMO_MODE  # Should be "true"

# Check private key is set (will be ignored)
echo $PRIVATE_KEY  # Should be set but not used
```

---

## Security Best Practices

1. **Always run CONNECT_ENDPOINTS first** - validates all connections
2. **Never skip PREFLIGHT** - ensures system integrity
3. **Test extensively in SIMULATION** - zero risk validation
4. **Start PILOT with minimum nodes** - limit exposure
5. **Monitor PILOT continuously** - watch for anomalies
6. **Use hardware wallet for LIVE** - YubiKey FIDO2 required
7. **Keep demo mode on for testing** - prevents accidental live trading
8. **Review logs after each mode** - catch issues early

---

## Troubleshooting

### Frontend Issues

**Problem**: Port 3000 already in use
```bash
# Solution: Kill process on port 3000
# Windows:
netstat -ano | findstr :3000
taskkill /PID <pid> /F

# macOS/Linux:
lsof -ti:3000 | xargs kill -9
```

**Problem**: Module not found errors
```bash
# Solution: Reinstall dependencies
rm -rf node_modules apps/dashboard/node_modules
npm run install:dashboard
```

**Problem**: Vite HMR not working
```bash
# Check if DISABLE_HMR is set
echo $DISABLE_HMR  # Should be empty or 'false'
```

### Backend Issues

**Problem**: Cannot connect to PostgreSQL
```bash
# Check if PostgreSQL is running
docker compose ps postgres

# Check PostgreSQL logs
docker compose logs postgres

# Test connection
docker compose exec postgres pg_isready -U apxuser -d allbright
```

**Problem**: Backend build fails
```bash
# Check Rust version
rustc --version  # Must be 1.88+

# Clean and rebuild
cd backend
cargo clean
cargo build --release
```

**Problem**: gRPC connection refused
```bash
# Verify backend is running
docker compose logs backend

# Check if port 50051 is exposed
netstat -an | grep 50051

# Test with grpcurl
grpcurl -plaintext localhost:50051 list
```

### Docker Issues

**Problem**: Permission denied errors
```bash
# Linux: Add user to docker group
sudo usermod -aG docker $USER
newgrp docker

# Or run with sudo (not recommended for production)
sudo docker compose up -d
```

**Problem**: Out of disk space
```bash
# Clean up Docker
docker system prune -a --volumes

# Remove unused images
docker image prune -a
```

**Problem**: Container keeps restarting
```bash
# Check container logs
docker compose logs <service-name>

# Common causes:
# - Missing environment variables
# - Database not ready yet
# - Port already in use
```

### Tauri Issues

**Problem**: WebView not found (Linux)
```bash
# Install webkit2gtk
sudo apt install libwebkit2gtk-4.1-dev
```

**Problem**: Build fails on Windows
```bash
# Install Visual Studio C++ Build Tools
# Download from: https://visualstudio.microsoft.com/downloads/
```

---

## Security Checklist

### Before First Deployment

- [ ] **Rotate all API keys** - Generate new keys for OpenAI, Groq, etc.
- [ ] **Generate secure SESSION_SECRET** - Use `openssl rand -hex 64`
- [ ] **Change DASHBOARD_PASS** - Use a strong password
- [ ] **Verify .env is in .gitignore** - Already configured ✓
- [ ] **Use hardware wallet** for production trading
- [ ] **Enable PAPER_TRADING_MODE** for initial testing
- [ ] **Set VITE_DEMO_MODE=true** for initial testing
- [ ] **Review firewall rules** - Only expose necessary ports
- [ ] **Enable MEV protection** if trading live
- [ ] **Monitor wallet activity** - Set up alerts

### Production Deployment

- [ ] **Use different .env files** for dev/staging/prod
- [ ] **Implement secrets management** (vault.enc, AWS Secrets Manager, etc.)
- [ ] **Enable TLS** - Configure certificates in backend
- [ ] **Restrict database access** - Use strong passwords, SSL connections
- [ ] **Enable Redis authentication** - Set Redis password
- [ ] **Regular backups** - Automate PostgreSQL backups
- [ ] **Monitor and alerting** - Set up Prometheus + Grafana
- [ ] **Rate limiting** - Implement API rate limits (disabled for HFT)

---

## Common Workflows

### Development Workflow

```bash
# Terminal 1: Start databases
docker compose up -d postgres redis

# Terminal 2: Start backend
docker compose up -d backend

# Terminal 3: Start frontend with hot reload
npm run dashboard:dev

# Make changes to frontend code - auto-reloads
# Check backend logs:
docker compose logs -f backend
```

### Testing Workflow

```bash
# 1. Ensure demo mode is on
export VITE_DEMO_MODE=true
export PAPER_TRADING_MODE=true

# 2. Start services
docker compose up -d

# 3. Run tests
npm run lint
cargo test --manifest-path backend/Cargo.toml

# 4. Test frontend
npm run dashboard:dev
# Open http://localhost:3000 and test UI
```

### Production Deployment

```bash
# 1. Configure environment
cp .env.example .env
# Edit .env with production values
nano .env

# 2. Build everything
npm run build

# 3. Start services
docker compose up -d

# 4. Verify health
curl http://localhost:3000/health
docker compose ps

# 5. Monitor logs
docker compose logs -f
```

---

## Next Steps

1. **Read the Security Guide**: Review `SECURITY.md` (if exists)
2. **Configure AI Agents**: Check `AI_AGENTS_GUIDE.md` (if exists)
3. **Set up Monitoring**: Configure Prometheus + Grafana
4. **Join Community**: Get support from other AllBright users

---

## Support

- **Documentation**: Check `/docs` folder
- **Issues**: Report bugs via GitHub Issues
- **Discord**: Join our Discord community
- **Email**: support@allbright.defi

---

## License

Proprietary - AllBright DeFi Software Engineering PLC
Copyright (c) 2024

**Last Updated**: 2026-06-28
**version 119.0.0
</parameter>
<task_progress>
- [x] Complete DEPLOYMENT.md with all 6 engine modes
- [x] Document mode progression and prerequisites
- [x] Add security implications for each mode
- [x] Include safe testing workflow
</task_progress>
</write_to_file>