# AllBright Naming Convention Correction

**Date:** 2026-07-13  
**Status:** CRITICAL CORRECTION — ALL REPORTS NEED UPDATES  
**Issue:** Incorrect naming convention used across all audit reports

---

## Executive Summary

A critical naming convention error was identified in all audit reports. The correct naming convention is:

- **Modules:** M001, M002, M003, ... (sequential numeric IDs)
- **AI Agents:** AI001, AI002, AI003, ... (sequential numeric IDs)

**Incorrect naming used in reports:**
- Functional agents: TradingAI, SecurityAI, FlashLoanAI, etc. ❌
- Supervisor agents: AI097, AI098, etc. ❌ (should start at AI001)

**Correct naming:**
- All agents: AI001, AI002, AI003, ... AI0XX
- All modules: M001, M002, M003, ... M0XX

---

## Correct Naming Convention

### Module IDs (M001-M999)
```
M001 - Wallet Management
M002 - Transaction Batcher
M003 - Gas Oracle
M004 - MEV Protection
M005 - State Synchronizer
M006 - Central C2 Server
M007 - Latency Tracking
M008 - Portfolio Rebalancer
M009 - Yield Aggregator
M010 - Risk Calculator
...
```

### AI Agent IDs (AI001-AI999)
```
AI001 - Core Trading Agent (formerly TradingAI)
AI002 - Arbitrage Detection Agent
AI003 - Trade Execution Agent
AI004 - Flash Loan Agent
AI005 - Security Agent (formerly SecurityAI)
AI006 - Governance Agent (formerly GovernanceAI)
AI007 - Audit Agent (formerly AuditAI)
AI008 - Telemetry Agent (formerly TelemetryAI)
AI009 - Deployment Agent (formerly DeploymentAI)
AI010 - Learning Agent (formerly LearningAI)
...
```

---

## Required Corrections

### Reports Requiring Updates

| Report | Errors | Priority |
|--------|--------|----------|
| FILE_MODULE_MAPPING_REPORT.md | 100+ references | P0-Critical |
| AI_AGENT_REGISTRY_AUDIT_REPORT.md | 200+ references | P0-Critical |
| MODULE_REGISTRY_AUDIT_REPORT.md | 50+ references | P0-Critical |
| GOVERNANCE_IMPLEMENTATION_AUDIT_REPORT.md | 30+ references | P1-High |
| REFLECTION_ENGINE_AUDIT_REPORT.md | 20+ references | P1-High |
| GOVERNANCE_GATEKEEPER_AUDIT_REPORT.md | 15+ references | P1-High |

### Correction Examples

#### Example 1: FILE_MODULE_MAPPING_REPORT.md

**Incorrect:**
```
| `backend/m025_trade_executor.rs` | `backend::modules` | Trade execution logic | flashbots_mev_protection.rs | TradingAI | P0-Critical |
```

**Correct:**
```
| `backend/m025_trade_executor.rs` | `backend::modules` | Trade execution logic | flashbots_mev_protection.rs | AI001 | P0-Critical |
```

#### Example 2: AI_AGENT_REGISTRY_AUDIT_REPORT.md

**Incorrect:**
```
| **TradingAI** | 5 | Trade execution, arbitrage detection, order routing |
```

**Correct:**
```
| **AI001** | 5 | Trade execution, arbitrage detection, order routing |
```

#### Example 3: MODULE_REGISTRY_AUDIT_REPORT.md

**Incorrect:**
```
The registry claims 27 STUB modules, but they are all marked IMPLEMENTED in the registry.
```

**Correct:**
```
The registry claims 27 STUB modules (M001-M027), but they are all marked IMPLEMENTED in the registry.
```

---

## Agent Name Mapping (Old → New)

| Old Name | New ID | Rationale |
|----------|--------|-----------|
| AISE | AI001 | Primary AI system |
| TradingAI | AI002 | Core trading functions |
| FlashLoanAI | AI003 | Flash loan execution |
| SecurityAI | AI004 | Security operations |
| GovernanceAI | AI005 | Governance engine |
| AuditAI | AI006 | Audit functions |
| TelemetryAI | AI007 | Metrics and alerts |
| DeploymentAI | AI008 | CI/CD and deployment |
| LearningAI | AI009 | Machine learning |
| OptimizationAI | AI010 | Optimization |
| RoutingAI | AI011 | Order routing |
| PricingAI | AI012 | Slippage and gas |
| InfrastructureAI | AI013 | Infrastructure |
| ReliabilityAI | AI014 | Backup and DR |
| FrontendAI | AI015 | Dashboard UI |
| CopilotAI | AI016 | Copilot features |
| SimulationAI | AI017 | Simulation |
| SolidityBot | AI018 | Smart contracts |
| CommanderAI | AI019 | Commander oversight |
| CoreAI | AI020 | Core system |

**Supervisor Agents (existing in MODULE_REGISTRY.toml):**
| Old ID | New ID | Notes |
|--------|--------|-------|
| AI097 | AI021 | Supervisor Core |
| AI098 | AI022 | Supervisor Trading |
| AI099 | AI023 | Supervisor Security |
| AI100 | AI024 | Supervisor Infrastructure |
| AI101 | AI025 | Supervisor Profit |
| AI102 | AI026 | Supervisor Growth |
| AI103 | AI027 | Supervisor Velocity |
| AI104 | AI028 | Supervisor Efficiency |
| AI105 | AI029 | Supervisor Security Sub |
| AI106 | AI030 | Supervisor Quality |
| AI107 | AI031 | Copilot Auditor |

---

## Immediate Actions Required

1. **Update all 6 audit reports** with correct naming convention
2. **Update MODULE_REGISTRY.toml** to reflect proper naming
3. **Notify team** of naming convention change
4. **Update documentation** to use M001-M999 and AI001-AI999 format

---

## Naming Convention Rules

### Module Naming (M###)
- Format: `M` + 3-digit number (001-999)
- Example: `M001`, `M025`, `M137`
- Reserved ranges:
  - M001-M099: Core trading and execution
  - M100-M199: Security and governance
  - M200-M299: Learning and optimization
  - M300-M399: Infrastructure and deployment
  - M400-M499: Frontend and UI
  - M500-M599: External integrations
  - M600-M699: Testing and simulation
  - M700-M799: Data and analytics
  - M800-M899: Compliance and audit
  - M900-M999: Experimental/reserved

### AI Agent Naming (AI###)
- Format: `AI` + 3-digit number (001-999)
- Example: `AI001`, `AI019`, `AI031`
- Reserved ranges:
  - AI001-AI020: Functional agents
  - AI021-AI040: Supervisor agents
  - AI041-AI060: Specialist agents
  - AI061-AI080: Domain agents
  - AI081-AI100: Reserved for future

---

## Validation Checklist

- [ ] All module references updated to M001-M999 format
- [ ] All AI agent references updated to AI001-AI999 format
- [ ] MODULE_REGISTRY.toml corrected
- [ ] AI_AGENT_REGISTRY.toml created with correct naming
- [ ] All 6 audit reports corrected
- [ ] Team notified of naming convention
- [ ] Documentation updated

---

*Correction notice generated by AllBright Governance Auditor.*