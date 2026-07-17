# AllBright Constitutional Governance — Implementation Plan

## Executive Summary

Transform the AllBright AIGUIDE constitutional framework into a **live, monitored, enforceable governance layer** integrated with:
1. The **Module Registry** (`MODULE_REGISTRY.toml`)
2. The **Unified Intelligence System** (91 AISE agents + Copilot loop)
3. The **Reports & Compliance** dashboard page

---

## 1. Constitutional Governance Analysis

### 1.1 Current State

| Component | Location | Status | Gap |
|-----------|----------|--------|-----|
| **AIGUIDE Constitution** | `D:\ALLBRIGHT\AIGUIDE` | Document exists | Not enforced in code |
| **Relationship Matrix** | `backend/relationship_matrix.rs` | Implemented | Not wired to Copilot loop |
| **Module Registry** | `MODULE_REGISTRY.toml` | 119 modules tracked | No runtime API |
| **Governance Agents** | `backend/ai_agents.rs` | 4 stubs (AI047, AI048, AI087, AI089) | Non-functional |
| **Reports Page** | `apps/AIDASH/src/App.tsx` Tab 11 | Static compliance toggles | No governance monitoring |

### 1.2 Constitutional Principles (from AIGUIDE)

| Principle | Enforcement Status |
|-----------|-------------------|
| Single-objective optimization (Profit Growth) | Partial — profit target exists but no constitutional guard |
| No subsystem optimized in isolation | Not enforced |
| Relationship matrix continuously learned | Implemented but not used in decisions |
| All agents orchestrated by Copilot | Partial — agents run but no governance checks |
| Every action logged and auditable | Partial — logs exist but no governance audit trail |
| System must continuously improve | Not measured |

---

## 2. Module Registry Integration

### 2.1 Current Registry Structure

```toml
[meta]
version = "V119"
total_modules = 119
total_kpis = 72

[[module]]
id = "M057"
name = "Pool Dispatcher"
file = "backend/m057_pool_dispatcher.rs"
status = "IMPLEMENTED"
kpis = ["KPI-06", "KPI-07", "KPI-08", "KPI-09", "KPI-10"]
```

### 2.2 Required Enhancements

| Enhancement | Description | Priority |
|-------------|-------------|----------|
| **Runtime Registry API** | Expose `/api/governance/modules` endpoint with live status | P0 |
| **Constitutional Alignment Score** | Each module maps to subsystems/KPIs; score = alignment % | P0 |
| **Module Health Metrics** | Execution success rate, latency, last-run timestamp | P1 |
| **Dependency Graph** | Track module-to-module dependencies for cascade impact analysis | P1 |
| **Version Control** | Track module versions and drift from constitution | P2 |

---

## 3. Unified Intelligence System Integration

### 3.1 Current Agent Registry

```rust
// backend/main.rs:36
pub fn register_agents() -> HashMap<String, Box<dyn Agent>> {
    let mut map = HashMap::new();
    map.insert("AI001".to_string(), Box::new(ai_agents::AI001DesktopAgent::new()));
    // ... 91 agents total
}
```

### 3.2 Governance Agent Gap Analysis

| Agent ID | Current State | Required Function |
|----------|---------------|-------------------|
| **AI047** | Stub — returns `"AI047 proposal: {input}"` | DAO proposal lifecycle management |
| **AI048** | Stub — returns `"AI048 vote: {input}"` | Voting quorum, tally, execution |
| **AI087** | Stub — returns `"AI087 DAO governor: {input}"` | Parameter change governance |
| **AI089** | Duplicate — Inspector (AI089) conflicts with GovernanceStaker | Resolve ID conflict |

### 3.3 New Governance Agents Required

| Agent ID | Name | Function | Integration |
|----------|------|----------|-------------|
| **AI092** | ConstitutionEnforcer | Validates all actions against AIGUIDE laws | Pre-action gate |
| **AI093** | RelationshipMatrixLearner | Updates 6x6 causal matrix from observed data | Post-action learning |
| **AI094** | SubsystemImpactAnalyzer | Evaluates cross-subsystem impact before optimization | Pre-optimization gate |
| **AI095** | AuditLogger | Immutable governance audit trail | All actions |
| **AI096** | KpiAlignmentMonitor | Checks KPI drift from constitutional targets | Continuous monitoring |

