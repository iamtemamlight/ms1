// ==============================================================================
// Unified Intelligence Module (UIM)
// Purpose: Central orchestration layer for all AI agents and modules
// Layer: Intelligence Layer — between Copilot and Operational Layer
// ==============================================================================

use std::collections::HashMap;
use chrono;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentCategory {
    Core,
    FleetManagement,
    Trading,
    Governance,
    Infrastructure,
    Operations,
    Management,
    Analysis,
    Supervisor,
    Constitutional,
    CopilotAudit,
}

#[derive(Debug, Clone)]
pub struct AgentRegistration {
    pub id: String,
    pub name: String,
    pub category: AgentCategory,
    pub module_id: String,
    pub enabled: bool,
    pub health: f64,
    pub last_execution: Option<String>,
    pub executions: u64,
    pub successes: u64,
    pub failures: u64,
}

#[derive(Debug, Clone)]
pub struct ModuleAgentMapping {
    pub module_id: String,
    pub module_name: String,
    pub agent_id: String,
    pub agent_name: String,
    pub status: MappingStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MappingStatus {
    Mapped,
    Unmapped,
    Shared,
    Supervisor,
}

#[derive(Debug)]
pub struct AiseUnifiedIntelligence {
    pub agents: HashMap<String, AgentRegistration>,
    pub module_mappings: HashMap<String, ModuleAgentMapping>,
    pub supervisor_agents: Vec<String>,
    pub copilot_loop_active: bool,
    pub last_sync: Option<String>,
    pub total_agents: u64,
    pub active_agents: u64,
}

impl AiseUnifiedIntelligence {
    pub fn new() -> Self {
        let mut uim = Self {
            agents: HashMap::new(),
            module_mappings: HashMap::new(),
            supervisor_agents: vec![
                "AI097".to_string(), // Supervisor Core
                "AI098".to_string(), // Supervisor Trading
                "AI099".to_string(), // Supervisor Security
                "AI100".to_string(), // Supervisor Infrastructure
                "AI101".to_string(), // Supervisor Profit
                "AI102".to_string(), // Supervisor Growth
                "AI103".to_string(), // Supervisor Velocity
                "AI104".to_string(), // Supervisor Efficiency
                "AI105".to_string(), // Supervisor Security Subsystem
                "AI106".to_string(), // Supervisor Quality
                "AI107".to_string(), // Copilot Auditor
            ],
            copilot_loop_active: false,
            last_sync: None,
            total_agents: 0,
            active_agents: 0,
        };

        uim.initialize_default_mappings();
        uim.register_default_agents();
        uim
    }

