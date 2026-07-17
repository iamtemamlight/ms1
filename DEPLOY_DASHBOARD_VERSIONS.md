# AllBright Dashboard — Two Deployment Versions with Reserved LocalPorts
**Generated:** 2026-07-15  
**Scope:** Tauri Desktop (MSI/NSIS) + React/Vite (Web) — both on reserved ports

---

## Port Reservation Map

All reserved ports across both deployment versions:

| Port | Version | Service | Protocol |
|------|---------|---------|----------|
| **3000** | Both | Rust production backend (default) | HTTP/REST |
| **3001** | Both | Rust production backend (.env override) | HTTP/REST |
| **3002** | Vite/Web | Express simulation server (UI dev only) | HTTP/REST |
| **50051** | Both | Rust backend gRPC | gRPC |
| **50052** | Both | Rust backend WebSocket | WS |
| **5173** | Vite/Web | Vite dev server (HMR) | HTTP |
| **5180** | Vite/Web | Vite production preview | HTTP |
| **5200** | Vite/Web | Nginx production proxy | HTTP/HTTPS |
| **8545** | Both | LocalPort RPC relay (ETH) | JSON-RPC |
| **8546** | Both | LocalPort RPC relay (BASE) | JSON-RPC |
| **8547** | Both | LocalPort RPC relay (POLYGON) | JSON-RPC |
| **8548** | Both | LocalPort RPC relay (ARBITRUM) | JSON-RPC |
| **8549** | Both | LocalPort RPC relay (OPTIMISM) | JSON-RPC |
| **9090** | Both | Prometheus metrics | HTTP |
| **9093** | Both | Alertmanager | HTTP |
| **5432** | Both | PostgreSQL | TCP |

---

# VERSION 1: Tauri Desktop Installer (MSI + NSIS)

## Architecture
```
┌──────────────────────────────────────────────────────────────────┐
│                    TAURI DESKTOP APP                              │
│  ┌──────────────────────────────────────────────────────────┐    │
│  │  Tauri Shell (Rust)                                      │    │
│  │  • MSI installer: AllBright_Dashboard_v119.0.0.msi       │    │
│  │  • NSIS installer: AllBright_Dashboard_v119.0.0.exe      │    │
│  │  • Publisher: AllBright DeFi Software Engineering Ltd.   │    │
│  └──────────────────────┬───────────────────────────────────┘    │
│                          │                                        │
│  ┌──────────────────────▼───────────────────────────────────┐    │
│  │  Tauri WebView (embedded browser)                        │    │
│  │  • React SPA loaded from `apps/dashboard/dist/`          │    │
│  │  • VITE_API_BASE=http://127.0.0.1:3001                   │    │
│  │  • CSP: connect-src 'self' ws: wss: http://127.0.0.1    │    │
│  └──────────────────────┬───────────────────────────────────┘    │
│                          │                                        │
│                          ▼                                        │
│               ┌──────────────────────┐                           │
│               │  Rust Backend (:3001)│◄────── gRPC :50051        │
│               │  Real 78 KPI Data    │                           │
│               │  Sub-0.1ms latency   │                           │
│               └──────────────────────┘                           │
└──────────────────────────────────────────────────────────────────┘
```

## Configuration: `tauri.conf.json`

```json
{
  "build": {
    "beforeBuildCommand": "npm run build",
    "beforeDevCommand": "",
    "devPath": "",
    "distDir": "../apps/dashboard/dist"
  },
  "package": {
    "active": true,
    "targets": "msi,nsis",
    "icon": ["../icons/icon.ico"]
  },
  "tauri": {
    "bundle": {
      "active": true,
      "targets": "msi,nsis",
      "identifier": "com.allbright.defi",
      "icon": ["../icons/icon.ico"],
      "publisher": "AllBright DeFi Software Engineering Ltd.",
      "windows": [{
        "title": "AllBright Dashboard - Production",
        "width": 1200,
        "height": 800,
        "resizable": true,
        "fullscreen": false
      }]
    },
    "security": {
      "csp": "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self' data:; connect-src 'self' ws: wss: http://127.0.0.1:3001 http://127.0.0.1:8545 http://127.0.0.1:8546 http://127.0.0.1:8547 http://127.0.0.1:8548 http://127.0.0.1:8549; frame-ancestors 'self';"
    },
    "allowlist": {
      "all": false,
      "fs": { "scope": ["$APPDATA/**", "$RESOURCE/**"] },
      "shell": { "open": true },
      "process": { "relaunch": false },
      "dialog": { "all": true }
    }
  }
}
```