---

## 4. Implementation Plan

### Phase 1: Constitutional Governance Engine (Backend)

**File:** `backend/constitution_engine.rs` (NEW)

```rust
pub struct ConstitutionEngine {
    relationship_matrix: Arc<Mutex<RelationshipMatrix>>,
    module_registry: ModuleRegistry,
    audit_trail: Vec<GovernanceEvent>,
    copilot_loop: tokio::time::Interval,
}

impl ConstitutionEngine {
    // Enforce AIGUIDE Part XI — Architectural Laws
    pub async fn validate_action(&self, action: &SystemAction) -> GovernanceResult {
        // Law 1: Profit Growth is the only explicit user-defined objective
        // Law 2: No subsystem shall be optimized in isolation
        // Law 3: Relationship matrix continuously learned
        // Law 4: All agents orchestrated by Copilot
    }

    // Evaluate cross-subsystem impact (AIGUIDE 4.2)
    pub async fn evaluate_enterprise_impact(&self, changes: &[(Subsystem, f64)]) -> HashMap<Subsystem, f64> {
        self.relationship_matrix.evaluate_impact(changes)
    }

    // Log immutable audit trail (AIGUIDE Part VIII)
    pub fn record_governance_event(&mut self, event: GovernanceEvent) {
        self.audit_trail.push(event);
    }
}
```

### Phase 2: Module Registry Service

**File:** `backend/module_registry.rs` (NEW)

```rust
pub struct ModuleRegistry {
    modules: DashMap<String, ModuleEntry>,
    kpi_map: DashMap<String, Vec<String>>, // KPI -> Module IDs
}

pub struct ModuleEntry {
    pub id: String,
    pub name: String,
    pub file: String,
    pub status: ModuleStatus,
    pub kpis: Vec<String>,
    pub subsystems: Vec<Subsystem>,
    pub constitutional_alignment: f64,
    pub last_health_check: Option<DateTime<Utc>>,
}
```

**New API Endpoints:**

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/governance/modules` | GET | All modules with health status |
| `/api/governance/modules/:id` | GET | Single module detail |
| `/api/governance/constitution/validate` | POST | Validate action against constitution |
| `/api/governance/relationship-matrix` | GET | Current 6x6 causal matrix |
| `/api/governance/audit-trail` | GET | Immutable governance events |
| `/api/governance/compliance-score` | GET | Constitutional compliance % |

### Phase 3: Governance Agent Enhancement

**File:** `backend/ai_agents.rs` — Enhance existing stubs

```rust
// AI047 - Proposal Manager (ENHANCE)
pub struct AI047ProposalManager {
    pub enabled: bool,
    pub active_proposals: HashMap<String, GovernanceProposal>,
}

impl Agent for AI047ProposalManager {
    fn execute(&mut self, input: &str) -> Result<String, String> {
        // Parse proposal from input
        // Validate against constitution
        // Queue for voting
        // Return proposal ID and status
    }
}

// AI048 - Vote Manager (ENHANCE)
// AI087 - DAO Governor (ENHANCE)
// AI092-AI096 - NEW governance agents
```

### Phase 4: Frontend Governance Dashboard

**Location:** `apps/AIDASH/src/App.tsx` — Tab 11 (REPORTS & COMPLIANCE)

**New Section: Constitutional Governance Monitor**

```
┌─────────────────────────────────────────────────────────────┐
│  🏛️ CONSTITUTIONAL GOVERNANCE                          v1.0 │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │  72/72 KPIs  │  │ 91/91 Agents │  │ 119 Modules  │    │
│  │  ALIGNED     │  │ ORCHESTRATED │  │  REGISTERED  │    │
│  │   ████████   │  │   ████████   │  │   ████████   │    │
│  └──────────────┘  └──────────────┘  └──────────────┘    │
│                                                             │
│  SUBSYSTEM HEALTH MATRIX                                   │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ Profit      [██████████] 96.4%  │ Growth    [███████] 85.2% │
│  │ Velocity    [█████████]  91.1% │ Efficiency[████████] 92.8% │
│  │ Security    [██████████] 98.7% │ Quality   [███████] 88.3%  │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  6x6 RELATIONSHIP MATRIX (Live)                            │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  Visual heatmap of causal relationships             │   │
│  │  Reinforcing (green) / Balancing (yellow)            │   │
│  │  Constraining (red)                                  │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  GOVERNANCE AUDIT TRAIL                                    │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ [2026-07-07 10:15] Action: ProfitTarget +0.5%       │   │
│  │   Impact: +Profit +2.1%, -Velocity -0.3%, OK        │   │
│  │ [2026-07-07 10:12] Action: RiskMode → aggressive    │   │
│  │   Impact: +Velocity +5.2%, +Shield +1.1%, OK        │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  CONSTITUTIONAL COMPLIANCE                                 │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ Law 1: Single Objective ✓                           │   │
│  │ Law 2: No Isolated Optimization ✓                   │   │
│  │ Law 3: Continuous Learning ✓                        │   │
│  │ Law 4: Copilot Orchestration ✓                      │   │
│  │ Law 5: Full Auditability ✓                          │   │
│  │ Law 6: Continuous Improvement ✓                     │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### Phase 5: Backend-Frontend Integration

