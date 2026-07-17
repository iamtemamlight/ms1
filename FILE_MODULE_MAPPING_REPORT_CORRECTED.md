# AllBright File Module Mapping Report (CORRECTED)

**Date:** 2026-07-13  
**Status:** CORRECTED — NAMING CONVENTION FIXED  
**Correction:** Updated to use M001-M999 and AI001-AI999 format per NAMING_CONVENTION_CORRECTION.md

---

## Executive Summary

This report maps all important files in the AllBright project to their parent modules, functions, dependencies, assigned AI agents, and governance classifications. **NOTE: AI agent IDs corrected to AI001-AI999 format.**

---

## 1. File-to-Module Mapping

### 1.1 Backend Core Modules (Rust)

| File Path | Parent Module | Primary Function | Dependencies | AI Agent | Governance Classification |
|-----------|---------------|------------------|--------------|----------|---------------------------|
| `backend/main.rs` | `backend` | Application entry point | All modules | AI019 (CommanderAI) | P0-Critical |
| `backend/server.js` | `backend` | Node.js server (if used) | Express, Prisma | -- | P1-High |
| `backend/trading_engine.rs` | `backend` | Core trading orchestration | M025, M022, M027 | AI001 (AISE) | P0-Critical |
| `backend/ai_agents.rs` | `backend::ai` | AI agent management | ai/manager.rs | AI001 (AISE) | P0-Critical |
| `backend/ai/manager.rs` | `backend::ai` | AI lifecycle manager | ai_agents.rs | AI001 (AISE) | P0-Critical |
| `backend/aise_unified_intelligence.rs` | `backend::aise` | Unified AI system | ai_agents.rs | AI001 (AISE) | P0-Critical |
| `backend/security_gate.rs` | `backend::security` | Access control & auth | m029_access_controller.rs | AI004 (SecurityAI) | P0-Critical |
| `backend/m025_trade_executor.rs` | `backend::modules` | Trade execution logic | flashbots_mev_protection.rs | AI002 (TradingAI) | P0-Critical |
| `backend/m022_arbitrage_detector.rs` | `backend::modules` | Arbitrage opportunity detection | m023_liquidity_analyzer.rs | AI002 (TradingAI) | P0-Critical |
| `backend/m137_flash_loan_executor.rs` | `backend::modules` | Flash loan executor | FlashLoanArbitrage.sol | AI003 (FlashLoanAI) | P0-Critical |
| `backend/flashbots_mev_protection.rs` | `backend::modules` | MEV protection | m025_trade_executor.rs | AI004 (SecurityAI) | P0-Critical |
| `backend/contracts/aave.rs` | `backend::contracts` | Aave protocol bindings | Alloy | AI018 (SolidityBot) | P1-High |
| `backend/contracts/uniswap.rs` | `backend::contracts` | Uniswap protocol bindings | Alloy | AI018 (SolidityBot) | P1-High |
| `backend/contracts/balancer.rs` | `backend::contracts` | Balancer protocol bindings | Alloy | AI018 (SolidityBot) | P1-High |
| `backend/contracts/dydx.rs` | `backend::contracts` | dYdX protocol bindings | Alloy | AI018 (SolidityBot) | P1-High |
| `backend/m050_governance_engine.rs` | `backend::governance` | Governance enforcement | m079_constitutional_enforcer.rs | AI005 (GovernanceAI) | P0-Critical |
| `backend/m078_governance_auditor.rs` | `backend::governance` | Governance auditing | m050_governance_engine.rs | AI005 (GovernanceAI) | P0-Critical |
| `backend/m079_constitutional_enforcer.rs` | `backend::governance` | Constitutional rules | m050_governance_engine.rs | AI005 (GovernanceAI) | P0-Critical |
| `backend/m080_compliance_reporter.rs` | `backend::governance` | Compliance reporting | m050_governance_engine.rs | AI005 (GovernanceAI) | P1-High |
| `backend/m014_audit_logger.rs` | `backend::audit` | Audit trail logging | Postgres/Prisma | AI006 (AuditAI) | P0-Critical |
| `backend/m033_audit_trail.rs` | `backend::audit` | Audit trail queries | m014_audit_logger.rs | AI006 (AuditAI) | P1-High |
| `backend/m135_data_quality.rs` | `backend::audit` | Data quality checks | m014_audit_logger.rs | AI006 (AuditAI) | P1-High |
| `backend/m132_copilot_auditor.rs` | `backend::audit` | Copilot audit | m014_audit_logger.rs | AI016 (CopilotAI) | P1-High |
| `backend/m133_sovereign_audit.rs` | `backend::audit` | Sovereign audit | m014_audit_logger.rs | AI016 (CopilotAI) | P1-High |
| `backend/m134_commander_audit.rs` | `backend::audit` | Commander audit | m014_audit_logger.rs | AI016 (CopilotAI) | P1-High |
| `backend/kpi_telemetry.rs` | `backend::telemetry` | KPI metrics collection | Prometheus | AI007 (TelemetryAI) | P1-High |
| `backend/m083_metrics.rs` | `backend::telemetry` | Metrics aggregation | kpi_telemetry.rs | AI007 (TelemetryAI) | P1-High |
| `backend/m046_metrics_collector.rs` | `backend::telemetry` | Metrics collection | kpi_telemetry.rs | AI007 (TelemetryAI) | P1-High |
| `backend/m084_alerts.rs` | `backend::telemetry` | Alerting system | kpi_telemetry.rs | AI007 (TelemetryAI) | P1-High |
| `backend/m048_alert_dispatcher.rs` | `backend::telemetry` | Alert dispatching | m084_alerts.rs | AI007 (TelemetryAI) | P1-High |
| `backend/m049_incident_tracker.rs` | `backend::telemetry` | Incident tracking | m084_alerts.rs | AI007 (TelemetryAI) | P1-High |
| `backend/hot_swap_module.rs` | `backend::deployment` | Hot module swapping | m140_builder_monitor.rs | AI008 (DeploymentAI) | P1-High |
| `backend/deployment.rs` | `backend::deployment` | Deployment logic | hot_swap_module.rs | AI008 (DeploymentAI) | P1-High |
| `backend/build_guard.rs` | `backend::deployment` | Build validation | deployment.rs | AI008 (DeploymentAI) | P2-Medium |
| `backend/m140_builder_monitor.rs` | `backend::deployment` | Build monitoring | build_guard.rs | AI008 (DeploymentAI) | P2-Medium |
| `backend/m141_relay_monitor.rs` | `backend::deployment` | Relay monitoring | m140_builder_monitor.rs | AI008 (DeploymentAI) | P2-Medium |
| `backend/copilot_system_access.rs` | `backend::copilot` | Copilot access control | security_gate.rs | AI016 (CopilotAI) | P1-High |
| `backend/m142_reim.rs` | `backend::reim` | REIM (Reinforcement Learning) | ai_agents.rs | AI009 (LearningAI) | P1-High |
| `backend/m056_learning_engine.rs` | `backend::learning` | Learning engine | m142_reim.rs | AI009 (LearningAI) | P1-High |
| `backend/m060_model_trainer.rs` | `backend::learning` | Model training | m056_learning_engine.rs | AI009 (LearningAI) | P1-High |
| `backend/m064_data_pipeline.rs` | `backend::learning` | Data pipeline | m065_feature_store.rs | AI009 (LearningAI) | P1-High |
| `backend/m065_feature_store.rs` | `backend::learning` | Feature store | m064_data_pipeline.rs | AI009 (LearningAI) | P1-High |
| `backend/continuum_optimization.rs` | `backend::optimization` | Continuum optimization | m054_auto_optimizer.rs | AI010 (OptimizationAI) | P1-High |
| `backend/m044_optimization.rs` | `backend::optimization` | General optimization | continuum_optimization.rs | AI010 (OptimizationAI) | P1-High |
| `backend/m054_auto_optimizer.rs` | `backend::optimization` | Auto-optimization | m044_optimization.rs | AI010 (OptimizationAI) | P1-High |
| `backend/fixed_point_core.rs` | `backend::math` | Fixed-point arithmetic | -- | AI010 (OptimizationAI) | P1-High |
| `backend/simd_state.rs` | `backend::math` | SIMD state management | fixed_point_core.rs | AI010 (OptimizationAI) | P1-High |
| `backend/multi_objective_solver.rs` | `backend::math` | Multi-objective optimization | fixed_point_core.rs | AI010 (OptimizationAI) | P1-High |
| `backend/rolling_window.rs` | `backend::math` | Rolling window calculations | fixed_point_core.rs | AI010 (OptimizationAI) | P1-High |
| `backend/optimization_velocity.rs` | `backend::math` | Optimization velocity | fixed_point_core.rs | AI010 (OptimizationAI) | P1-High |
| `backend/graph_route_optimizer.rs` | `backend::routing` | Graph-based routing | m026_order_router.rs | AI011 (RoutingAI) | P1-High |
| `backend/m026_order_router.rs` | `backend::routing` | Order routing | graph_route_optimizer.rs | AI011 (RoutingAI) | P1-High |
| `backend/m057_pool_dispatcher.rs` | `backend::routing` | Pool dispatch | m026_order_router.rs | AI011 (RoutingAI) | P1-High |
| `backend/m027_slippage_calculator.rs` | `backend::pricing` | Slippage calculation | m023_liquidity_analyzer.rs | AI012 (PricingAI) | P1-High |
| `backend/m024_price_monitor.rs` | `backend::pricing` | Price monitoring | m027_slippage_calculator.rs | AI012 (PricingAI) | P1-High |
| `backend/m007_gas_oracle.rs` | `backend::pricing` | Gas price oracle | m024_price_monitor.rs | AI012 (PricingAI) | P1-High |
| `backend/m029_access_controller.rs` | `backend::security` | Access control | security_gate.rs | AI004 (SecurityAI) | P0-Critical |
| `backend/m031_key_rotator.rs` | `backend::security` | Key rotation | m030_encryption_manager.rs | AI004 (SecurityAI) | P0-Critical |
| `backend/m030_encryption_manager.rs` | `backend::security` | Encryption | m031_key_rotator.rs | AI004 (SecurityAI) | P0-Critical |
| `backend/m055_env_vault.rs` | `backend::security` | Environment secrets | m030_encryption_manager.rs | AI004 (SecurityAI) | P0-Critical |
| `backend/intrusion_detection.rs` | `backend::security` | Intrusion detection | m035_threat_monitor.rs | AI004 (SecurityAI) | P0-Critical |
| `backend/m035_threat_monitor.rs` | `backend::security` | Threat monitoring | intrusion_detection.rs | AI004 (SecurityAI) | P0-Critical |
| `backend/m036_incident_responder.rs` | `backend::security` | Incident response | m035_threat_monitor.rs | AI004 (SecurityAI) | P0-Critical |
| `backend/m037_backup_manager.rs` | `backend::reliability` | Backup management | disaster_recovery.rs | AI014 (ReliabilityAI) | P1-High |
| `backend/disaster_recovery.rs` | `backend::reliability` | Disaster recovery | m037_backup_manager.rs | AI014 (ReliabilityAI) | P1-High |
| `backend/m021_regional_modules.rs` | `backend::infrastructure` | Regional modules | m066_fleet_controller.rs | AI013 (InfrastructureAI) | P1-High |
| `backend/m066_fleet_controller.rs` | `backend::infrastructure` | Fleet control | m021_regional_modules.rs | AI013 (InfrastructureAI) | P1-High |
| `backend/m082_k8s_manager.rs` | `backend::infrastructure` | Kubernetes management | m066_fleet_controller.rs | AI013 (InfrastructureAI) | P1-High |
| `backend/m039_load_balancer.rs` | `backend::infrastructure` | Load balancing | m082_k8s_manager.rs | AI013 (InfrastructureAI) | P1-High |
| `backend/m040_service_mesh.rs` | `backend::infrastructure` | Service mesh | m039_load_balancer.rs | AI013 (InfrastructureAI) | P1-High |

