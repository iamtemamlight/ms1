# Copilot Requirements Per Mode & User Override Analysis

## 📋 **OVERVIEW**

### **Copilot System:**
- **Loop Interval:** 5 seconds
- **AI Agents:** 91 registered (AI001-AI091)
- **Providers:** OpenRouter, Groq, Google AI, OpenAI
- **Decision Loop:** `run_copilot_decision_loop()` in `backend/main.rs`

### **Mode Progression:**
```
CONNECT_ENDPOINTS → DEBUG → PREFLIGHT → SIMULATION → PILOT → LIVE
```

---

## 🤖 **COPILOT REQUIREMENTS PER MODE**

### **Mode 1: CONNECT_ENDPOINTS**

**Copilot Status:** ⚠️ OFFLINE (waiting for connection)

| Requirement | Status | Details |
|-------------|--------|---------|
| API Keys | ❌ Not configured | OPENAI_API_KEY, GROQ_API_KEY missing |
| Backend Connection | ❌ Not connected | VITE_BACKEND_API_URL not set |
| WebSocket | ❌ Not connected | VITE_WS_URL not set |
| Agent Registry | ❌ Not loaded | Requires backend connection |
| Copilot Loop | ❌ Not running | Depends on backend |

**Copilot Actions:**
```typescript
// BEFORE connection:
- Show "Copilot disconnected" status
- Display configuration checklist
- Guide user to import .env file
- Validate API keys format

// AFTER connection:
- Initialize agent registry (91 agents)
- Start 5-second decision loop
- Begin fleet KPI calculation
- Enable AI recommendations
```

**User Override:** ❌ NOT ALLOWED
- Reason: Copilot cannot function without backend connection
- Override Attempt: Will show error "Connect endpoints first"

---

### **Mode 2: DEBUG**

**Copilot Status:** 🟡 ACTIVE (Part 1 - Independent Verification)

| Requirement | Status | Details |
|-------------|--------|---------|
| API Keys | ✅ Required | At least 1 AI provider configured |
| Backend Connection | ✅ Required | Connected in CONNECT mode |
| Agent Registry | ✅ Required | 91 agents must load |
| Copilot Loop | ✅ Running | 5-second interval active |
| Fleet KPI Access | ✅ Required | Read 72 KPIs |

**Copilot Actions:**
```typescript
// DEBUG mode copilot loop (every 5s):
1. calculate_fleet_kpis()
   - Read all 72 KPI values
   - Calculate pillar scores (profit, VELOCITY, SHIELD, EFFICIENCY, CONTINUITY, MARKET)
   - Compute APEX deflection score

2. execute_agents()
   - Dispatch fleet management agents (AI003-AI020)
   - Dispatch trading agents (AI021-AI030)
   - Dispatch security agents (AI025-AI036)
   - Collect agent decisions

3. AI Analysis (via OpenRouter/Groq)
   - Analyze KPI deviations
   - Generate recommendations
   - Predict optimization opportunities

4. Verification Checks (Independent - Part 1)
   - ✅ Core Engine: Newton-Raphson, Pool Dispatcher, Auto Optimizer
   - ✅ Security Layers 1-10: Network, PSP, mTLS, Secrets, Resources
   - ✅ Silicon Integration: 91 agents active, copilot running
   - ✅ Zero Checksum: Must equal 0

5. Copilot Output:
   - executiveSummary: "DEBUG verification complete. All 72 KPIs within tolerance."
   - recommendations: ["Proceed to PREFLIGHT", "All security checks passed"]
   - nextActions: ["Continue to PREFLIGHT mode"]
```

**User Override:** ✅ ALLOWED (with conditions)
- **Condition:** Zero Checksum = 0 + All Part 1 checks passed
- **Override Action:** Commander can skip to PREFLIGHT
- **Copilot Response:** "Override detected. Proceeding to PREFLIGHT with Commander approval."

---

### **Mode 3: PREFLIGHT**

**Copilot Status:** 🟢 ACTIVE (Part 2 - Independent Re-verification)