    fn initialize_default_mappings(&mut self) {
        let mappings = vec![
            // Core modules
            ("M001", "Wallet Management Engine", "AI001", "Desktop Agent", MappingStatus::Mapped),
            ("M002", "Installer Agent", "AI002", "Installer Agent", MappingStatus::Mapped),
            // Fleet management
            ("M003", "Transaction Batcher", "AI003", "Health Monitor", MappingStatus::Shared),
            ("M004", "Auto-Optimization Agent", "AI004", "Risk Manager", MappingStatus::Shared),
            ("M005", "State Synchronizer", "AI019", "State Syncer", MappingStatus::Mapped),
            ("M006", "Central C2 Server", "AI097", "Supervisor Core", MappingStatus::Supervisor),
            // Trading modules
            ("M007", "Gas Price Oracle", "AI010", "Gas Optimizer", MappingStatus::Mapped),
            ("M008", "MEV Protection Engine", "AI008", "MEV Shield", MappingStatus::Mapped),
            ("M009", "Latency Tracking", "AI006", "Latency Tracker", MappingStatus::Mapped),
            ("M010", "Portfolio Rebalancer", "AI007", "Pool Rebalancer", MappingStatus::Mapped),
            ("M011", "Yield Aggregator", "AI005", "Yield Optimizer", MappingStatus::Mapped),
            ("M012", "Risk Calculator", "AI004", "Risk Manager", MappingStatus::Mapped),
            ("M013", "Compliance Checker", "AI017", "Compliance Checker", MappingStatus::Mapped),
            ("M014", "Audit Logger", "AI095", "Audit Logger", MappingStatus::Mapped),
            ("M015", "Performance Reporter", "AI016", "Performance Tracker", MappingStatus::Mapped),
            ("M016", "Liquidity Depth Assessment", "AI021", "Liquidity Scanner", MappingStatus::Mapped),
            ("M017", "Gas Cycle Timing", "AI026", "Gas Tracker", MappingStatus::Mapped),
            ("M018", "Solver Precision Tradeoff", "AI085", "Model Agent", MappingStatus::Mapped),
            ("M019", "Cross-Region State Sync", "AI020", "Analytics Engine", MappingStatus::Shared),
            ("M020", "Regional Routing", "AI074", "Router", MappingStatus::Mapped),
            ("M021", "Cross-Region State Sync", "AI019", "State Syncer", MappingStatus::Mapped),
            ("M022", "Arbitrage Detector", "AI013", "Arbitrage Scanner", MappingStatus::Mapped),
            ("M023", "Liquidity Analyzer", "AI021", "Liquidity Scanner", MappingStatus::Mapped),
            ("M024", "Price Monitor", "AI022", "Price Feed", MappingStatus::Mapped),
            ("M025", "Trade Executor", "AI024", "Swap Router", MappingStatus::Mapped),
            ("M026", "Order Router", "AI074", "Router", MappingStatus::Mapped),
            ("M027", "Slippage Calculator", "AI011", "Slippage Monitor", MappingStatus::Mapped),
            ("M028", "Fraud Detector", "AI080", "Detector", MappingStatus::Mapped),
            ("M029", "Access Controller", "AI035", "Access Control", MappingStatus::Mapped),
            ("M030", "Encryption Manager", "AI030", "Bridge Relayer", MappingStatus::Shared),
            // Infrastructure modules
            ("M031", "Key Rotator", "AI012", "Nonce Manager", MappingStatus::Shared),
            ("M032", "Certificate Manager", "AI078", "Firewall", MappingStatus::Shared),
            ("M033", "Audit Trail", "AI088", "Auditor", MappingStatus::Mapped),
            ("M034", "Anomaly Detector", "AI080", "Detector", MappingStatus::Mapped),
            ("M035", "Threat Monitor", "AI035", "Access Control", MappingStatus::Shared),
            ("M036", "Incident Responder", "AI098", "Supervisor Trading", MappingStatus::Supervisor),
            ("M037", "Backup Manager", "AI099", "Supervisor Security", MappingStatus::Supervisor),
            ("M038", "Container Manager", "AI100", "Supervisor Infrastructure", MappingStatus::Supervisor),
            ("M039", "Load Balancer", "AI060", "Load Balancer", MappingStatus::Mapped),
            ("M040", "Service Mesh", "AI074", "Router", MappingStatus::Shared),
            ("M042", "Configuration Manager", "AI059", "Cache Manager", MappingStatus::Shared),
            ("M043", "Secret Manager", "AI055", "Distribution Manager", MappingStatus::Shared),
            ("M044", "DEX Optimization", "AI024", "Swap Router", MappingStatus::Mapped),
            ("M045", "Health Checker", "AI003", "Health Monitor", MappingStatus::Mapped),
            ("M046", "Metrics Collector", "AI063", "Metrics Aggregator", MappingStatus::Mapped),
            ("M047", "Log Aggregator", "AI062", "Logger", MappingStatus::Mapped),
            ("M048", "Alert Dispatcher", "AI051", "Alert Dispatcher", MappingStatus::Mapped),
            ("M049", "Incident Tracker", "AI098", "Supervisor Trading", MappingStatus::Supervisor),
            ("M050", "Governance Engine", "AI092", "Constitution Enforcer", MappingStatus::Mapped),
            // Security modules
            ("M051", "Mimicry Engine", "AI008", "MEV Shield", MappingStatus::Shared),
            ("M052", "DEX Router", "AI024", "Swap Router", MappingStatus::Mapped),
            ("M053", "Guardrails", "AI053", "Fee Collector", MappingStatus::Shared),
            ("M054", "Auto Optimizer", "AI004", "Risk Manager", MappingStatus::Shared),
            ("M055", "Encrypted Vault", "AI055", "Distribution Manager", MappingStatus::Shared),
            ("M056", "Learning Engine", "AI093", "Relationship Matrix Learner", MappingStatus::Mapped),
            ("M057", "Pool Dispatcher", "AI073", "Pool Manager", MappingStatus::Mapped),
            ("M058", "Shadow Replay", "AI084", "Simulator", MappingStatus::Mapped),
            ("M059", "State Synchronizer", "AI019", "State Syncer", MappingStatus::Mapped),
            ("M060", "Model Trainer", "AI086", "Trainer", MappingStatus::Mapped),
            ("M061", "Daily Profit Cap", "AI015", "Emergency Stop", MappingStatus::Mapped),
            ("M062", "Hourly Profit Cap", "AI015", "Emergency Stop", MappingStatus::Shared),
            ("M063", "Daily Loss Limit", "AI015", "Emergency Stop", MappingStatus::Shared),
            ("M064", "Data Pipeline", "AI093", "Relationship Matrix Learner", MappingStatus::Shared),
            ("M065", "Feature Store", "AI082", "Predictor", MappingStatus::Shared),
            ("M066", "Fleet Controller", "AI003", "Health Monitor", MappingStatus::Shared),
            ("M067", "RPC Consensus", "AI075", "Gateway", MappingStatus::Mapped),
            ("M068", "Market Scanner", "AI021", "Liquidity Scanner", MappingStatus::Shared),
            ("M069", "Opportunity Analyzer", "AI082", "Predictor", MappingStatus::Mapped),
            ("M070", "Trade Optimizer", "AI004", "Risk Manager", MappingStatus::Shared),
            ("M071", "Execution Engine", "AI024", "Swap Router", MappingStatus::Shared),
            ("M072", "Portfolio Manager", "AI007", "Pool Rebalancer", MappingStatus::Shared),
            ("M073", "Cross-Agent Learning", "AI093", "Relationship Matrix Learner", MappingStatus::Mapped),
            ("M074", "Champion/Challenger", "AI086", "Trainer", MappingStatus::Mapped),
            ("M075", "C2 Redundancy", "AI097", "Supervisor Core", MappingStatus::Supervisor),
            ("M076", "Disaster Recovery", "AI099", "Supervisor Security", MappingStatus::Supervisor),
            ("M077", "Intrusion Detection", "AI035", "Access Control", MappingStatus::Shared),
            ("M078", "Governance Auditor", "AI088", "Auditor", MappingStatus::Mapped),
            ("M079", "Constitutional Enforcer", "AI092", "Constitution Enforcer", MappingStatus::Mapped),
            ("M080", "Compliance Reporter", "AI068", "Reporter", MappingStatus::Mapped),
            ("M081", "YAML Templates", "AI082", "Predictor", MappingStatus::Shared),
            ("M082", "K8s Manager", "AI100", "Supervisor Infrastructure", MappingStatus::Supervisor),
            ("M083", "Metrics Aggregator", "AI063", "Metrics Aggregator", MappingStatus::Mapped),
            ("M084", "Alert System", "AI051", "Alert Dispatcher", MappingStatus::Mapped),
            ("M086", "Market Conditions Observer", "AI022", "Price Feed", MappingStatus::Shared),
            ("M087", "Regulatory Environment", "AI017", "Compliance Checker", MappingStatus::Shared),
            ("M088", "Yield Factors", "AI005", "Yield Optimizer", MappingStatus::Shared),
            ("M099", "ZK Proof Security", "AI053", "Fee Collector", MappingStatus::Shared),
            // External modules
            ("M100", "Advanced Analytics", "AI081", "Analyzer", MappingStatus::Mapped),
            ("M101", "Predictive Engine", "AI082", "Predictor", MappingStatus::Mapped),
            ("M102", "Scenario Simulator", "AI084", "Simulator", MappingStatus::Mapped),
            ("M103", "Stress Tester", "AI104", "Chaos Engineer", MappingStatus::Mapped),
            ("M104", "Chaos Engineer", "AI104", "Chaos Engineer", MappingStatus::Mapped),
            ("M105", "Auto Scaler", "AI060", "Load Balancer", MappingStatus::Shared),
            ("M106", "Cost Optimizer", "AI063", "Metrics Aggregator", MappingStatus::Shared),
            ("M107", "Resource Scheduler", "AI069", "Scheduler", MappingStatus::Mapped),
            ("M108", "Capacity Planner", "AI082", "Predictor", MappingStatus::Shared),
            ("M109", "Performance Tuner", "AI066", "Profiler", MappingStatus::Mapped),
            ("M110", "Security Hardener", "AI078", "Firewall", MappingStatus::Mapped),
            ("M111", "Compliance Automator", "AI017", "Compliance Checker", MappingStatus::Shared),
            ("M112", "Audit Automator", "AI088", "Auditor", MappingStatus::Shared),
            ("M113", "Report Generator", "AI068", "Reporter", MappingStatus::Mapped),
            ("M114", "Dashboard Builder", "AI081", "Analyzer", MappingStatus::Shared),
            ("M115", "Alert Manager", "AI051", "Alert Dispatcher", MappingStatus::Shared),
            ("M116", "Incident Manager", "AI098", "Supervisor Trading", MappingStatus::Supervisor),
            ("M117", "Change Manager", "AI097", "Supervisor Core", MappingStatus::Supervisor),
            ("M118", "Release Manager", "AI097", "Supervisor Core", MappingStatus::Supervisor),
            ("M119", "Config Controller", "AI059", "Cache Manager", MappingStatus::Shared),
            ("M120", "Feature Controller", "AI059", "Cache Manager", MappingStatus::Shared),
            ("M121", "Access Manager", "AI035", "Access Control", MappingStatus::Shared),
            ("M122", "Identity Manager", "AI035", "Access Control", MappingStatus::Shared),
            ("M123", "Secret Controller", "AI055", "Distribution Manager", MappingStatus::Shared),
            ("M124", "Key Manager", "AI012", "Nonce Manager", MappingStatus::Shared),
            ("M125", "Transaction Signer", "AI001", "Desktop Agent", MappingStatus::Shared),
            ("M126", "Mempool Monitor", "AI028", "Mempool Watcher", MappingStatus::Mapped),
            ("M127", "Bundle Submitter", "AI027", "Block Builder", MappingStatus::Mapped),
            ("M128", "Flashloan Executor", "AI014", "Flash Loan Guard", MappingStatus::Mapped),
            ("M129", "Arbitrage Executor", "AI013", "Arbitrage Scanner", MappingStatus::Mapped),
            ("M130", "Liquidity Provider", "AI021", "Liquidity Scanner", MappingStatus::Mapped),
            ("M131", "Yield Farmer", "AI005", "Yield Optimizer", MappingStatus::Mapped),
            ("M132", "Copilot Auditor", "AI107", "Copilot Auditor", MappingStatus::Supervisor),
            // Three-Layer Autonomous Audit Framework modules (M133-M137)
            ("M133", "Sovereign Audit Engine", "AI106", "Supervisor Quality", MappingStatus::Supervisor),
            ("M134", "Commander Audit & Learning", "AI100", "Supervisor Infrastructure", MappingStatus::Supervisor),
            ("M135", "Flash Loan Governance Governor", "AI092", "Constitution Enforcer", MappingStatus::Supervisor),
            ("M136", "Flash Loan Verifier", "AI088", "Auditor", MappingStatus::Mapped),
            ("M137", "Flash Loan Executor", "AI014", "Flash Loan Guard", MappingStatus::Mapped),
            // Orphan-agent modules (M138-M187): give every registered agent a
            // dedicated 1:1 module so the agent<->module mapping is bidirectional
            // (Protocol 3) and every module has exactly one owner (Protocol 1).
            ("M138", "Wallet Rotation", "AI009", "Wallet Rotator", MappingStatus::Mapped),
            ("M139", "Network Monitor", "AI018", "Network Monitor", MappingStatus::Mapped),
            ("M140", "Order Book", "AI023", "Order Book", MappingStatus::Mapped),
            ("M141", "Token Balance", "AI025", "Token Balance", MappingStatus::Mapped),
            ("M142", "Rollup Sequencer", "AI029", "Rollup Sequencer", MappingStatus::Mapped),
            ("M143", "NFT Manager", "AI031", "NFT Manager", MappingStatus::Mapped),
            ("M144", "Multisig Manager", "AI032", "Multisig Manager", MappingStatus::Mapped),
            ("M145", "Timelock Controller", "AI033", "Timelock Controller", MappingStatus::Mapped),
            ("M146", "Proxy Admin", "AI034", "Proxy Admin", MappingStatus::Mapped),
            ("M147", "Budget Manager", "AI036", "Budget Manager", MappingStatus::Mapped),
            ("M148", "Treasury", "AI037", "Treasury", MappingStatus::Mapped),
            ("M149", "Donation Manager", "AI038", "Donation Manager", MappingStatus::Mapped),
            ("M150", "Grant Manager", "AI039", "Grant Manager", MappingStatus::Mapped),
            ("M151", "Vesting Schedule", "AI040", "Vesting Schedule", MappingStatus::Mapped),
            ("M152", "Oracle Price Feed", "AI041", "Oracle Price", MappingStatus::Mapped),
            ("M153", "Oracle Aggregator", "AI042", "Aggregator", MappingStatus::Mapped),
            ("M154", "Validator Set", "AI043", "Validator Set", MappingStatus::Mapped),
            ("M155", "Slashing Manager", "AI044", "Slashing Manager", MappingStatus::Mapped),
            ("M156", "Delegation Manager", "AI045", "Delegation Manager", MappingStatus::Mapped),
            ("M157", "Snapshot Manager", "AI046", "Snapshot Manager", MappingStatus::Mapped),
            ("M158", "Proposal Manager", "AI047", "Proposal Manager", MappingStatus::Mapped),
            ("M159", "Vote Manager", "AI048", "Vote Manager", MappingStatus::Mapped),
            ("M160", "Governance Queue", "AI049", "Queuing Manager", MappingStatus::Mapped),
            ("M161", "Governance Execution", "AI050", "Execution Manager", MappingStatus::Mapped),
            ("M162", "Channel Manager", "AI052", "Channel Manager", MappingStatus::Mapped),
            ("M163", "Incentive Manager", "AI054", "Incentive Manager", MappingStatus::Mapped),
            ("M164", "Rate Limiter", "AI056", "Rate Limiter", MappingStatus::Mapped),
            ("M165", "Retry Manager", "AI057", "Retry Manager", MappingStatus::Mapped),
            ("M166", "Circuit Breaker", "AI058", "Circuit Breaker", MappingStatus::Mapped),
            ("M167", "Throttler", "AI061", "Throttler", MappingStatus::Mapped),
            ("M168", "Tracer", "AI064", "Tracer", MappingStatus::Mapped),
            ("M169", "Debugger", "AI065", "Debugger", MappingStatus::Mapped),
            ("M170", "System Monitor", "AI067", "Monitor", MappingStatus::Mapped),
            ("M171", "Worker Pool", "AI070", "Worker", MappingStatus::Mapped),
            ("M172", "Dispatcher", "AI071", "Dispatcher", MappingStatus::Mapped),
            ("M173", "Task Queue", "AI072", "Queue Manager", MappingStatus::Mapped),
            ("M174", "Bridge Manager", "AI076", "Bridge", MappingStatus::Mapped),
            ("M175", "Proxy Manager", "AI077", "Proxy", MappingStatus::Mapped),
            ("M176", "Vulnerability Scanner", "AI079", "Scanner", MappingStatus::Mapped),
            ("M177", "Forecaster", "AI083", "Forecaster", MappingStatus::Mapped),
            ("M178", "Validator", "AI087", "Validator", MappingStatus::Mapped),
            ("M179", "Inspector", "AI089", "Inspector", MappingStatus::Mapped),
            ("M180", "Reviewer", "AI090", "Reviewer", MappingStatus::Mapped),
            ("M181", "Approver", "AI091", "Approver", MappingStatus::Mapped),
            ("M182", "Subsystem Impact Analyzer", "AI094", "Subsystem Impact Analyzer", MappingStatus::Mapped),
            ("M183", "KPI Alignment Monitor", "AI096", "KPI Alignment Monitor", MappingStatus::Mapped),
            ("M184", "Profit Subsystem Supervisor", "AI101", "Supervisor Profit", MappingStatus::Supervisor),
            ("M185", "Growth Subsystem Supervisor", "AI102", "Supervisor Growth", MappingStatus::Supervisor),
            ("M186", "Velocity Subsystem Supervisor", "AI103", "Supervisor Velocity", MappingStatus::Supervisor),
            ("M187", "Security Subsystem Supervisor", "AI105", "Supervisor Security Enhanced", MappingStatus::Supervisor),
            // Advanced optimization modules (M200-M206)
            ("M200", "Bayesian Optimization Engine", "AI093", "Relationship Matrix Learner", MappingStatus::Mapped),
            ("M201", "Multi-Objective Pareto Optimizer", "AI085", "Model Agent", MappingStatus::Mapped),
            ("M202", "Predictive Gas Price Model", "AI010", "Gas Optimizer", MappingStatus::Mapped),
            ("M203", "Market Impact Model", "AI021", "Liquidity Scanner", MappingStatus::Mapped),
            ("M204", "Market Regime Detector", "AI022", "Price Feed", MappingStatus::Mapped),
            ("M205", "Federated Learning System", "AI093", "Relationship Matrix Learner", MappingStatus::Shared),
            ("M206", "Optimization Bounds Verifier", "AI096", "KPI Alignment Monitor", MappingStatus::Mapped),
            // On-chain governance modules (M300-M303)
            ("M300", "On-Chain Governance Executor", "AI092", "Constitution Enforcer", MappingStatus::Mapped),
            ("M301", "Timelock Controller", "AI033", "Timelock Controller", MappingStatus::Mapped),
            ("M302", "Cross-Chain Governance Sync", "AI019", "State Syncer", MappingStatus::Mapped),
            ("M303", "Governance Slashing Conditions", "AI044", "Slashing Manager", MappingStatus::Mapped),
        ];

        for (module_id, module_name, agent_id, agent_name, status) in mappings {
            let mapping = ModuleAgentMapping {
                module_id: module_id.to_string(),
                module_name: module_name.to_string(),
                agent_id: agent_id.to_string(),
                agent_name: agent_name.to_string(),
                status,
            };
            self.module_mappings.insert(module_id.to_string(), mapping);
        }
    }