**New API Calls from Frontend:**

```typescript
// apps/AIDASH/src/lib/constitutionApi.ts

export const constitutionApi = {
  // Get constitutional compliance score
  getComplianceScore: async () => {
    const res = await fetch(`${API_BASE}/api/governance/compliance-score`);
    return res.json();
  },

  // Get subsystem health
  getSubsystemHealth: async () => {
    const res = await fetch(`${API_BASE}/api/governance/subsystems`);
    return res.json();
  },

  // Get relationship matrix
  getRelationshipMatrix: async () => {
    const res = await fetch(`${API_BASE}/api/governance/relationship-matrix`);
    return res.json();
  },

  // Get governance audit trail
  getAuditTrail: async (limit = 50) => {
    const res = await fetch(`${API_BASE}/api/governance/audit-trail?limit=${limit}`);
    return res.json();
  },

  // Validate action against constitution
  validateAction: async (action: SystemAction) => {
    const res = await fetch(`${API_BASE}/api/governance/constitution/validate`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(action),
    });
    return res.json();
  },

  // Get module registry status
  getModuleRegistry: async () => {
    const res = await fetch(`${API_BASE}/api/governance/modules`);
    return res.json();
  },
};
```

---

## 5. Detailed Task Breakdown

### Backend Tasks

| Task | File | Priority | Est. Effort |
|------|------|----------|-------------|
| Create `constitution_engine.rs` | `backend/constitution_engine.rs` | P0 | 4h |
| Create `module_registry.rs` runtime service | `backend/module_registry.rs` | P0 | 3h |
| Add 6 new governance API endpoints | `backend/main.rs` | P0 | 2h |
| Enhance AI047, AI048, AI087, AI089 | `backend/ai_agents.rs` | P0 | 3h |
| Create AI092-AI096 governance agents | `backend/ai_agents.rs` | P0 | 4h |
| Wire constitution engine to Copilot loop | `backend/main.rs` | P0 | 2h |
| Add governance event logging | `backend/constitution_engine.rs` | P1 | 2h |
| Add module health check task | `backend/module_registry.rs` | P1 | 2h |
| Expose relationship matrix via API | `backend/main.rs` | P1 | 1h |
| **Total Backend** | | | **23h** |

### Frontend Tasks

| Task | File | Priority | Est. Effort |
|------|------|----------|-------------|
| Create `constitutionApi.ts` | `apps/AIDASH/src/lib/constitutionApi.ts` | P0 | 1h |
| Create `GovernanceDashboard.tsx` | `apps/AIDASH/src/components/GovernanceDashboard.tsx` | P0 | 6h |
| Create `SubsystemHealthMatrix.tsx` | `apps/AIDASH/src/components/SubsystemHealthMatrix.tsx` | P0 | 3h |
| Create `RelationshipMatrixHeatmap.tsx` | `apps/AIDASH/src/components/RelationshipMatrixHeatmap.tsx` | P1 | 4h |
| Create `GovernanceAuditTrail.tsx` | `apps/AIDASH/src/components/GovernanceAuditTrail.tsx` | P1 | 2h |
| Create `ModuleRegistryTable.tsx` | `apps/AIDASH/src/components/ModuleRegistryTable.tsx` | P1 | 3h |
| Integrate into Reports & Compliance tab | `apps/AIDASH/src/App.tsx` | P0 | 1h |
| Add constitutional compliance scoring | `apps/AIDASH/src/components/GovernanceDashboard.tsx` | P0 | 2h |
| **Total Frontend** | | | **22h** |