| Requirement | Status | Details |
|-------------|--------|---------|
| API Keys | ✅ Required | At least 1 AI provider |
| Backend Connection | ✅ Required | Connected |
| Agent Registry | ✅ Required | 91 agents active |
| Copilot Loop | ✅ Running | 5-second interval |
| Fleet KPI Access | ✅ Required | Read 72 KPIs |
| Wallet Access | ✅ Required | Validate profit logic |
| Security Gate | ✅ Required | HSM/YubiKey check |

**Copilot Actions:**
```typescript
// PREFLIGHT mode copilot loop (every 5s):
1. calculate_fleet_kpis()
   - Re-read all 72 KPIs
   - Verify scores match DEBUG baseline

2. execute_agents()
   - Re-verify all 91 agents operational
   - Dispatch security agents (AI031-AI050)
   - Dispatch governance agents (AI041-AI050)

3. AI Analysis (Part 2 - Independent):
   - Re-verify Security Layers 1-10 (dual-confirmation)
   - Validate profit logic (M001 → M057 → M054 → M069)
   - Verify wallet addresses and private key integrity
   - Confirm profit withdrawal mechanisms

4. Security Gate Check:
   - HSM/YubiKey presence
   - Vault encryption status
   - Ethics guardrails active

5. Zero Checksum Verification (Part 2):
   - Full audit vs Sovereign findings
   - Must equal 0

6. Copilot Output:
   - executiveSummary: "PREFLIGHT complete. Security dual-confirmed. System ready for SIMULATION/PILOT."
   - recommendations: ["Proceed to SIMULATION", "All 10 security layers re-verified"]
   - nextActions: ["Run SIMULATION for strategy validation"]
```

**User Override:** ✅ ALLOWED (with conditions)
- **Condition:** Zero Checksum = 0 + All Part 2 checks passed + Security Gate passed
- **Override Actions:**
  - Skip to SIMULATION (normal progression)
  - Skip to PILOT (requires Risk < 0.45)
  - Skip to LIVE (⚠️ NOT recommended, requires Chief Architect + YubiKey)
- **Copilot Response:** "Override detected. Proceeding with Commander authorization."

---

### **Mode 4: SIMULATION**

**Copilot Status:** 🟢 ACTIVE (Shadow-Fork Testing)

| Requirement | Status | Details |
|-------------|--------|---------|
| API Keys | ✅ Required | For AI analysis |
| Backend Connection | ✅ Required | Connected |
| Agent Registry | ✅ Required | 91 agents active |
| Copilot Loop | ✅ Running | 5-second interval |
| Shadow-Fork RPC | ✅ Required | LocalPort port 8547 |
| Strategy Validation | ✅ Required | Backtesting logic |

**Copilot Actions:**
```typescript
// SIMULATION mode copilot loop (every 5s):
1. calculate_fleet_kpis()
   - Monitor shadow-fork execution
   - Track simulated profit

2. execute_agents()
   - Dispatch trading agents (AI021-AI030)
   - Dispatch analysis agents (AI081-AI091)
   - Validate strategy against historical data

3. AI Analysis:
   - Backtesting results analysis
   - Risk assessment (< 0.45 required)
   - Strategy optimization suggestions

4. Validation Checks:
   - Shadow-fork strategy validated
   - Backtesting success rate > 99%
   - Risk < 0.45
   - Deflection ≥ 0
   - Zero Checksum = 0

5. Copilot Output:
   - executiveSummary: "SIMULATION complete. Strategy validated with 99.2% backtest success."
   - recommendations: ["Proceed to PILOT", "Risk score: 0.32 (acceptable)"]
   - nextActions: ["Deploy to PILOT with 100-1000 nodes"]
```

**User Override:** ✅ ALLOWED (with conditions)
- **Condition:** Deflection ≥ 0 + Risk < 0.45 + Zero Checksum = 0
- **Override Actions:**
  - Skip to PILOT (normal progression)
  - Skip to LIVE (⚠️ HIGH RISK, requires Chief Architect + YubiKey + explicit override)
  - Restart SIMULATION with different parameters
- **Copilot Response:** "Override: Proceeding to PILOT with reduced node count for safety."

---

### **Mode 5: PILOT**

