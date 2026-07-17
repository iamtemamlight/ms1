# AllBright Dashboard — Deployment Status (Live)

**Deployed:** 2026-07-15 09:21 UTC  
**Version:** React + Vite (Dev Mode with Express Simulation)  
**Status:** ✅ **RUNNING**

---

## 🔗 Dashboard URL

```
http://localhost:3002
```

Open this in your browser to view the live AllBright Dashboard.

---

## Service Status

| Service | Port | Status | URL |
|---------|------|--------|-----|
| **Dashboard (React SPA)** | 3002 | ✅ RUNNING | http://localhost:3002 |
| **Express API (Simulation)** | 3002 | ✅ RUNNING | http://localhost:3002/api/health |
| **Rust Backend (Production)** | 3001 | ⏳ Compiling | (cargo run --release) |
| **LocalPort RPC Relay** | 8545-8549 | ⏳ Not started | node localport-rpc-relay.mjs |

---

## Verified Endpoints

```powershell
# Dashboard HTML (production build)
GET http://localhost:3002/
→ Status: 200, serves /assets/index-Ynq2vK3b.js (697KB React bundle)

# Health check
GET http://localhost:3002/api/health
→ {"status":"healthy","timestamp":1784132249358,"network":"Arbitrum Mainnet"}

# Live metrics
GET http://localhost:3002/api/metrics
→ Total Profit: $1398.2, Opportunities: 9, Collateral: $167,701

# Live arbitrage opportunities
GET http://localhost:3002/api/opportunities
→ 10 cross-DEX opportunities with net profit calculations
```

---

## What You'll See on the Dashboard

1. **Topbar** — AllBright V01 branding, theme toggle, currency selector, wallet balance
2. **Sidebar** — Dashboard / Command / Wallet / Compliance navigation + Kill Switch
3. **Dashboard View** — 7 metric cards (Detected, Executed, Win Rate, Avg Profit, Avg Gas, Latency, Security) + Cumulative Profit Trend chart + Live Opportunities table
4. **Commander View** — 6 autonomous control knobs + deployment pipeline
5. **Wallet View** — Smart contract revenue pool + manual settlement
6. **Copilot Panel** — AI assistant (Gemini/OpenRouter/Grok/Claude)
7. **Compliance View** — Governance reflection cards

---

## Notes

- The dashboard connects to the **Express simulation backend** on port 3002 by default (dev mode).
- For **production 78 KPI data** (sub-0.1ms latency), set `VITE_API_BASE=http://localhost:3001` and start the Rust backend.
- The Rust backend is still compiling in the background (cargo run --release takes 2-5 min on first build).
- To switch to production mode: edit `apps/dashboard/.env.production` and rebuild.

---

*Deployed via `start_vite.bat` → npm install + npm run dev (Express :3002 + Vite middleware)*
*Dashboard files: `apps/dashboard/src/` (App.tsx + 7 components + main.tsx)*
*Build output: `apps/dashboard/dist/` (production SPA)*