### 1.2 Frontend Modules (TypeScript/React)

| File Path | Parent Module | Primary Function | Dependencies | AI Agent | Governance Classification |
|-----------|---------------|------------------|--------------|----------|---------------------------|
| `apps/dashboard/src/main.tsx` | `dashboard` | React entry point | React, Vite | AI015 (FrontendAI) | P1-High |
| `apps/dashboard/src/App.tsx` | `dashboard` | Root component | React Router | AI015 (FrontendAI) | P1-High |
| `apps/dashboard/src/types.ts` | `dashboard` | Type definitions | -- | AI015 (FrontendAI) | P2-Medium |
| `apps/dashboard/src/components/Sidebar.tsx` | `dashboard::ui` | Navigation sidebar | React | AI015 (FrontendAI) | P2-Medium |
| `apps/dashboard/src/components/Topbar.tsx` | `dashboard::ui` | Top navigation bar | React | AI015 (FrontendAI) | P2-Medium |
| `apps/dashboard/src/components/DashboardView.tsx` | `dashboard::views` | Main dashboard view | kpi_telemetry.rs (IPC) | AI015 (FrontendAI) | P1-High |
| `apps/dashboard/src/components/ComplianceView.tsx` | `dashboard::views` | Compliance dashboard | m080_compliance_reporter.rs (IPC) | AI015 (FrontendAI) | P1-High |
| `apps/dashboard/src/components/CopilotPanel.tsx` | `dashboard::views` | Copilot AI panel | copilot_system_access.rs (IPC) | AI016 (CopilotAI) | P1-High |
| `apps/dashboard/src/components/CommanderView.tsx` | `dashboard::views` | Commander control panel | m050_governance_engine.rs (IPC) | AI019 (CommanderAI) | P0-Critical |
| `apps/dashboard/src/components/WalletView.tsx` | `dashboard::views` | Wallet management | m001_wallet_management.rs (IPC) | AI015 (FrontendAI) | P1-High |
| `apps/dashboard/src/hooks/useWeb3Wallet.ts` | `dashboard::hooks` | Web3 wallet hook | ethers.js | AI015 (FrontendAI) | P1-High |