**Copilot Status:** 🟢 ACTIVE (Controlled Live Deployment)

| Requirement | Status | Details |
|-------------|--------|---------|
| API Keys | ✅ Required | For AI analysis |
| Backend Connection | ✅ Required | Connected |
| Agent Registry | ✅ Required | 91 agents active |
| Copilot Loop | ✅ Running | 5-second interval |
| Live RPC | ✅ Required | Mainnet RPC active |
| Gasless Transactions | ✅ Required | Pimlico active |
| Node Count | ✅ Required | 1-1000 nodes |

**Copilot Actions:**
```typescript
// PILOT mode copilot loop (every 5s):
1. calculate_fleet_kpis()
   - Monitor live RPC execution
   - Track real profit accumulation
   - Calculate NPM (Net Profit Margin)

2. execute_agents()
   - Dispatch fleet management agents (AI003-AI020)
   - Dispatch trading agents (AI021-AI030)
   - Monitor runner health (AI061-AI070)

3. AI Analysis:
   - Live profit analysis
   - NPM monitoring (target: 1.5x - 3.0x)
   - Runner performance optimization
   - Risk monitoring

4. Validation Checks:
   - Node count: 1-1000
   - Live RPC profitable
   - Gasless transactions active
   - Deflection ≥ 0
   - Zero Checksum = 0

5. Copilot Output:
   - executiveSummary: "PILOT running. 350 nodes active. NPM: 2.8x. Hourly profit: +0.93 ETH."
   - recommendations: ["Increase to 500 nodes", "NPM excellent", "Ready for LIVE"]
   - nextActions: ["Request Chief Architect approval for LIVE mode"]
```

**User Override:** ✅ ALLOWED (with conditions)
- **Condition:** Deflection ≥ 0 + Zero Checksum = 0 + Profit > 0
- **Override Actions:**
  - Adjust node count (1-1000)
  - Restart PILOT with different segments
  - Skip to LIVE (⚠️ REQUIRES Chief Architect + YubiKey)
  - Abort to PREFLIGHT
- **Copilot Response:** "Override: Adjusting node count to 500. Monitoring profit trajectory."

---

### **Mode 6: LIVE**

**Copilot Status:** 🟢 FULLY ACTIVE (Autonomous Production)

| Requirement | Status | Details |
|-------------|--------|---------|
| API Keys | ✅ Required | For continuous AI analysis |
| Backend Connection | ✅ Required | Connected |
| Agent Registry | ✅ Required | 91 agents active |
| Copilot Loop | ✅ Running | 5-second interval |
| Live RPC | ✅ Required | Mainnet RPC |
| Gasless Transactions | ✅ Required | Pimlico active |
| Hardware Wallet | ✅ Required | YubiKey NFC |
| Chief Architect Approval | ✅ Required | Dashboard approval |
| Deflection ≥ 0.8 | ✅ Required | Final threshold |

**Copilot Actions:**
```typescript
// LIVE mode copilot loop (every 5s):
1. calculate_fleet_kpis()
   - Continuous KPI monitoring
   - Real-time profit tracking
   - NPM calculation (target: 2.5x - 3.5x)

2. execute_agents()
   - All 91 agents active
   - Continuous fleet optimization
   - Real-time risk monitoring
   - Auto-adjustment via M054 Auto Optimizer

3. AI Analysis (Continuous):
   - Real-time trade prediction
   - Market regime detection
   - Profit optimization
   - Risk alert generation

4. Continuous Checks:
   - YubiKey heartbeat (every 30s)
   - Profit target: 100 ETH/day
   - NPM floor: 1.5x minimum
   - Deflection: ≥ 0.8 (optimal)

5. Copilot Output:
   - executiveSummary: "LIVE mode active. 10,000 nodes. 92.4 ETH/day. NPM: 3.2x. Velocity: ACCELERATING."
   - recommendations: ["Increase to 15,000 nodes", "NPM optimal", "Consider Base chain expansion"]
   - nextActions: ["Continuous optimization active"]
```