## Build Commands

```powershell
# === BUILD TAURI DESKTOP INSTALLERS (MSI + NSIS) ===

# Step 1: Build the React frontend
cd apps/dashboard
npm install
npm run build

# Step 2: Build Tauri desktop installers
cd src-tauri
cargo tauri build --bundles msi,nsis

# Output files:
#   src-tauri/target/release/bundle/msi/AllBright_Dashboard_v119.0.0_x64.msi
#   src-tauri/target/release/bundle/nsis/AllBright_Dashboard_v119.0.0_x64-setup.exe
```

## Running the Desktop App

```powershell
# Install via MSI (silent mode):
msiexec /i AllBright_Dashboard_v119.0.0_x64.msi /quiet

# Install via NSIS (silent mode):
AllBright_Dashboard_v119.0.0_x64-setup.exe /S

# Post-install: Start Rust backend before launching app
cd backend
cargo run --release
# Rust backend listens on :3001 per .env

# Launch the installed desktop app
# Start Menu > AllBright Dashboard
```

## Port Map for Tauri Version

| Port | Bound To | Purpose |
|------|----------|---------|
| **3001** | `127.0.0.1` | Rust backend HTTP API |
| **50051** | `127.0.0.1` | Rust backend gRPC |
| **50052** | `127.0.0.1` | Rust backend WebSocket |
| **8545-8549** | `127.0.0.1` | LocalPort RPC relay |
| **9090** | `127.0.0.1` | Prometheus |

---

# VERSION 2: React + Vite Web Application

## Architecture
```
┌──────────────────────────────────────────────────────────────────┐
│                    REACT + VITE WEB APP                          │
│                                                                  │
│  ┌──────────────────┐    ┌──────────────────┐                   │
│  │  DEV MODE         │    │  PRODUCTION MODE │                   │
│  │  Vite HMR Server  │    │  Express Static  │                   │
│  │  :5173 (hot reload)│   │  :3002 (served)  │                   │
│  │  OR               │    │  OR              │                   │
│  │  Express Dev      │    │  Nginx Proxy     │                   │
│  │  :3002 (full sim) │    │  :5200 (HTTPS)   │                   │
│  └────────┬─────────┘    └────────┬─────────┘                   │
│           │                       │                              │
│           └───────────┬───────────┘                              │
│                       │                                          │
│                       ▼                                          │
│            ┌──────────────────────┐                             │
│            │  API_BASE Select     │                             │
│            │  ENV: VITE_API_BASE  │                             │
│            │  = :3001 (Rust)      │                             │
│            │  or :3002 (Express)  │                             │
│            └────────┬─────────────┘                             │
│                     │                                            │
│                     ▼                                            │
│          ┌──────────────────────┐                               │
│          │  Rust Backend (:3001)│◄────── gRPC :50051            │
│          │  Real 78 KPI Data    │                               │
│          │  Sub-0.1ms latency   │                               │
│          └──────────────────────┘                               │
└──────────────────────────────────────────────────────────────────┘
```

## Configuration: `vite.config.ts`