    fn register_default_agents(&mut self) {
        let agents = vec![
            ("AI107", "Copilot Auditor", AgentCategory::CopilotAudit, "M132", true),
        ];

        for (id, name, category, module_id, enabled) in agents {
            let registration = AgentRegistration {
                id: id.to_string(),
                name: name.to_string(),
                category,
                module_id: module_id.to_string(),
                enabled,
                health: 1.0,
                last_execution: None,
                executions: 0,
                successes: 0,
                failures: 0,
            };
            self.register_agent(registration);
        }
    }

    pub fn register_agent(&mut self, agent: AgentRegistration) {
        self.agents.insert(agent.id.clone(), agent.clone());
        self.total_agents += 1;
        if agent.enabled {
            self.active_agents += 1;
        }
    }

    pub fn get_agent_for_module(&self, module_id: &str) -> Option<&ModuleAgentMapping> {
        self.module_mappings.get(module_id)
    }

    pub fn get_supervisor_agents(&self) -> Vec<&str> {
        self.supervisor_agents.iter().map(|s| s.as_str()).collect()
    }

    pub fn get_mapping_stats(&self) -> (usize, usize, usize, usize) {
        let mut mapped = 0;
        let mut unmapped = 0;
        let mut shared = 0;
        let mut supervisor = 0;

        for mapping in self.module_mappings.values() {
            match mapping.status {
                MappingStatus::Mapped => mapped += 1,
                MappingStatus::Unmapped => unmapped += 1,
                MappingStatus::Shared => shared += 1,
                MappingStatus::Supervisor => supervisor += 1,
            }
        }

        (mapped, unmapped, shared, supervisor)
    }