**User Override:** ✅ ALLOWED (with strict conditions)
- **Condition:** Deflection ≥ 0.8 + Chief Architect approval + YubiKey authenticated
- **Override Actions:**
  - Adjust node count (10,000-100,000)
  - Pause execution (pause/resume)
  - Abort to PILOT (ROLLBACK mode)
  - Abort to PREFLIGHT (emergency)
  - Execute emergency stop (kill switch)
- **Copilot Response:** "Override acknowledged. Executing ROLLBACK to PILOT state. Preserving last known good configuration."

---

## 🔐 **USER OVERRIDE PROTOCOL**

### **Override Levels:**

| Mode | Override Level | Permission | Requirements | Copilot Reaction |
|------|---------------|------------|--------------|------------------|
| **CONNECT** | ❌ None | N/A | N/A | Cannot override |
| **DEBUG** | ⚠️ Commander | Standard approval | Zero Checksum = 0 | Allows skip to PREFLIGHT |
| **PREFLIGHT** | ⚠️ Commander | Standard approval | Zero Checksum = 0 | Allows skip to SIMULATION/PILOT |
| **SIMULATION** | ⚠️ Commander | Override allowed | Deflection ≥ 0, Risk < 0.45 | Allows skip to PILOT/LIVE |
| **PILOT** | ⚠️ Commander | Override allowed | Deflection ≥ 0, Profit > 0 | Allows node adjustment, skip to LIVE |
| **LIVE** | 🔴 Chief Architect | **REQUIRES YubiKey** | Deflection ≥ 0.8, Approval | Full control (rollback, pause, kill) |

### **Override Decision Tree:**

```
User requests override
    ↓
Is backend connected?
    ├─ NO → ❌ BLOCKED (show error)
    └─ YES → Continue
        ↓
Is mode running?
    ├─ YES → Check override permission
    │   ├─ CONNECT → ❌ BLOCKED
    │   ├─ DEBUG → ✅ ALLOWED (Commander)
    │   ├─ PREFLIGHT → ✅ ALLOWED (Commander)
    │   ├─ SIMULATION → ✅ ALLOWED (if Risk < 0.45)
    │   ├─ PILOT → ✅ ALLOWED (if profitable)
    │   └─ LIVE → 🔴 REQUIRES Chief Architect + YubiKey
    └─ NO → Continue
        ↓
Are prerequisites met?
    ├─ NO → ❌ BLOCKED (show missing requirements)
    └─ YES → ✅ ALLOWED
        ↓
Copilot validates override safety
    ↓
Is override safe?
    ├─ NO → ⚠️ WARNING (require confirmation)
    └─ YES → ✅ EXECUTE
        ↓
Log override to audit trail
    ↓
Notify Chief Architect (if LIVE mode)
```

### **Override Implementation:**

```typescript
interface OverrideRequest {
  fromMode: string;
  toMode: string;
  userId: string;
  reason: string;
  timestamp: string;
  yubiKeyToken?: string; // Required for LIVE mode
}

interface OverrideValidation {
  canOverride: boolean;
  reason?: string;
  warning?: string;
  requiredApproval: 'COMMANDER' | 'CHIEF_ARCHITECT';
  requiresYubiKey: boolean;
}

const validateOverride = (request: OverrideRequest): OverrideValidation => {
  // 1. Check backend connected
  if (!isConnected) {
    return { canOverride: false, reason: 'Backend not connected' };
  }

  // 2. Check mode running
  if (!runningMode) {
    return { canOverride: false, reason: 'No mode running' };
  }

  // 3. Check prerequisites
  const prerequisites = getPrerequisites(request.fromMode);
  if (!prerequisites.met) {
    return { canOverride: false, reason: `Missing: ${prerequisites.missing}` };
  }

  // 4. Check security criteria
  if (request.fromMode === 'LIVE') {
    return {
      canOverride: true,
      requiredApproval: 'CHIEF_ARCHITECT',
      requiresYubiKey: true,
      warning: 'LIVE mode override requires Chief Architect approval and YubiKey authentication'
    };
  }

  // 5. Copilot safety check
  const safetyCheck = copilot.validateOverride(request);
  if (!safetyCheck.safe) {
    return {
      canOverride: true,
      warning: `Copilot warning: ${safetyCheck.message}`
    };
  }

  return { canOverride: true, requiredApproval: 'COMMANDER' };
};
```