```typescript
import tailwindcss from '@tailwindcss/vite';
import react from '@vitejs/plugin-react';
import path from 'path';
import { defineConfig } from 'vite';

export default defineConfig(() => {
  return {
    plugins: [react(), tailwindcss()],
    resolve: {
      alias: {
        '@': path.resolve(__dirname, '.'),
      },
    },
    build: {
      outDir: 'dist',
      sourcemap: false,
      minify: 'esbuild',
      rollupOptions: {
        output: {
          manualChunks: {
            vendor: ['react', 'react-dom', 'recharts'],
            ui: ['lucide-react'],
          },
        },
      },
    },
    server: {
      port: 5173,
      strictPort: false,    // Allow fallback to 5174 if 5173 is busy
      host: '127.0.0.1',
      proxy: {
        // DEV: Proxy API calls to Rust backend on :3001
        '/api': {
          target: 'http://127.0.0.1:3001',
          changeOrigin: true,
          secure: false,
        },
      },
    },
    preview: {
      port: 5180,
      host: '127.0.0.1',
    },
  };
});
```

## Configuration: `apps/dashboard/.env.development`

```env
# Vite Dev Mode — connects to Express simulation (UI prototyping)
VITE_API_BASE=http://localhost:3002
VITE_ENGINE_MODE=simulation
VITE_DEMO_MODE=true
VITE_DEBUG=true
VITE_LOG_LEVEL=debug
```

## Configuration: `apps/dashboard/.env.production`

```env
# Vite Production Mode — connects to Rust backend (real 78 KPI data)
VITE_API_BASE=http://localhost:3001
VITE_ENGINE_MODE=production
VITE_DEMO_MODE=false
VITE_DEBUG=false
VITE_LOG_LEVEL=info
```

## Build & Run Commands

```powershell
# === DEV MODE (Hot Reload + Express Simulation) ===

# Terminal 1: Start Express simulation (fake data for UI dev)
cd apps/dashboard
npm install
npm run dev
# Express listens on :3002 — serves Vite HMR + API simulation

# Open browser: http://localhost:5173
# Vite proxies /api/* to Express :3002


# === PRODUCTION MODE (Static Build + Rust Backend) ===

# Terminal 1: Start Rust production backend
cd backend
set HTTP_BIND_ADDR=0.0.0.0:3001
cargo run --release
# Rust backend listens on :3001 — real 78 KPI data

# Terminal 2: Build and preview React frontend
cd apps/dashboard
npm install
npm run build

# Option A: Express production server (port 3002)
npm run start
# Open: http://localhost:3002

# Option B: Vite preview (port 5180)
npx vite preview --port 5180 --host 127.0.0.1
# Open: http://localhost:5180

# Option C: Nginx production (port 5200 — HTTPS ready)
# Use nginx.conf below
```

## Production Nginx Configuration: `apps/dashboard/nginx.conf`

```nginx
server {
    listen 5200 ssl;
    server_name localhost;

    # SSL (optional — for production with real cert)
    ssl_certificate     ./certs/localhost.pem;
    ssl_certificate_key ./certs/localhost-key.pem;

    root C:/MS1/AB4/apps/dashboard/dist;
    index index.html;

    # Serve static assets with caching
    location /assets/ {
        expires 1y;
        add_header Cache-Control "public, immutable";
    }

    # Proxy API calls to Rust backend on :3001
    location /api/ {
        proxy_pass http://127.0.0.1:3001;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # Proxy WebSocket to Rust backend on :50052
    location /ws/ {
        proxy_pass http://127.0.0.1:50052;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }

    # SPA fallback — all other routes serve index.html
    location / {
        try_files $uri $uri/ /index.html;
    }

    # Security headers
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
}
```

## Port Map for Vite/Web Version

| Port | Mode | Service | Purpose |
|------|------|---------|---------|
| **3001** | Production | Rust backend HTTP | Real 78 KPI data |
| **3002** | Dev/Prod | Express simulation/serve | UI dev + static prod |
| **5173** | Dev only | Vite HMR server | Hot reload dev |
| **5180** | Prod only | Vite preview | Production preview |
| **5200** | Prod only | Nginx HTTPS proxy | Production serving |
| **50051** | Both | Rust gRPC | Fleet command |
| **50052** | Both | Rust WebSocket | Real-time data |
| **8545-8549** | Both | LocalPort RPC | EVM multi-chain |
| **9090** | Both | Prometheus | Metrics |

---

## Comparison Matrix

