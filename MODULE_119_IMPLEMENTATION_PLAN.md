# AllBright 119-Module Implementation Plan
## Path to 119 Modules from Current 76

**Current State:** 76 modules registered in `MODULE_REGISTRY.toml`  
**Target State:** 119 modules  
**Gap:** 43 missing modules  
**Strategy:** Implement in 4 phases by functional priority

---

## Phase 1: Critical Trading & Security Core (6 modules)

| Module ID | Name | CGM Subsystem | File | Priority |
|-----------|------|---------------|------|----------|
| M003 | Transaction Batcher | Velocity | `backend/m003_transaction_batcher.rs` | P0 |
| M007 | Gas Price Oracle | Efficiency | `backend/m007_gas_oracle.rs` | P0 |
| M008 | MEV Protection Engine | Security | `backend/m008_mev_protection.rs` | P0 |
| M010 | Portfolio Rebalancer | Profit | `backend/m010_portfolio_rebalancer.rs` | P0 |
| M011 | Yield Aggregator | Growth | `backend/m011_yield_aggregator.rs` | P0 |
| M012 | Risk Calculator | Security | `backend/m012_risk_calculator.rs` | P0 |

**Rationale:** These are the most critical missing modules for live trading. M003, M007, M008 are referenced in existing code but have no implementation. M010-M012 are core financial logic.

---

## Phase 2: Trading Operations & Analytics (12 modules)

| Module ID | Name | CGM Subsystem | File | Priority |
|-----------|------|---------------|------|----------|
| M013 | Compliance Checker | Security | `backend/m013_compliance_checker.rs` | P1 |
| M014 | Audit Logger | Quality | `backend/m014_audit_logger.rs` | P1 |
| M015 | Performance Reporter | Quality | `backend/m015_performance_reporter.rs` | P1 |
| M022 | Arbitrage Detector | Profit | `backend/m022_arbitrage_detector.rs` | P1 |
| M023 | Liquidity Analyzer | Velocity | `backend/m023_liquidity_analyzer.rs` | P1 |
| M024 | Price Monitor | Profit | `backend/m024_price_monitor.rs` | P1 |
| M025 | Trade Executor | Velocity | `backend/m025_trade_executor.rs` | P1 |
| M026 | Order Router | Efficiency | `backend/m026_order_router.rs` | P1 |
| M027 | Slippage Calculator | Efficiency | `backend/m027_slippage_calculator.rs` | P1 |
| M028 | Fraud Detector | Security | `backend/m028_fraud_detector.rs` | P1 |
| M029 | Access Controller | Security | `backend/m029_access_controller.rs` | P1 |
| M030 | Encryption Manager | Security | `backend/m030_encryption_manager.rs` | P1 |

**Rationale:** These complete the trading pipeline from detection to execution to reporting.

---

## Phase 3: Infrastructure & Governance (15 modules)

| Module ID | Name | CGM Subsystem | File | Priority |
|-----------|------|---------------|------|----------|
| M031 | Key Rotator | Security | `backend/m031_key_rotator.rs` | P2 |
| M032 | Certificate Manager | Security | `backend/m032_certificate_manager.rs` | P2 |
| M033 | Audit Trail | Quality | `backend/m033_audit_trail.rs` | P2 |
| M034 | Anomaly Detector | Security | `backend/m034_anomaly_detector.rs` | P2 |
| M035 | Threat Monitor | Security | `backend/m035_threat_monitor.rs` | P2 |
| M036 | Incident Responder | Security | `backend/m036_incident_responder.rs` | P2 |
| M037 | Backup Manager | Continuity | `backend/m037_backup_manager.rs` | P2 |
| M038 | Container Manager | Efficiency | `backend/m038_container_manager.rs` | P2 |
| M039 | Load Balancer | Efficiency | `backend/m039_load_balancer.rs` | P2 |
| M040 | Service Mesh | Efficiency | `backend/m040_service_mesh.rs` | P2 |
| M042 | Configuration Manager | Quality | `backend/m042_config_manager.rs` | P2 |
| M043 | Secret Manager | Security | `backend/m043_secret_manager.rs` | P2 |
| M045 | Health Checker | Continuity | `backend/m045_health_checker.rs` | P2 |
| M046 | Metrics Collector | Efficiency | `backend/m046_metrics_collector.rs` | P2 |
| M047 | Log Aggregator | Quality | `backend/m047_log_aggregator.rs` | P2 |

**Rationale:** Infrastructure resilience and operational excellence.

---

## Phase 4: Advanced Intelligence & Governance (10 modules)

| Module ID | Name | CGM Subsystem | File | Priority |
|-----------|------|---------------|------|----------|
| M048 | Alert Dispatcher | Security | `backend/m048_alert_dispatcher.rs` | P3 |
| M049 | Incident Tracker | Quality | `backend/m049_incident_tracker.rs` | P3 |
| M050 | Governance Engine | All | `backend/m050_governance_engine.rs` | P3 |
| M056 | Learning Engine | Growth | `backend/m056_learning_engine.rs` | P3 |
| M060 | Model Trainer | Growth | `backend/m060_model_trainer.rs` | P3 |
| M064 | Data Pipeline | Efficiency | `backend/m064_data_pipeline.rs` | P3 |
| M065 | Feature Store | Efficiency | `backend/m065_feature_store.rs` | P3 |
| M078 | Governance Auditor | Quality | `backend/m078_governance_auditor.rs` | P3 |
| M079 | Constitutional Enforcer | All | `backend/m079_constitutional_enforcer.rs` | P3 |
| M080 | Compliance Reporter | Quality | `backend/m080_compliance_reporter.rs` | P3 |

**Rationale:** Advanced governance and continuous learning capabilities.

---

## Implementation Pattern for Each Module

Each module follows this pattern to ensure consistency and CGM alignment:

```rust
// backend/m0xx_module_name.rs
pub struct M0XXModuleName {
    pub enabled: bool,
    pub metrics: ModuleMetrics,
}

pub struct ModuleMetrics {
    pub executions: u64,
    pub successes: u64,
    pub failures: u64,
    pub last_execution: Option<String>,
}

impl M0XXModuleName {
    pub fn new() -> Self { ... }
    pub fn execute(&mut self) -> Result<ModuleResult, ModuleError> { ... }
    pub fn get_health(&self) -> ModuleHealth { ... }
}
```

**Integration steps for each module:**
1. Create Rust source file in `backend/`
2. Add `mod m0xx_module_name;` to `backend/main.rs`
3. Register in `HotSwapRegistry` in `CentralC2Server::register_core_modules()`
4. Add entry to `MODULE_REGISTRY.toml`
5. Wire into Copilot loop if applicable

---

## Success Criteria

| Metric | Target |
|--------|--------|
| Total modules in registry | 119 |
| Modules with Rust implementation | ≥ 100 |
| Modules registered in HotSwapRegistry | 119 |
| CGM compliance score | ≥ 95% |
| Backend compilation | Clean build |

---

## Timeline

| Phase | Modules | Effort | Target Completion |
|-------|---------|--------|-------------------|
| Phase 1 | 6 | 2 days | Day 2 |
| Phase 2 | 12 | 3 days | Day 5 |
| Phase 3 | 15 | 4 days | Day 9 |
| Phase 4 | 10 | 3 days | Day 12 |
| Integration | All | 2 days | Day 14 |
| **Total** | **43** | **14 days** | **Day 14** |