    pub fn get_unmapped_modules(&self) -> Vec<&str> {
        self.module_mappings
            .values()
            .filter(|m| m.status == MappingStatus::Unmapped)
            .map(|m| m.module_id.as_str())
            .collect()
    }

    pub fn sync_with_agent_registry(&mut self, agent_ids: &[String]) {
        self.last_sync = Some(chrono::Utc::now().to_rfc3339());
        self.active_agents = agent_ids.len() as u64;
    }

    /// Enable an agent by ID (returns error if not found).
    pub fn enable_agent(&mut self, id: &str) -> Result<(), String> {
        if let Some(registration) = self.agents.get_mut(id) {
            registration.enabled = true;
            self.active_agents += 1;
            Ok(())
        } else {
            Err(format!("Agent {} not found in unified intelligence registry", id))
        }
    }

    /// Disable an agent by ID (returns error if not found).
    pub fn disable_agent(&mut self, id: &str) -> Result<(), String> {
        if let Some(registration) = self.agents.get_mut(id) {
            registration.enabled = false;
            if registration.enabled && self.active_agents > 0 {
                self.active_agents -= 1;
            }
            Ok(())
        } else {
            Err(format!("Agent {} not found in unified intelligence registry", id))
        }
    }

    /// Get all registered agent IDs.
    pub fn get_all_agent_ids(&self) -> Vec<String> {
        self.agents.keys().cloned().collect()
    }

    /// Get a single agent registration by ID.
    pub fn get_agent(&self, id: &str) -> Option<&AgentRegistration> {
        self.agents.get(id)
    }
}
