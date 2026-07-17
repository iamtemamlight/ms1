# AllBright LIVE Production Mode Guide

## Step-by-Step Instructions for Commander

### THE COMMANDER AUTHORIZES. THE COPILOT EXECUTES.

```
Commander │ ▼ Deployment Environment (Local / Cloud) │ ▼ Deployment Mode │ ▼ AllBright Copilot │ ▼ Fleet Capacity Manager │ ▼ Execution Fleet
```

---

## PHASE 1: Environment Configuration (Authorization)

**Step 1: Prepare .env file**
```bash
copy .env.production .env
# Edit .env with your credentials:
# - PRIVATE_KEY=0x... (your wallet private key)
# - ALLBRIGHT_VAULT_PASSWORD=... (secure password)
# - GROQ_API_KEY=gsk_...
# - OPENAI_API_KEY=sk-...
```

**Step 2: Upload Configuration via Dashboard**
1. Navigate to AllBright-Auto-Desktop (http://localhost:3001)
2. Click "ENV CONFIG" in sidebar
3. Upload your .env file
4. Verify all required keys are detected (green indicators)

**Step 3: Select Deployment Environment**
- Click "COMMAND POST" → "Deployment Pipeline"
- Choose **Local** or **Cloud**:
  - Local: For development/testing
  - Cloud: For production (recommended)

---

## PHASE 2: Security Activation (Authorization)

**Step 4: Enable Security Controls**
1. Click "SECURITY CONTROLS" in sidebar
2. Toggle ON:
   - YubiKey HSM (required for Live)
   - Encrypted Vault (required for Live)
   - Optional: Mutual TLS, RBAC

**Step 5: Select Fleet Capacity Mode**
- In Command Post → Fleet Capacity Mode:
  - **Auto (Emergent)**: Recommended - Copilot determines optimal capacity
  - **Manual**: Commander controls fleet size directly

---

## PHASE 3: Deployment Pipeline (Authorization & Monitoring)

**Step 6: Set Operating Parameters**
- Profit Target: 1.5% (recommended default)
- Risk Mode: Conservative
- Stability Mode: Normal

**Step 7: Execute Deployment Pipeline**
1. In "DEPLOYMENT PIPE" panel, click "Execute"
2. Select modes in sequence:
   - 🔍 **Preflight** → Validate all systems
   - 🧪 **Simulation** → Test execution pipeline
   - 🚀 **Live** → Enable real transactions

**Step 8: Monitor Copilot Guidance**
- Copilot Panel shows real-time status
- Watch for green checkmarks (PASS)
- Follow any recommendations displayed

---

## PHASE 4: LIVE Activation

**Step 9: Transition to Live**
1. Ensure Preflight shows PASS
2. Complete Simulation successfully
3. Click "Execute" → Select "Live"
4. Confirm security prompts

**Step 10: Verify Live Operation**
- Copilot shows: "Live mode active"
- Fleet status shows active runners
- No red/yellow alerts in whispers
- System Intelligence shows "Ready"

---

## LIVE Mode Checklist

| Step | Action | Commander | Copilot |
|------|--------|-----------|---------|
| 1 | .env configured | ⬜ | - |
| 2 | .env uploaded | ⬜ | - |
| 3 | Environment selected | ⬜ | - |
| 4 | Security enabled | ⬜ | - |
| 5 | Fleet mode selected | ⬜ | - |
| 6 | Profit/Risk/Stability set | ⬜ | - |
| 7 | Preflight executed | ⬜ | Auto-executes validation |
| 8 | Simulation completed | ⬜ | Auto-runs pipeline |
| 9 | Live authorized | ⬜ | Auto-validates safety gates |
| 10 | Live operation verified | ⬜ | Auto-supervises |

---

## Safety Rules (Auto-Enforced by Copilot)

Live deployment is **blocked** if:
- Preflight fails
- Wallet validation fails
- Critical infrastructure unavailable
- Risk exceeds authorized envelope
- Stability below minimum threshold
- AI confidence below configured minimum