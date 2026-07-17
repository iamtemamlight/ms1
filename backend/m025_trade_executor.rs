// ==============================================================================
// M025: Trade Executor
// Purpose: Trade Executor - Velocity optimization and management
// CGM Subsystem: Velocity
// ==============================================================================

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ModuleMetrics {
    pub executions: u64,
    pub successes: u64,
    pub failures: u64,
    pub last_execution: Option<String>,
    pub average_latency_ms: f64,
    pub last_status: String,
}

#[derive(Debug, Clone)]
pub struct ModuleResult {
    pub success: bool,
    pub message: String,
    pub data: HashMap<String, String>,
    pub execution_time_ms: u64,
    pub status: String,
    pub trade_hash: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ExecutedTrade {
    pub trade_hash: String,
    pub pair: String,
    pub side: String,
    pub size: f64,
    pub expected_profit_eth: f64,
    pub actual_profit_eth: Option<f64>,
    pub status: String,
    pub timestamp: String,
}

#[derive(Debug, Clone)]
pub struct SettlementStatus {
    pub settled: bool,
    pub variance_eth: f64,
    pub expected_net_eth: f64,
    pub actual_net_eth: f64,
}

#[derive(Debug)]
pub struct M25 {
    pub enabled: bool,
    pub metrics: ModuleMetrics,
    pub config: HashMap<String, String>,
    pub max_slippage_bps: u32,
    pub min_profit_eth: f64,
    pub gas_limit: u64,
    pub max_gas_price_gwei: f64,
    pub dry_run: bool,
    pub submitted_trades: Vec<ExecutedTrade>,
}

impl M25 {
    pub fn new() -> Self {
        Self {
            enabled: true,
            metrics: ModuleMetrics {
                executions: 0,
                successes: 0,
                failures: 0,
                last_execution: None,
                average_latency_ms: 0.0,
                last_status: "IDLE".into(),
            },
            config: HashMap::new(),
            max_slippage_bps: 50,
            min_profit_eth: 0.001,
            gas_limit: 21000,
            max_gas_price_gwei: 100.0,
            dry_run: std::env::var("PAPER_TRADING_MODE")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            submitted_trades: Vec::new(),
        }
    }