---

## 🧠 **COPILOT DECISION LOOP (5s INTERVAL)**

### **Current Implementation:**

```rust
// backend/main.rs
pub async fn run_copilot_decision_loop(&mut self) {
    let mut interval = tokio::time::interval(Duration::from_secs(5));
    
    loop {
        interval.tick().await;
        
        // 1. Calculate fleet KPIs
        let fleet_kpis = self.calculate_fleet_kpis();
        
        // 2. Execute AI agents
        let agent_results = self.execute_agents(&fleet_kpis);
        
        // 3. AI analysis (OpenRouter/Groq)
        let ai_recommendations = self.ask_ai_auto(&fleet_kpis, &agent_results).await;
        
        // 4. Evaluate mode progression
        let mode_suggestion = self.evaluate_mode_progression(&fleet_kpis);
        
        // 5. Emit copilot advice
        self.broadcast_copilot_advice(ai_recommendations, mode_suggestion).await;
    }
}
```

### **Copilot Output Format:**

```typescript
{
  timestamp: string;
  mode: string;
  loopInterval: 5;
  fleetKPIs: {
    apex: number;
    profit: number;
    velocity: number;
    shield: number;
    efficiency: number;
    continuity: number;
    market: number;
  };
  agentResults: {
    totalAgents: 91;
    activeAgents: number;
    decisions: any[];
  };
  aiAnalysis: {
    provider: 'openrouter' | 'groq' | 'openai';
    model: string;
    executiveSummary: string;
    recommendations: string[];
    nextActions: string[];
    confidence: number; // 0-1
  };
  modeProgression: {
    currentMode: string;
    suggestedNextMode: string;
    canProgress: boolean;
    requirements: string[];
    overrideAvailable: boolean;
  };
  alerts: {
    type: 'INFO' | 'WARNING' | 'CRITICAL';
    message: string;
  }[];
}
```

---

## 📊 **MODE-SPECIFIC COPILOT BEHAVIOR**

### **Summary Table:**

| Mode | Copilot Active | API Keys Required | Agents Active | Override Allowed | Override Permission | Special Requirements |
|------|---------------|-------------------|---------------|------------------|---------------------|----------------------|
| CONNECT | ❌ No | ❌ Not yet | ❌ Not loaded | ❌ No | N/A | Import .env first |
| DEBUG | ✅ Yes | ✅ Yes (1+) | ✅ 91 agents | ✅ Yes | Commander | Zero Checksum = 0 |
| PREFLIGHT | ✅ Yes | ✅ Yes | ✅ 91 agents | ✅ Yes | Commander | Zero Checksum = 0, HSM/YubiKey |
| SIMULATION | ✅ Yes | ✅ Yes | ✅ 91 agents | ✅ Yes | Commander | Risk < 0.45 |
| PILOT | ✅ Yes | ✅ Yes | ✅ 91 agents | ✅ Yes | Commander | Deflection ≥ 0, Profit > 0 |
| LIVE | ✅ Yes | ✅ Yes | ✅ 91 agents | ✅ Yes | Chief Architect | YubiKey, Deflection ≥ 0.8 |

---

## ⚠️ **CURRENT ISSUES IDENTIFIED**

### **1. No Override Permission Enforcement**
```typescript
// Current code: ANY user can override ANY mode
{isCurrentRunning ? "⏹ ABORT" : mode.action}
// onClick={() => executeMode(configuringMode)} // No permission check!
```

**Problem:** No role-based access control (RBAC)
**Impact:** Any user can skip DEBUG/PREFLIGHT and go directly to LIVE
**Recommendation:** Add permission check:
```typescript
const canOverride = (mode: string) => {
  const requiredRole = MODE_OVERRIDE_ROLES[mode];
  return user.role === requiredRole || user.role === 'CHIEF_ARCHITECT';
};
```

