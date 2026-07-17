# ALLBRIGHT C2 — Render Deployment Guide

Generated: 2026-07-17
Repository: https://github.com/iamtemamlight/ms1

---

## 🚀 Deployment Configuration

This repository is configured for deployment on **Render** using the `render.yaml` blueprint.

### Services Deployed

| Service | Type | Port | Plan | Health Check |
|---------|------|------|------|--------------|
| allbright-backend | Web (Docker) | 3001 | Starter | `/healthz` |
| allbright-dashboard | Web (Docker) | 80 | Starter | `/` |
| allbright-redis | Redis | 6379 | Starter | Internal |
| allbright-db | PostgreSQL | 5432 | Starter | Internal |

---

## 📋 Pre-Deployment Checklist

### 1. **Set Up Render Account**
- [ ] Sign up at [render.com](https://render.com)
- [ ] Create new account or log in
- [ ] Connect your GitHub account

### 2. **Prepare GitHub Repository**
- [ ] Push this repository to GitHub:
  ```bash
  git remote -v
  # Should show: origin git@github.com:iamtemamlight/ms1.git (fetch)
  
  # If not set, add remote:
  git remote add origin git@github.com:iamtemamlight/ms1.git
  
  # Push to main branch:
  git add .
  git commit -m "feat: Add Render deployment configuration"
  git push -u origin main
  ```

### 3. **Configure Render Environment Variables**

**Critical:** These must be set in the Render Dashboard before deployment:

#### Backend Service (`allbright-backend`)
1. Go to Render Dashboard → Your Service → Environment
2. Add these variables:

| Key | Value | Notes |
|-----|-------|-------|
| `OPENROUTER_API_KEY` | `<set-in-render-dashboard>` | From your .env |
| `GROQ_API_KEY` | `<set-in-render-dashboard>` | From your .env |
| `OPENAI_API_KEY` | `<set-in-render-dashboard>` | From your .env |
| `PIMLICO_API_KEY` | `<set-in-render-dashboard>` | From your .env |
| `PRIVATE_KEY` | `<set-in-render-dashboard>` | **CRITICAL** - From your .env (simulation mode safe) |

**Optional** (for live trading):
- `WALLET_ADDRESS`
- `EXECUTOR_ADDRESS`
- `FLASHLOAN_CONTRACT_ADDRESS`
- `ETH_RPC_URL`
- `BASE_RPC_URL`
- Other chain RPC URLs

#### Dashboard Service (`allbright-dashboard`)
Add these variables:

| Key | Value | Notes |
|-----|-------|-------|
| `VITE_BACKEND_API_URL` | Auto-populated by render.yaml | Points to backend service |
| `VITE_WS_URL` | Auto-populated by render.yaml | Points to backend service |
| `VITE_ENGINE_MODE` | `simulation` | **Keep as simulation for safety** |
| `VITE_DEMO_MODE` | `true` | Enable demo mode |
| `VITE_DEBUG` | `false` | Disable debug in production |

---

## 🔒 Security Notes

### API Keys & Secrets
- **DO NOT** commit API keys to GitHub
- Use Render's environment variable UI to set secrets
- The `render.yaml` uses `sync: false` for sensitive keys (they won't be synced from local .env)

### Simulation Mode
- **Current configuration uses `VITE_ENGINE_MODE=simulation`**
- This is **SAFE** - no real transactions will be executed
- Change to `production` only after:
  1. Smart contract audit completed
  2. CircuitBreaker wired into Rust backend
  3. Auth middleware added to HTTP endpoints
  4. Manual testing in shadow-fork mode

### Exposed Keys Warning
Your `.env` file contains exposed API keys. Before going live:
1. **Rotate ALL API keys** listed in the .env file
2. Review with external security auditor
3. Add auth middleware before exposing HTTP API to internet

---

## 🚀 Deployment Steps

### Step 1: Create Render Blueprint
1. Log into [Render Dashboard](https://dashboard.render.com)
2. Click **"New+"** → **"Blueprint"**
3. Connect your GitHub repository: `iamtemamlight/ms1`
4. Select branch: `main`
5. Select `render.yaml` file
6. Click **"Apply"**

### Step 2: Set Environment Variables
1. Wait for blueprint to provision services
2. Go to each service → **Environment** tab
3. Add the environment variables listed above
4. Save changes (triggers redeploy)

### Step 3: Verify Deployment
1. Check service logs for errors
2. Test health endpoints:
   - Backend: `https://allbright-backend.onrender.com/healthz`
   - Dashboard: `https://allbright-dashboard.onrender.com/`
3. Verify WebSocket connection: `wss://allbright-backend.onrender.com`

### Step 4: Post-Deployment
1. Update DNS records if using custom domain
2. Enable auto-deploy on GitHub pushes
3. Set up monitoring alerts in Render
4. Review Render logs for first 24 hours

---

## 🔧 Configuration Details

### Port Mapping
- Backend HTTP: **3001** (configured in `HTTP_BIND_ADDR`)
- Backend gRPC: **50051**
- Backend WebSocket: **50052**
- Dashboard: **80** (nginx default)

### Build Process
**Backend:**
1. Uses `rust:1.96-slim` builder image
2. Compiles Rust binary with LTO optimization
3. Copies binary to `debian:bookworm-slim` runtime
4. Runs as non-root user (`appuser`)

**Dashboard:**
1. Uses `node:20-alpine` builder image
2. Runs `npm ci --only=production`
3. Builds React app with Vite
4. Serves with `nginx:alpine`

### Database
- Render PostgreSQL with automatic backups
- Connection string auto-injected via `fromDatabase`
- Initial schema migration required (run `db-init` binary on first deploy)

### Redis
- Render managed Redis (starter plan)
- Connection string auto-injected via `fromService`
- Used for caching and session storage

---

## 🐛 Troubleshooting

### Backend Not Starting
1. Check Render logs for compilation errors
2. Verify `DATABASE_URL` and `REDIS_URL` are set
3. Ensure database has run initial migrations

### Dashboard Shows 502/504
1. Backend service may still be building (Rust takes 5-10 min)
2. Check backend health endpoint
3. Verify `VITE_BACKEND_API_URL` points to correct backend URL

### WebSocket Connection Failed
1. Render free tier may not support WebSockets (upgrade to paid plan)
2. Check backend gRPC port (50051) is accessible
3. Verify firewall rules in Render

### Database Connection Issues
1. Verify `DATABASE_URL` format (Render provides this)
2. Check database is in same region as services
3. Run database migrations manually if needed

---

## 📊 Monitoring

### Render Built-in Monitoring
- CPU/Memory usage graphs
- Request latency metrics
- Error rate tracking
- Auto-scaling metrics (if configured)

### Application Logs
- View logs in Render Dashboard
- Set up log drains for external aggregation
- Configure alerts for error spikes

### Health Checks
- Backend: `GET /healthz` → `200 OK`
- Dashboard: `GET /` → HTML response
- Database: Managed by Render

---

## 🔄 Updates & Redeployment

### Automatic Deployments
- Render auto-deploys on push to `main` branch
- Requires `render.yaml` to be in repository root

### Manual Redeployment
1. Go to service → **"Manual Deploy"** → **"Deploy latest commit"**
2. Or trigger via Render CLI:
   ```bash
   render deploy
   ```

### Rollback
1. Go to service → **"Events"** tab
2. Click **"Rollback"** on previous successful deployment

---

## 📈 Scaling

### Current Setup (Starter Plan)
- Backend: 0.5 CPU, 512 MB RAM
- Dashboard: 0.5 CPU, 512 MB RAM
- Redis: 256 MB
- Database: 1 GB storage

### Upgrade Path
1. Go to service → **"Settings"** → **"Plan"**
2. Select higher tier (Standard/Pro)
3. Update autoscaling rules in `render.yaml` if needed

---

## 🎯 Next Steps (Post-MVP)

1. **Smart Contract Audit**
   - Remove double-approve pattern
   - Add deadline/slippage enforcement
   - External security audit for mainnet

2. **Backend Hardening**
   - Wire CircuitBreaker into execution flow
   - Add auth middleware (JWT/OAuth)
   - Restrict CORS to specific origins
   - Bind HTTP to internal interface

3. **Monitoring Enhancement**
   - Add AlertManager config to Prometheus
   - Verify metrics registry wiring
   - Set up PagerDuty/OpsGenie integration

4. **Rotate Exposed Keys**
   - Generate new API keys for all services
   - Update in Render environment variables
   - Revoke old keys

---

## 📞 Support

- Render Docs: https://render.com/docs
- Repository: https://github.com/iamtemamlight/ms1
- Issues: https://github.com/iamtemamlight/ms1/issues

---

**Status:** ✅ Ready for deployment
**Last Updated:** 2026-07-17
**Deployed By:** Render Blueprint
**Environment:** Production (Simulation Mode)