| Feature | Tauri Desktop (MSI/NSIS) | React + Vite (Web) |
|---------|--------------------------|-------------------|
| **Distribution** | Windows installer | Browser URL |
| **Install Size** | ~50-80 MB | 0 (browser) |
| **Rust Backend** | Bundled separately | Separate process |
| **LocalPort RPC** | External (node relay) | External (node relay) |
| **Auto-start backend** | Manual (separate terminal) | Manual (separate terminal) |
| **Hot Reload** | ❌ Full rebuild needed | ✅ Vite HMR |
| **Security** | CSP + allowlist | Browser sandbox + CSP |
| **Port Conflict** | None (unique ports) | None (unique ports) |
| **HTTPS** | ❌ (localhost) | ✅ (Nginx :5200) |
| **Start Command** | Start Menu shortcut | `npm run dev` or URL |
| **Build Command** | `cargo tauri build` | `npm run build` |
| **Install Command** | MSI/NSIS installer | N/A |

---

## Quick Start: Both Versions Side-by-Side

```powershell
# ====================================================================
# STEP 1: Start Infrastructure (shared between both versions)
# ====================================================================
# Terminal A: Start Rust production backend (REQUIRED for real 78 KPI data)
cd backend
set HTTP_BIND_ADDR=0.0.0.0:3001
cargo run --release

# Terminal B: Start LocalPort RPC relay (EVEN multi-chain proxy)
node localport-rpc-relay.mjs

# Terminal C: Start Docker infrastructure
docker-compose up -d postgres redis prometheus

# ====================================================================
# STEP 2a: Start Tauri Desktop Version
# ====================================================================
# Terminal D: Build frontend + Tauri installers
cd apps/dashboard
npm install
npm run build
cd ../../
npm run tauri:build
# Install from: src-tauri/target/release/bundle/msi/*.msi
# Launch from Start Menu

# ====================================================================
# STEP 2b: Start React + Vite Web Version
# ====================================================================
# Terminal D: Dev mode (hot reload)
cd apps/dashboard
npm install
npm run dev
# Open: http://localhost:5173

# Terminal E: Production build
cd apps/dashboard
npm run build
npx vite preview --port 5180
# Open: http://localhost:5180

# ====================================================================
# VERIFICATION
# ====================================================================
# Test Rust backend health:
curl http://localhost:3001/api/health

# Test LocalPort RPC:
curl http://localhost:8545 -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'

# Test Prometheus:
curl http://localhost:9090/api/v1/query?query=up
```

---

## Environment File for Tauri Desktop Build

Create `apps/dashboard/.env.tauri` for desktop-specific build:

```env
# Tauri Desktop Build Config
VITE_API_BASE=http://127.0.0.1:3001
VITE_ENGINE_MODE=production
VITE_DEMO_MODE=false
VITE_DEBUG=false
VITE_LOG_LEVEL=info
VITE_WS_URL=ws://127.0.0.1:50052
```

---

## File Structure

```
AB4/
├── apps/dashboard/
│   ├── .env.development          ← Vite dev: Express :3002
│   ├── .env.production           ← Vite prod: Rust :3001
│   ├── .env.tauri               ← Tauri build: Rust :3001
│   ├── vite.config.ts           ← Vite with proxy, ports, chunks
│   ├── nginx.conf               ← Nginx production config
│   └── src/
│       ├── App.tsx              ← API_BASE selects backend
│       └── components/          ← 7 dashboard components
├── src-tauri/
│   ├── tauri.conf.json          ← Tauri with restricted allowlist
│   └── Cargo.toml               ← Tauri Rust dependencies
├── backend/
│   ├── main.rs                  ← Rust backend (HTTP :3000/3001)
│   └── .env                     ← HTTP_BIND_ADDR=0.0.0.0:3001
├── DEPLOY_DASHBOARD_VERSIONS.md ← THIS FILE — deployment guide
└── localport-rpc-relay.mjs      ← RPC relay :8545-8549
```

---

*Both versions are ready for LocalPort deployment on reserved ports.*
*See `DASHBOARD_LOCALPORT_READINESS_100_PERCENT.md` for component readiness details.*