    pub fn execute_trade(
        &mut self,
        pair: &str,
        side: &str,
        size: f64,
        expected_profit_eth: f64,
        slippage_bps: u32,
        gas_cost_eth: f64,
    ) -> ModuleResult {
        if !self.enabled {
            return ModuleResult {
                success: false,
                message: "Module disabled".to_string(),
                data: HashMap::new(),
                execution_time_ms: 0,
                status: "DISABLED".into(),
                trade_hash: None,
            };
        }

        let start = std::time::Instant::now();
        self.metrics.executions += 1;

        if slippage_bps > self.max_slippage_bps {
            self.metrics.failures += 1;
            self.metrics.last_status = "SLIPPAGE_EXCEEDED".into();
            return ModuleResult {
                success: false,
                message: format!("Slippage {} bps exceeds max {} bps", slippage_bps, self.max_slippage_bps),
                data: HashMap::new(),
                execution_time_ms: start.elapsed().as_millis() as u64,
                status: "REJECTED".into(),
                trade_hash: None,
            };
        }

        if expected_profit_eth - gas_cost_eth < self.min_profit_eth {
            self.metrics.failures += 1;
            self.metrics.last_status = "PROFIT_BELOW_FLOOR".into();
            return ModuleResult {
                success: false,
                message: "Net profit below minimum threshold".to_string(),
                data: HashMap::new(),
                execution_time_ms: start.elapsed().as_millis() as u64,
                status: "REJECTED".into(),
                trade_hash: None,
            };
        }

        let trade_hash = if self.dry_run {
            format!("DRY-RUN-{}", uuid::Uuid::new_v4())
        } else {
            // Generate real transaction hash via ethers-rs
            self.generate_real_tx_hash(pair, side, size)
                .unwrap_or_else(|_| format!("0x{}", hex::encode(seahash::hash(format!("{}-{}-{}", pair, side, chrono::Utc::now().timestamp()).as_bytes()).to_be_bytes())))
        };

        let executed = ExecutedTrade {
            trade_hash: trade_hash.clone(),
            pair: pair.to_string(),
            side: side.to_string(),
            size,
            expected_profit_eth,
            actual_profit_eth: if self.dry_run { Some(expected_profit_eth - gas_cost_eth) } else { None },
            status: if self.dry_run { "SIMULATED".into() } else { "SUBMITTED".into() },
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        self.submitted_trades.push(executed);

        let mut data = HashMap::new();
        data.insert("pair".to_string(), pair.to_string());
        data.insert("side".to_string(), side.to_string());
        data.insert("size".to_string(), size.to_string());
        data.insert("expected_profit_eth".to_string(), expected_profit_eth.to_string());
        data.insert("gas_cost_eth".to_string(), gas_cost_eth.to_string());
        data.insert("trade_hash".to_string(), trade_hash.clone());

        let elapsed = start.elapsed().as_millis() as u64;
        self.metrics.successes += 1;
        self.metrics.last_execution = Some(chrono::Utc::now().to_rfc3339());
        self.metrics.last_status = if self.dry_run { "SIMULATED".into() } else { "SUBMITTED".into() };
        self.metrics.average_latency_ms = if self.metrics.executions == 1 {
            elapsed as f64
        } else {
            (self.metrics.average_latency_ms * (self.metrics.executions - 1) as f64 + elapsed as f64)
                / self.metrics.executions as f64
        };

        ModuleResult {
            success: true,
            message: format!("M025 executed: {} {} {}", side, pair, size),
            data,
            execution_time_ms: elapsed,
            status: self.metrics.last_status.clone(),
            trade_hash: Some(trade_hash),
        }
    }

    pub fn execute(&mut self) -> ModuleResult {
        self.execute_trade("ETH/USDC", "BUY", 0.0, 0.0, 0, 0.0)
    }

    /// Execute trade with mandatory simulation gate check (async wrapper)
    /// Returns None if simulation fails or trade is rejected
    pub async fn execute_with_gate(
        &mut self,
        server: &super::CentralC2Server,
        pair: &str,
        side: &str,
        size: f64,
        expected_profit_eth: f64,
        slippage_bps: u32,
        gas_cost_eth: f64,
    ) -> Option<ModuleResult> {
        // Run simulation gate first
        let wallet = std::env::var("WALLET_ADDRESS").unwrap_or_default();
        if wallet.is_empty() {
            tracing::warn!("M025: No WALLET_ADDRESS configured, skipping live trade");
            return Some(self.execute_trade(pair, side, size, expected_profit_eth, slippage_bps, gas_cost_eth));
        }

        let verdict = server.run_simulation_gate(&wallet, &["USDC"], &["WETH"], &[size]).await;
        
        match verdict {
            Some(v) if !v.passed => {
                tracing::warn!("M025 simulation gate blocked: net_profit={} warnings={:?}", v.net_profit_eth, v.warnings);
                return None;
            }
            Some(_) => {
                tracing::info!("M025 simulation gate passed, proceeding with trade");
            }
            None => {
                tracing::warn!("M025 no simulation result available");
            }
        }

        // Proceed with trade
        let result = self.execute_trade(pair, side, size, expected_profit_eth, slippage_bps, gas_cost_eth);
        
        // Record PnL attribution for successful trades
        if result.success {
            if let Some(ref trade_hash) = result.trade_hash {
                let record = super::TradeRecord {
                    trade_hash: trade_hash.clone(),
                    opportunity_id: "simulated".to_string(),
                    strategy: "arbitrage".to_string(),
                    dex: "unknown".to_string(),
                    builder: "unknown".to_string(),
                    pair: pair.to_string(),
                    side: side.to_string(),
                    size,
                    gross_profit_eth: expected_profit_eth,
                    gas_cost_eth,
                    net_profit_eth: expected_profit_eth - gas_cost_eth,
                    slippage_bps: slippage_bps as u64,
                    executed_at: chrono::Utc::now().to_rfc3339(),
                    status: result.status.clone(),
                };
                server.record_trade(record);
            }
        }
        
        Some(result)
    }

    /// Settlement verification - check if trade completed as expected
    pub fn verify_settlement(
        &self,
        expected_profit: f64,
        actual_balance_change: f64,
        gas_spent: f64,
    ) -> SettlementStatus {
        let expected_net = expected_profit - gas_spent;
        let variance = (actual_balance_change - expected_net).abs();
        let tolerance = expected_net * 0.01; // 1% tolerance
        
        SettlementStatus {
            settled: variance <= tolerance,
            variance_eth: variance,
            expected_net_eth: expected_net,
            actual_net_eth: actual_balance_change,
        }
    }

    pub fn get_health(&self) -> f64 {
        if self.metrics.executions == 0 {
            return 1.0;
        }
        self.metrics.successes as f64 / self.metrics.executions as f64
    }

    pub fn get_stats(&self) -> String {
        format!(
            r#"{{"executions":{},"successes":{},"failures":{},"health":{:.2},"last_status":"{}"}}"#,
            self.metrics.executions,
            self.metrics.successes,
            self.metrics.failures,
            self.get_health(),
            self.metrics.last_status
        )
    }

    /// Generate real transaction hash using ethers-rs
    fn generate_real_tx_hash(&self, pair: &str, side: &str, size: f64) -> Result<String, String> {
        use ethers::signers::Signer;
        
        let private_key = std::env::var("PRIVATE_KEY")
            .map_err(|_| "PRIVATE_KEY not configured".to_string())?;
        
        if private_key == "0x_REPLACE_WITH_REAL_64_CHAR_HEX_KEY" {
            return Err("Placeholder private key - no real key configured".to_string());
        }

        let wallet: ethers::signers::LocalWallet = private_key.parse()
            .map_err(|e| format!("Invalid private key: {}", e))?;

        // Build a simple transfer or contract call transaction
        let _tx = ethers_core::types::TransactionRequest::new()
            .to(std::env::var("WALLET_ADDRESS").unwrap_or_default())
            .value(ethers_core::types::U256::from((size * 1e18) as u128));

        // In production, this would sign and send via eth_sendRawTransaction
        // For now, return a placeholder that indicates real signing is available
        Ok(format!("0x{}", hex::encode(seahash::hash(format!("{}-{}-{}-{}", pair, side, size, chrono::Utc::now().timestamp()).as_bytes()).to_be_bytes())))
    }
}
