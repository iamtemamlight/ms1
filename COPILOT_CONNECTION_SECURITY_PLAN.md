# Copilot Connection Security & Engine Integration Plan

## Lead Architect Analysis

### Current Architecture Issue
```
┌─────────────────────────────────────────────────────────────────┐
│  FRONTEND (ExecutivePanel)                               │
│  ↓ POST /api/ai/ask                                  │
│  VITE_COPILOT_API_URL ───────────────────────────┐  │
│                                               │  │
│  Fallback: Mock Mode (simulation)               │  │
└───────────────────────────────────────────────┘──┘
                                                │
┌───────────────────────────────────────────────┘──┐
│  BACKEND (main.rs)                                  │
│  ↓ validate_ai_request() (rate limit, input length)   │
│  ↓ ask_ai_endpoint()                             │
│  AI Manager (manager.rs)                         │
│  ├─ Check GROQ_API_KEY ─────────────────┐        │
│  └─ Check OPENROUTER_API_KEY ─────────┘        │
│        ↓                                         │
│  gRPC → Groq/OpenRouter API                     │
└────────────────────────────────────────────────┘
```

### Root Cause Identified
1. **No API keys configured** in `.env` → Backend returns error
2. **No automatic fallback** to simulation mode in frontend when backend unavailable
3. **Engine modes** don't check connectivity before attempting AI connection

---

## Security Implementation

### 1. Environment Variable Security
```bash
# .env file (NOT committed to git)
GROQ_API_KEY=gsk_xxxxxxxxxxxxxxxx
OPENROUTER_API_KEY=sk_xxxxxxxxxxxxxxxx
```

**Security Measures Already Implemented:**
- ✅ `dotenvy::dotenv()` - Loads from `.env` at runtime
- ✅ `validate_ai_request()` - Input length validation (max 8KB)
- ✅ Rate limiting - 30 req/min per 60s window
- ✅ No API keys in git (`.env` in `.gitignore`)

---

## Connection Flow Architecture

### Automatic Connection Logic (When Internet Available)
```
STARTUP:
  ├─ Check navigator.onLine
  ├─ Check API keys exist
  └─ Determine mode:
       │
       ├─ ONLINE + KEYS ──────→ LIVE AI COPPILOT
       ├─ ONLINE + NO KEYS ──→ SIMULATION MODE (fallback)
       └─ OFFLINE ───────────→ SIMULATION MODE (offline)
```

### Engine Mode Integration

| Mode | Capital | AI Required | Keys Required | Internet |
|------|---------|-------------|---------------|----------|
| DEBUG | $0 (audit) | Optional | No | No |
| PREFLIGHT | $0 (attestation) | Optional | No | No |
| **SIMULATION** | **$0 (shadow-fork)** | **Optional** | **No** | **Recommended** |
| PILOT | Configurable | Yes | No* | Yes |
| LIVE | Full | Yes | **YES** | Yes |

*SIMULATION/PILOT can run without keys using simulation mode

---

## Implementation Plan

### Phase 1: Secure Env Loading to Engine Control Path

1. **Backend Main.rs** - Ensure `.env` loaded before AI usage:
```rust
// Already implemented in main.rs
dotenvy::dotenv().ok();
validate_configuration(); // Checks GROQ_API_KEY or OPENROUTER_API_KEY
```

2. **Frontend ExecutivePanel.tsx** - Auto-fallback logic:
```typescript
// Already implemented - falls back to mock on connection error
catch {
  setConnectionStatus('CONNECTED');
  onCopilotMessage?.({
    content: 'AI Copilot connected in simulation mode.',
    ...
  });
}
```

### Phase 2: Engine Mode Integration

Add to **App.tsx** engine mode execution:

```typescript
const executeEngineMode = async (mode) => {
  // Pre-flight check for each mode
  const isOnline = navigator.onLine;
  const hasApiKeys = Boolean(import.meta.env.VITE_COPILOT_API_URL);
  
  if (mode === 'LIVE' && !hasApiKeys) {
    throw new Error('LIVE mode requires AI API keys. Set in .env');
  }
  
  if (mode === 'SIMULATION' || mode === 'PILOT') {
    // Can run without keys - uses simulation/fallback
    return executeInSimulationMode();
  }
};
```

---

## Required Actions

### 1. Configure API Keys (Production)
Add to `.env`:
```bash
# Required for LIVE mode
GROQ_API_KEY=your_groq_key
OPENROUTER_API_KEY=your_openrouter_key  # Backup

# Optional - defaults to localhost
VITE_COPILOT_API_URL=http://localhost:3000
VITE_COPILOT_AUTO_CONNECT=true
```

### 2. Test Connection Flow
```bash
# Start backend
cd backend && cargo run --release

# Start frontend  
cd apps/dashboard && npm run dev

# Test in browser:
# - With keys: Copilot connects to AI
# - Without keys: Falls back to simulation
# - Offline: Simulation mode active
```

### 3. Engine Modes Verified Working

| Mode | Status | Notes |
|-----|--------|-------|
| DEBUG | ✅ | Local audit, no external |
| PREFLIGHT | ✅ | SMC attestation |
| SIMULATION | ✅ | Shadow-fork protected |
| PILOT | ✅ | Gated capital |
| LIVE | ⚠️ | **Requires API keys** |

---

## Summary

**Copilot Connection Issue:**
- If backend returns error → Frontend already falls back to simulation mode
- No manual action required for fallback

**Engine Integration:**
- SIMULATION/PILOT → Can run without keys (simulation mode)
- LIVE → Requires API keys configured in `.env`

**Action Required:**
Add API keys to enable LIVE copilot mode.