### **2. No Copilot Override Validation**
```typescript
// Missing:
if (overrideRequested) {
  const copilotValidation = await copilot.validateOverride(request);
  if (!copilotValidation.safe) {
    showWarning(copilotValidation.message);
  }
}
```

**Problem:** Overrides bypass copilot safety checks
**Impact:** Users can bypass critical safety checks
**Recommendation:** Add copilot validation step

### **3. No Audit Trail for Overrides**
```typescript
// Missing:
await auditLog.log({
  action: 'MODE_OVERRIDE',
  from: runningMode,
  to: requestedMode,
  userId: user.id,
  reason: userReason,
  copilotApproved: copilotValidation.safe
});
```

**Problem:** Cannot trace override history
**Impact:** No accountability for overrides
**Recommendation:** Add comprehensive audit logging


### **5. Copilot Bypassed in Override**
```typescript
// Current behavior:
// User clicks override → Mode executes immediately
// Copilot is NOT consulted

// Expected behavior:
// User clicks override → Copilot validates → Mode executes
```

**Problem:** Override bypasses AI safety checks
**Impact:** Safety guarantees lost
**Recommendation:** Always consult copilot before override

---

## ✅ **RECOMMENDATIONS**

### **1. Implement Role-Based Access Control**
```typescript
interface User {
  id: string;
  role: 'COMMANDER' | 'CHIEF_ARCHITECT' | 'OPERATOR';
  yubiKeyId?: string;
}

const MODE_OVERRIDE_ROLES = {
  'CONNECT': null, // No override
  'DEBUG': 'COMMANDER',
  'PREFLIGHT': 'COMMANDER',
  'SIMULATION': 'COMMANDER',
  'PILOT': 'COMMANDER',
  'LIVE': 'CHIEF_ARCHITECT'
};
```

### **2. Add Copilot Override Validation**
```typescript
const handleOverride = async (request: OverrideRequest) => {
  // 1. Validate permissions
  const permission = validateOverride(request);
  if (!permission.canOverride) {
    throw new Error(permission.reason);
  }

  // 2. Consult copilot
  const copilotValidation = await copilot.validateOverride(request);
  if (!copilotValidation.safe) {
    const confirmed = await confirm(copilotValidation.warning);
    if (!confirmed) return;
  }

  // 3. Execute override
  await executeMode(request.toMode);

  // 4. Log to audit trail
  await auditLog.log({
    action: 'MODE_OVERRIDE',
    ...request,
    copilotApproved: copilotValidation.safe
  });
};
```

### **3. Add Override Confirmation Dialog**
```tsx
<OverrideConfirmationDialog
  visible={showOverrideDialog}
  from={runningMode}
  to={requestedMode}
  copilotWarning={copilotWarning}
  onConfirm={executeOverride}
  onCancel={cancelOverride}
/>
```

### **4. Add Override History**
```typescript
interface OverrideHistory {
  timestamp: string;
  userId: string;
  fromMode: string;
  toMode: string;
  reason: string;
  copilotApproved: boolean;
  result: 'SUCCESS' | 'FAILED';
}

// Display in UI:
<OverrideHistoryTable overrides={overrideHistory} />
```

---

## 🎯 **WHAT IS OVERRIDE?**

### **Definition:**
**Override = User forcefully advancing to the next mode WITHOUT completing all checks**

### **Example Scenario:**
```
NORMAL FLOW:
DEBUG (5-10min) → PREFLIGHT (2-5min) → SIMULATION (30min) → PILOT → LIVE

WITH OVERRIDE:
DEBUG (2min) → 🚀 OVERRIDE → PREFLIGHT (1min) → 🚀 OVERRIDE → LIVE
```

### **Why Override Exists:**
1. **Emergency Situations:** System needs to go LIVE immediately
2. **Development Speed:** Skip redundant checks during testing
3. **Chief Architect Authority:** Bypass checks with proper authorization

### **Current Problem:**
```typescript
// ❌ CURRENT: NO CONTROLS - ANYONE CAN OVERRIDE
if (user.click_override_button()) {
  go_to_next_mode(); // DANGEROUS!
}

// ✅ RECOMMENDED: ROLE-BASED OVERRIDE
if (user.role === 'COMMANDER' && checks_passed) {
  copilot.validate_override();
  log_to_audit_trail();
  go_to_next_mode();
}
```