### Integration Tasks

| Task | Description | Priority | Est. Effort |
|------|-------------|----------|-------------|
| Wire governance to Tauri IPC | Add governance commands to `main.rs` | P1 | 2h |
| Add governance to CopilotPanel | Copilot can explain constitutional decisions | P1 | 2h |
| Add governance events to OperationsCenter | Live feed of governance actions | P2 | 2h |
| **Total Integration** | | | **6h** |

**Grand Total: ~51 hours**

---

## 6. Integration Points

### 6.1 Module Registry → Constitution Engine

```
ModuleRegistry::register_module(id, entry)
  → ConstitutionEngine::validate_constitutional_alignment(entry)
    → Returns alignment_score (0.0-1.0)
    → Stored in ModuleEntry.constitutional_alignment
```

### 6.2 Copilot Loop → Governance Agents

```
run_copilot_decision_loop() [every 5s]
  → execute_agents() [91 agents including AI092-AI096]
    → AI092: Validate all pending actions against constitution
    → AI093: Update relationship matrix from observed outcomes
    → AI094: Analyze subsystem impact before optimization
    → AI095: Record immutable audit event
    → AI096: Check KPI alignment drift
```

### 6.3 Frontend → Backend Governance API

```
GovernanceDashboard.tsx
  → constitutionApi.getComplianceScore()
  → constitutionApi.getSubsystemHealth()
  → constitutionApi.getRelationshipMatrix()
  → constitutionApi.getAuditTrail()
  → constitutionApi.getModuleRegistry()
```

### 6.4 Reports & Compliance Page Integration

```
Tab 11: REPORTS & COMPLIANCE
  ├── Section 1: Regulatory Alignment Audit (EXISTING)
  ├── Section 2: Constitutional Governance Monitor (NEW)
  │   ├── Compliance Score
  │   ├── Subsystem Health Matrix
  │   ├── Relationship Matrix Heatmap
  │   ├── Governance Audit Trail
  │   └── Module Registry Status
  └── Section 3: Audit Report Export (ENHANCED)
```

---

## 7. Success Criteria

| Metric | Target |
|--------|--------|
| Constitutional compliance score | ≥ 98% at all times |
| Module registry API availability | 100% uptime |
| Governance agent execution | Every 5s Copilot tick |
| Audit trail completeness | 100% of actions logged |
| Relationship matrix freshness | Updated within 1 Copilot cycle |
| Frontend governance dashboard load time | < 2s |
| Cross-subsystem impact analysis latency | < 100ms |

---

## 8. Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Governance checks slow down Copilot loop | Run governance in parallel tokio task |
| Relationship matrix drift | Confidence-weighted updates, minimum sample size |
| Module registry desync | Health check every 30s, auto-recovery |
| Frontend performance | Virtualized tables, paginated audit trail |
| Constitutional override by user | Require multi-sig for constitution changes |

---

## 9. Implementation Order

1. **Week 1:** Backend Constitution Engine + Module Registry Service
2. **Week 2:** Governance Agent Enhancement + API Endpoints
3. **Week 3:** Frontend Governance Dashboard + Integration
4. **Week 4:** Testing, Audit Trail verification, Deployment

---

## 10. Files to Create/Modify

### New Files
- `backend/constitution_engine.rs`
- `backend/module_registry.rs`
- `backend/governance_types.rs`
- `apps/AIDASH/src/lib/constitutionApi.ts`
- `apps/AIDASH/src/components/GovernanceDashboard.tsx`
- `apps/AIDASH/src/components/SubsystemHealthMatrix.tsx`
- `apps/AIDASH/src/components/RelationshipMatrixHeatmap.tsx`
- `apps/AIDASH/src/components/GovernanceAuditTrail.tsx`
- `apps/AIDASH/src/components/ModuleRegistryTable.tsx`

### Modified Files
- `backend/main.rs` — Add governance endpoints, wire to Copilot loop
- `backend/ai_agents.rs` — Enhance AI047, AI048, AI087, add AI092-AI096
- `apps/AIDASH/src/App.tsx` — Add governance section to Tab 11
- `MODULE_REGISTRY.toml` — Add constitutional_alignment field to each module
