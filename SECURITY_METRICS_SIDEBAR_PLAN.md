# Security System Metrics Sidebar - Implementation Plan

## Part 1: Auto-Optimization Primary Drivers Analysis

### ✅ Current Optimization Logic Status: **CORRECT**

The system correctly optimizes the primary drivers of each KPI through dimension mapping:

```
For each KPI deviation → compute adjustment factor → apply to dimension
```

### KPI-to-Primary Driver Mapping (Verified Correct):

| Pillar | Primary Driver Dimension | Adjustment Formula |
|--------|-------------------------|-------------------|
| ALPHA (KPIs 1-12) | Corridor (d0), Bribe (d1), Bundle (d3), FlashLoan (d4) | `1.0 + deviation * multiplier` |
| VELOCITY (KPIs 13-24) | Block Phase (d2), Solver (d24), JIT (d23) | `1.0 + deviation * 0.15-0.25` |
| SHIELD (KPIs 25-36) | Shield Routing (d8), Pool Tier (d16), Regional (d6) | `1.0 + deviation * 0.2-0.3` |
| EFFICIENCY (KPIs 37-48) | Capital (d10), Multi-Hop (d11), Gas (d21) | `1.0 + deviation * 0.15-0.25` |
| CONTINUITY (KPIs 49-60) | Runner Cap (d22), Chain Sel (d20) | `1.0 + deviation * 0.2-0.25` |
| MARKET (KPIs 61-72) | Pair Sel (d2), Region (d1), Mode (d3) | `1.0 + deviation * 0.1-0.2` |

### Optimization Flow:
1. Each KPI deviation triggers corresponding dimension adjustment
2. Adjustment factor scales with deviation magnitude
3. Multiple KPIs can drive same dimension (cross-pillar coordination)

---

## Part 2: Security System Metrics Sidebar Implementation

### 10-Layer Security System (from SECURITY_ZK_PROOF_PROPOSAL.md):

| Layer # | Security Layer | Module | Method | Status |
|---------|----------------|--------|--------|--------|
| 1 | Stealth Network | security_gate.rs | WireGuard + Registry | Active |
| 2 | HSM/YubiKey Auth | security_gate.rs | Physical key | Active |
| 3 | Vault AES-256 | m055_env_vault.rs | AES-256-GCM | Active |
| 4 | Memory Protection | security_gate.rs | Guard pages + mlock | Active |
| 5 | Installer Signature | security_gate.rs | Code signing | Active |
| 6 | Windows DEP/ASLR | security_gate.rs | CFG + Stack cookies | Active |
| 7 | ZK Proof | m099_zk_proof.rs | Groth16 + Pedersen | Proposed |
| 8 | RBAC | security_gate.rs | Role-Based Access | To Add |
| 9 | Input Validation | security_gate.rs | Sanitize + Validate | To Add |
| 10 | Encrypted Transit | security_gate.rs | TLS 1.3 + mTLS | To Add |

---

## Implementation Plan for Security Metrics Sidebar

### Step 1: Create Security Layer Types (api.ts)
```typescript
export interface SecurityLayerMetrics {
  layerNumber: number;
  layerName: string;
  module: string;
  method: string;
  measuredValue: number;  // 0-100 score
  targetValue: number;
  status: 'ACTIVE' | 'PASSED' | 'FAILED' | 'PENDING';
  lastCheck: string;      // ISO timestamp
  probability: string;    // Security strength (1-in-X)
}
```

### Step 2: Create SecurityMetricsSidebar.tsx Component
Features:
- Sortable table by: Layer #, Security Score, Status
- Color-coded status indicators (emerald/green/red)
- Measured values vs targets displayed
- Refresh/Validate button to trigger checks

### Step 3: Add API Endpoint (backend/main.rs)
```rust
// /api/security/layers/metrics
async fn get_security_layer_metrics() -> Result<Json<serde_json::Value>, AppError> {
    // Return all 10 layers with measured values
}
```

### Step 4: Add Layer Measurements to security_gate.rs
```rust
// Add get_security_metrics() returning all layer statuses
// Each layer computes its own security score (0-100)
```

### Step 5: Add Sidebar Route in App.tsx
```tsx
<SidebarSection title="Security System" icon={Shield}>
  <SecurityMetricsSidebar />
</SidebarSection>
```

---

## File Structure:
```
apps/dashboard/src/components/SecurityMetricsSidebar.tsx  (NEW)
apps/dashboard/src/lib/api.ts  (ADD interfaces + endpoint)
backend/security_gate.rs  (ADD get_security_metrics())
backend/main.rs  (ADD /api/security/layers/metrics route)
```

---

## Mock Data for Development:
```json
{
  "layers": [
    {"layerNumber": 1, "layerName": "Stealth Network", "measuredValue": 99.5, "targetValue": 100, "status": "PASSED"},
    {"layerNumber": 2, "layerName": "HSM/YubiKey", "measuredValue": 95.0, "targetValue": 100, "status": "ACTIVE"},
    ...
  ]
}
```

---

**Requires your approval before implementation.**