### 1.3 Smart Contracts (Solidity)

| File Path | Parent Module | Primary Function | Dependencies | AI Agent | Governance Classification |
|-----------|---------------|------------------|--------------|----------|---------------------------|
| `contracts/FlashLoanArbitrage.sol` | `contracts` | Flash loan receiver | Aave V3, Balancer V2, Uniswap V2 | AI018 (SolidityBot) | P0-Critical |
| `contracts/test/FlashLoanArbitrage.t.sol` | `contracts::test` | Unit tests | forge-std | AI018 (SolidityBot) | P1-High |
| `contracts/Foundry.toml` | `contracts` | Foundry config | forge, std | AI018 (SolidityBot) | P1-High |
| `AB4/CircuitBreaker.sol` | `contracts` | Circuit breaker logic | OpenZeppelin | AI018 (SolidityBot) | P1-High |

---

## 2. Orphan Files (No Parent Module Registered)

| File Path | Issue | Recommendation |
|-----------|-------|----------------|
| `AB4/checkis` | No extension, no parent module registered | Register in MODULE_REGISTRY.toml or remove if obsolete |
| `AB4/d` | Single letter directory, purpose unclear | Audit content; register or archive |
| `AB4/dimensionsd` | Appears to be a typo (`dimensionsd` vs `dimensions`) | Rename to `dimensions` or remove |
| `AB4/pagepage` | No extension, unclear purpose | Document or remove |
| `AB4/probog` | No extension, unclear purpose | Document or remove |
| `AB4/objecomandmd` | No extension, appears to be concatenation | Split into proper files or remove |

