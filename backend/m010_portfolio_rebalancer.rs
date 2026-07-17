// ==============================================================================
// M010: Portfolio Rebalancer
// Purpose: Maintain optimal portfolio allocation across assets
// CGM Subsystem: Profit
// ==============================================================================

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct AssetAllocation {
    pub asset: String,
    pub target_weight: f64,
    pub current_weight: f64,
    pub current_value: f64,
    pub deviation: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RebalanceStrategy {
    Threshold,
    Periodic,
    VolatilityAdjusted,
}

#[derive(Debug, Clone)]
pub struct RebalanceAction {
    pub asset: String,
    pub action: RebalanceActionType,
    pub amount: f64,
    pub estimated_cost: f64,
    pub priority: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RebalanceActionType {
    Buy,
    Sell,
    Hold,
}

#[derive(Debug, Clone)]
pub struct RebalanceResult {
    pub actions: Vec<RebalanceAction>,
    pub total_deviation: f64,
    pub estimated_cost: f64,
    pub expected_return: f64,
    pub timestamp: String,
}

#[derive(Debug)]
pub struct PortfolioRebalancer {
    pub enabled: bool,
    pub allocations: HashMap<String, AssetAllocation>,
    pub strategy: RebalanceStrategy,
    pub rebalance_threshold: f64,
    pub last_rebalance: Option<String>,
    pub total_rebalances: u64,
    pub total_trades: u64,
}

impl PortfolioRebalancer {
    pub fn new() -> Self {
        let mut allocations = HashMap::new();
        allocations.insert("ETH".to_string(), AssetAllocation {
            asset: "ETH".to_string(),
            target_weight: 0.4,
            current_weight: 0.4,
            current_value: 0.0,
            deviation: 0.0,
        });
        allocations.insert("USDC".to_string(), AssetAllocation {
            asset: "USDC".to_string(),
            target_weight: 0.3,
            current_weight: 0.3,
            current_value: 0.0,
            deviation: 0.0,
        });
        allocations.insert("WBTC".to_string(), AssetAllocation {
            asset: "WBTC".to_string(),
            target_weight: 0.2,
            current_value: 0.0,
            current_weight: 0.2,
            deviation: 0.0,
        });
        allocations.insert("DAI".to_string(), AssetAllocation {
            asset: "DAI".to_string(),
            target_weight: 0.1,
            current_value: 0.0,
            current_weight: 0.1,
            deviation: 0.0,
        });

        Self {
            enabled: true,
            allocations,
            strategy: RebalanceStrategy::Threshold,
            rebalance_threshold: 0.05,
            last_rebalance: None,
            total_rebalances: 0,
            total_trades: 0,
        }
    }

    pub fn update_allocation(&mut self, asset: &str, current_value: f64, total_portfolio: f64) {
        if let Some(alloc) = self.allocations.get_mut(asset) {
            alloc.current_value = current_value;
            alloc.current_weight = if total_portfolio > 0.0 { current_value / total_portfolio } else { 0.0 };
            alloc.deviation = (alloc.current_weight - alloc.target_weight).abs();
        }
    }

    pub fn check_rebalance_needed(&self) -> bool {
        self.allocations.values().any(|a| a.deviation > self.rebalance_threshold)
    }

    pub fn calculate_rebalance(&mut self) -> RebalanceResult {
        if !self.enabled {
            return RebalanceResult {
                actions: vec![],
                total_deviation: 0.0,
                estimated_cost: 0.0,
                expected_return: 0.0,
                timestamp: chrono::Utc::now().to_rfc3339(),
            };
        }

        let mut actions = Vec::new();
        let mut total_deviation = 0.0;
        let mut estimated_cost = 0.0;

        for alloc in self.allocations.values() {
            total_deviation += alloc.deviation;

            if alloc.deviation > self.rebalance_threshold {
                let action_type = if alloc.current_weight < alloc.target_weight {
                    RebalanceActionType::Buy
                } else {
                    RebalanceActionType::Sell
                };

                let amount = (alloc.target_weight - alloc.current_weight).abs();
                let cost = amount * 0.003;

                actions.push(RebalanceAction {
                    asset: alloc.asset.clone(),
                    action: action_type,
                    amount,
                    estimated_cost: cost,
                    priority: (alloc.deviation * 10.0) as u8,
                });

                estimated_cost += cost;
                self.total_trades += 1;
            }
        }

        actions.sort_by(|a, b| b.priority.cmp(&a.priority));

        let expected_return = total_deviation * 0.15;

        self.last_rebalance = Some(chrono::Utc::now().to_rfc3339());
        self.total_rebalances += 1;

        RebalanceResult {
            actions,
            total_deviation,
            estimated_cost,
            expected_return,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn get_portfolio_summary(&self) -> String {
        let total_value: f64 = self.allocations.values().map(|a| a.current_value).sum();
        
        let allocations: Vec<String> = self.allocations.values()
            .map(|a| format!(r#""{}":{{"value":{},"weight":{:.3}}}"#, a.asset, a.current_value, a.current_weight))
            .collect();

        format!(
            r#"{{"total_value":{},"allocations":{{{}}}"#,
            total_value,
            allocations.join(",")
        )
    }
}