### **User Question: "Should I add or remove override?"**

**ANSWER: KEEP OVERRIDE, BUT ADD SECURITY CONTROLS**

#### **Why Keep Override?**
- ✅ Documented in Chief Architect Deployment Plan as legitimate feature
- ✅ Commander/Chief Architect need emergency override capability
- ✅ Allows skipping redundant checks in development
- ✅ Required for LIVE mode emergency controls

#### **What to Add:**
1. **RBAC:** Only Commander+ can override
2. **Copilot Validation:** AI checks if override is safe
3. **Audit Trail:** Log all overrides with user ID, timestamp, reason
4. **Confirmation Dialog:** User must confirm override
5. **Override History:** Track all overrides for accountability

#### **What NOT to Add:**
- ❌ Rate limiting (removed as requested)
- ❌ Daily limits
- ❌ Complex approval workflows

---

## ✅ **RECOMMENDATIONS**

### **1. Implement Role-Based Access Control**
```typescript
interface User {
  id: string;
  role: 'COMMANDER' | 'CHIEF_ARCHITECT' | 'OPERATOR';
  yubiKeyId?: string;
}

const MODE_OVERRIDE_ROLES = {
  'CONNECT': null, // No override
  'DEBUG': 'COMMANDER',
  'PREFLIGHT': 'COMMANDER',
  'SIMULATION': 'COMMANDER',
  'PILOT': 'COMMANDER',
  'LIVE': 'CHIEF_ARCHITECT'
};
```

### **2. Add Copilot Override Validation**
```typescript
const handleOverride = async (request: OverrideRequest) => {
  // 1. Validate permissions
  const permission = validateOverride(request);
  if (!permission.canOverride) {
    throw new Error(permission.reason);
  }

  // 2. Consult copilot
  const copilotValidation = await copilot.validateOverride(request);
  if (!copilotValidation.safe) {
    const confirmed = await confirm(copilotValidation.warning);
    if (!confirmed) return;
  }

  // 3. Execute override
  await executeMode(request.toMode);

  // 4. Log to audit trail
  await auditLog.log({
    action: 'MODE_OVERRIDE',
    ...request,
    copilotApproved: copilotValidation.safe
  });
};
```

### **3. Add Override Confirmation Dialog**
```tsx
<OverrideConfirmationDialog
  visible={showOverrideDialog}
  from={runningMode}
  to={requestedMode}
  copilotWarning={copilotWarning}
  onConfirm={executeOverride}
  onCancel={cancelOverride}
/>
```

### **4. Add Override History**
```typescript
interface OverrideHistory {
  timestamp: string;
  userId: string;
  fromMode: string;
  toMode: string;
  reason: string;
  copilotApproved: boolean;
  result: 'SUCCESS' | 'FAILED';
}

// Display in UI:
<OverrideHistoryTable overrides={overrideHistory} />
```

---

## 📊 **SUMMARY**

### **Override Behavior Per Mode:**

| Mode | Can Override? | Who Can Override? | Copilot Validates? | Audit Logged? |
|------|---------------|-------------------|-------------------|---------------|
| CONNECT | ❌ No | N/A | N/A | N/A |
| DEBUG | ✅ Yes | Commander | ✅ Yes | ✅ Yes |
| PREFLIGHT | ✅ Yes | Commander | ✅ Yes | ✅ Yes |
| SIMULATION | ✅ Yes | Commander | ✅ Yes | ✅ Yes |
| PILOT | ✅ Yes | Commander | ✅ Yes | ✅ Yes |
| LIVE | ✅ Yes | Chief Architect + YubiKey | ✅ Yes | ✅ Yes |

### **Final Recommendation:**
**KEEP OVERRIDE** - It's a legitimate feature for authorized users, but add:
1. Role-based access control (RBAC)
2. Copilot validation
3. Audit logging
4. Confirmation dialogs

**DO NOT add rate limiting** - Removed as requested.

**Toggle to Act mode to implement secure override system.**