**Total Orphans:** 6 files/directories

---

## 3. Unregistered Modules

| Module File | Expected Parent | Status |
|-------------|-----------------|--------|
| `backend/m001_wallet_management.rs` | `backend::wallet` | Unregistered |
| `backend/m003_transaction_batcher.rs` | `backend::batching` | Unregistered |
| `backend/m008_mev_protection.rs` | `backend::mev` | Unregistered |
| `backend/m009_latency.rs` | `backend::telemetry` | Unregistered |
| `backend/m010_portfolio_rebalancer.rs` | `backend::portfolio` | Unregistered |
| `backend/m011_yield_aggregator.rs` | `backend::yield` | Unregistered |
| `backend/m012_risk_calculator.rs` | `backend::risk` | Unregistered |
| `backend/m013_compliance_checker.rs` | `backend::compliance` | Unregistered |
| `backend/m015_performance_reporter.rs` | `backend::reporting` | Unregistered |
| `backend/m034_anomaly_detector.rs` | `backend::security` | Unregistered |
| `backend/m038_container_manager.rs` | `backend::infrastructure` | Unregistered |
| `backend/m042_config_manager.rs` | `backend::config` | Unregistered |
| `backend/m043_secret_manager.rs` | `backend::security` | Unregistered |
| `backend/m045_health_checker.rs` | `backend::reliability` | Unregistered |
| `backend/m047_log_aggregator.rs` | `backend::observability` | Unregistered |

**Total Unregistered:** 15 modules

---

## 4. Missing AI Ownership

Files without assigned AI agent (using corrected AI### format):

| File Path | Suggested Agent |
|-----------|-----------------|
| `backend/balance_simulator.rs` | AI017 (SimulationAI) |
| `backend/build.rs` | AI008 (DeploymentAI) |
| `backend/c2_redundancy.rs` | AI013 (InfrastructureAI) |
| `backend/c2_service.proto` | AI013 (InfrastructureAI) |
| `backend/cert_utils.rs` | AI004 (SecurityAI) |
| `backend/champion_challenger.rs` | AI009 (LearningAI) |
| `backend/chaos_lab.rs` | AI014 (ReliabilityAI) |
| `backend/constitution_guard.rs` | AI005 (GovernanceAI) |
| `backend/cross_agent_learning.rs` | AI009 (LearningAI) |
| `backend/db_init.rs` | AI008 (DeploymentAI) |
| `backend/error.rs` | AI020 (CoreAI) |
| `backend/key_manager.rs` | AI004 (SecurityAI) |
| `backend/multi_objective_solver.rs` | AI010 (OptimizationAI) |
| `backend/nonce_manager.rs` | AI004 (SecurityAI) |
| `backend/optimization_velocity.rs` | AI010 (OptimizationAI) |
| `backend/private_mempool.rs` | AI004 (SecurityAI) |
| `backend/relationship_matrix.rs` | AI010 (OptimizationAI) |
| `backend/signer.rs` | AI004 (SecurityAI) |
| `backend/signer.proto` | AI004 (SecurityAI) |
| `backend/telemetry.rs` | AI007 (TelemetryAI) |
| `backend/upgrade4_pipeline.rs` | AI008 (DeploymentAI) |
| `backend/upgrade4_kpi_sim/` | AI017 (SimulationAI) |
| `backend/upgrade4_sim/` | AI017 (SimulationAI) |
| `backend/ai/` (directory) | AI001 (AISE) |

**Total Missing Ownership:** 24 files/directories

---

## 5. AI Agent Registry (Corrected)

| Agent ID | Name | Type | Modules Assigned |
|----------|------|------|------------------|
| **AI001** | Core AI System (AISE) | Core | 8 |
| **AI002** | Trading Engine AI | Functional | 5 |
| **AI003** | Flash Loan AI | Functional | 2 |
| **AI004** | Security AI | Functional | 12 |
| **AI005** | Governance AI | Functional | 4 |
| **AI006** | Audit AI | Functional | 5 |
| **AI007** | Telemetry AI | Functional | 6 |
| **AI008** | Deployment AI | Functional | 6 |
| **AI009** | Learning AI | Functional | 5 |
| **AI010** | Optimization AI | Functional | 5 |
| **AI011** | Routing AI | Functional | 3 |
| **AI012** | Pricing AI | Functional | 3 |
| **AI013** | Infrastructure AI | Functional | 5 |
| **AI014** | Reliability AI | Functional | 2 |
| **AI015** | Frontend AI | Functional | 8 |
| **AI016** | Copilot AI | Functional | 3 |
| **AI017** | Simulation AI | Functional | 2 |
| **AI018** | Solidity Bot | Functional | 4 |
| **AI019** | Commander AI | Functional | 3 |
| **AI020** | Core System AI | Functional | 2 |

---

## 6. Duplicate Functionality

### 6.1 Duplicate Pairs

| File A | File B | Overlap | Recommendation |
|--------|--------|---------|----------------|
| `backend/kpi_telemetry.rs` | `backend/m083_metrics.rs` | Both collect KPI metrics | Consolidate into `kpi_telemetry.rs` |
| `backend/m046_metrics_collector.rs` | `backend/m083_metrics.rs` | Both collect metrics | Consolidate into `m046_metrics_collector.rs` |
| `backend/deployment.rs` | `backend/hot_swap_module.rs` | Both handle deployment | Split concerns |
| `backend/m133_sovereign_audit.rs` | `backend/m134_commander_audit.rs` | Both perform audit | Merge with role parameter |

---

## Note

This is a corrected version. The original FILE_MODULE_MAPPING_REPORT.md contains the same data but with incorrect AI agent naming (TradingAI, SecurityAI, etc. instead of AI002, AI004, etc.).

**Correction applied:** All AI agent references updated to AI001-AI999 format per NAMING_CONVENTION_CORRECTION.md.