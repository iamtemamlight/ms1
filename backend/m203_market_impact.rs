// ==============================================================================
// M203: Market Impact Model (Almgren-Chriss)
// Purpose: Predict market impact for optimal position sizing and slippage estimation
//          Replaces static slippage parameters with dynamic market-aware model
// CGM Subsystem: Efficiency / Shield
// ==============================================================================

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Market impact parameters for a DEX/pool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketImpactParams {
    pub pool_address: String,
    pub dex: String,
    pub token_in: String,
    pub token_out: String,
    pub liquidity_depth_usd: f64,
    pub daily_volume_usd: f64,
    pub volatility: f64,
    pub spread_bps: f64,
    pub fee_bps: f64,
    pub block_number: u64,
    pub timestamp_ms: u64,
}

/// Trade characteristics for impact calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeCharacteristics {
    pub trade_size_usd: f64,
    pub is_buy: bool,
    pub is_urgent: bool,
    pub max_slippage_bps: f64,
}

/// Market impact result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketImpactResult {
    pub permanent_impact_usd: f64,
    pub temporary_impact_usd: f64,
    pub total_impact_usd: f64,
    pub total_impact_bps: f64,
    pub optimal_execution_time_ms: u64,
    pub recommended_splits: u32,
    pub expected_slippage_bps: f64,
    pub confidence: f64,
    pub impact_breakdown: ImpactBreakdown,
}

/// Detailed impact breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactBreakdown {
    pub fee_cost_usd: f64,
    pub spread_cost_usd: f64,
    pub volatility_cost_usd: f64,
    pub depth_cost_usd: f64,
    pub urgency_premium_usd: f64,
}

/// Almgren-Chriss market impact model
#[derive(Debug, Clone)]
pub struct AlmgrenChrissModel {
    pub permanent_impact_coeff: f64,
    pub temporary_impact_coeff: f64,
    pub risk_aversion: f64,
    pub execution_urgency: f64,
    pub volatility_estimate: f64,
    pub liquidity_depth: f64,
    pub daily_volume: f64,
}

impl AlmgrenChrissModel {
    pub fn new() -> Self {
        Self {
            permanent_impact_coeff: 0.1,
            temporary_impact_coeff: 0.5,
            risk_aversion: 0.5,
            execution_urgency: 1.0,
            volatility_estimate: 0.02,
            liquidity_depth: 10_000_000.0,
            daily_volume: 100_000_000.0,
        }
    }

    /// Calculate market impact for a given trade
    pub fn calculate_impact(
        &self,
        params: &MarketImpactParams,
        trade: &TradeCharacteristics,
    ) -> MarketImpactResult {
        let size_ratio = trade.trade_size_usd / self.liquidity_depth.max(1.0);
        let volume_ratio = trade.trade_size_usd / self.daily_volume.max(1.0);

        // Almgren-Chriss permanent impact (linear in size)
        // I_permanent = eta * X where eta ~ 0.1-0.3 for DeFi
        let eta = self.permanent_impact_coeff * (1.0 + params.volatility);
        let permanent_impact = eta * trade.trade_size_usd * size_ratio;

        // Temporary impact (square root or linear depending on depth)
        // I_temporary = epsilon * sign(X) * |X|^alpha
        // For DeFi pools, typically square root: I_temp ~ gamma * sqrt(|X| * sigma / V)
        let gamma = self.temporary_impact_coeff * params.spread_bps / 10000.0;
        let sigma = self.volatility_estimate;
        let v = self.daily_volume;
        let temporary_impact = gamma * (trade.trade_size_usd * sigma / v).sqrt() * trade.trade_size_usd;

        // Fee cost
        let fee_cost = trade.trade_size_usd * params.fee_bps / 10000.0;

        // Spread cost
        let spread_cost = trade.trade_size_usd * params.spread_bps / 10000.0;

        // Volatility cost (risk of price movement during execution)
        let volatility_cost = trade.trade_size_usd * sigma * self.execution_urgency * 0.5;

        // Urgency premium (pay more to execute faster)
        let urgency_premium = if trade.is_urgent {
            temporary_impact * 0.5
        } else {
            0.0
        };

        let total_impact = permanent_impact + temporary_impact + fee_cost + spread_cost + volatility_cost + urgency_premium;
        let total_impact_bps = (total_impact / trade.trade_size_usd) * 10000.0;

        // Optimal execution time (fraction of day)
        let optimal_time_fraction = (self.risk_aversion * sigma.powi(2) * self.liquidity_depth / trade.trade_size_usd).sqrt();
        let optimal_execution_time_ms = (optimal_time_fraction * 86400.0 * 1000.0).max(100.0).min(60000.0) as u64;

        // Recommended splits (Iceberg-style execution)
        let recommended_splits = if trade.trade_size_usd > self.liquidity_depth * 0.1 {
            (trade.trade_size_usd / (self.liquidity_depth * 0.05)).ceil() as u32
        } else {
            1
        };

        let confidence = (1.0 - size_ratio.min(1.0)) * (1.0 - volume_ratio.min(1.0));

        MarketImpactResult {
            permanent_impact_usd: permanent_impact,
            temporary_impact_usd: temporary_impact,
            total_impact_usd: total_impact,
            total_impact_bps,
            optimal_execution_time_ms,
            recommended_splits: recommended_splits.max(1),
            expected_slippage_bps: total_impact_bps + params.spread_bps,
            confidence: confidence.max(0.3),
            impact_breakdown: ImpactBreakdown {
                fee_cost_usd: fee_cost,
                spread_cost_usd: spread_cost,
                volatility_cost_usd: volatility_cost,
                depth_cost_usd: permanent_impact,
                urgency_premium_usd: urgency_premium,
            },
        }
    }

    /// Calculate optimal trade size given max acceptable slippage
    pub fn optimal_trade_size(
        &self,
        params: &MarketImpactParams,
        max_slippage_bps: f64,
        is_urgent: bool,
    ) -> f64 {
        // Invert the impact model: find X such that total_impact_bps <= max_slippage_bps
        // Simplified: solve for X where impact/X * 10000 <= max_slippage_bps
        let gamma = self.temporary_impact_coeff * params.spread_bps / 10000.0;
        let eta = self.permanent_impact_coeff * (1.0 + params.volatility);
        let fee_and_spread = params.fee_bps + params.spread_bps;
        
        // target_impact_bps = max_slippage_bps - fee_and_spread
        let target_impact_bps = (max_slippage_bps - fee_and_spread).max(0.0);
        
        // Solve: (eta * X/L + gamma * sqrt(X * sigma / V)) * 10000 = target
        // Approximate solution using bisection
        let mut low = 0.0;
        let mut high = self.liquidity_depth * 0.5;
        
        for _ in 0..50 {
            let mid = (low + high) / 2.0;
            let size_ratio = mid / self.liquidity_depth.max(1.0);
            let volume_ratio = mid / self.daily_volume.max(1.0);
            let permanent = eta * size_ratio * 10000.0;
            let temporary = gamma * (mid * self.volatility_estimate / self.daily_volume).sqrt() * 10000.0;
            let total_bps = permanent + temporary;
            
            if total_bps > target_impact_bps {
                high = mid;
            } else {
                low = mid;
            }
        }
        
        (low + high) / 2.0
    }

    /// Update model parameters from observed trades
    pub fn update_from_observation(&mut self, predicted: f64, actual: f64) {
        // Update coefficients based on prediction error
        let error = (actual - predicted).abs() / (actual + 1e-9);
        let learning_rate = 0.05;
        
        if actual > predicted {
            // Underestimated impact, increase coefficients
            self.permanent_impact_coeff *= 1.0 + learning_rate * error.min(0.5);
            self.temporary_impact_coeff *= 1.0 + learning_rate * error.min(0.5);
        } else {
            // Overestimated impact, decrease coefficients
            self.permanent_impact_coeff *= 1.0 - learning_rate * error.min(0.5);
            self.temporary_impact_coeff *= 1.0 - learning_rate * error.min(0.5);
        }

        // Clamp coefficients
        self.permanent_impact_coeff = self.permanent_impact_coeff.max(0.01).min(1.0);
        self.temporary_impact_coeff = self.temporary_impact_coeff.max(0.1).min(2.0);
    }

    /// Set pool-specific parameters
    pub fn set_pool_params(&mut self, liquidity_depth: f64, daily_volume: f64, volatility: f64) {
        self.liquidity_depth = liquidity_depth;
        self.daily_volume = daily_volume;
        self.volatility_estimate = volatility;
    }
}

impl Default for AlmgrenChrissModel {
    fn default() -> Self {
        Self::new()
    }
}

/// Flash loan specific impact calculator
pub struct FlashLoanImpactCalculator {
    pub ac_model: AlmgrenChrissModel,
    pub pool_cache: HashMap<String, MarketImpactParams>,
    pub prediction_history: Vec<(f64, f64)>, // (predicted, actual)
}

impl FlashLoanImpactCalculator {
    pub fn new() -> Self {
        Self {
            ac_model: AlmgrenChrissModel::new(),
            pool_cache: HashMap::new(),
            prediction_history: Vec::new(),
        }
    }

    /// Get or create pool parameters
    pub fn get_pool_params(&mut self, pool_address: &str, dex: &str, token_in: &str, token_out: &str) -> &MarketImpactParams {
        if !self.pool_cache.contains_key(pool_address) {
            let params = MarketImpactParams {
                pool_address: pool_address.to_string(),
                dex: dex.to_string(),
                token_in: token_in.to_string(),
                token_out: token_out.to_string(),
                liquidity_depth_usd: 10_000_000.0,
                daily_volume_usd: 50_000_000.0,
                volatility: 0.02,
                spread_bps: 5.0,
                fee_bps: 30.0,
                block_number: 0,
                timestamp_ms: 0,
            };
            self.pool_cache.insert(pool_address.to_string(), params);
        }
        self.pool_cache.get(pool_address).unwrap()
    }

    /// Update pool parameters
    pub fn update_pool_params(&mut self, params: MarketImpactParams) {
        self.pool_cache.insert(params.pool_address.clone(), params);
    }

    /// Calculate impact for flash loan trade
    pub fn calculate_flash_loan_impact(
        &mut self,
        pool_address: &str,
        loan_size_eth: f64,
        eth_price_usd: f64,
    ) -> MarketImpactResult {
        let params = self.get_pool_params(pool_address, "uniswap-v3", "ETH", "USDC").clone();
        let trade_size_usd = loan_size_eth * eth_price_usd;
        let trade = TradeCharacteristics {
            trade_size_usd,
            is_buy: true,
            is_urgent: true,
            max_slippage_bps: 50.0,
        };

        let result = self.ac_model.calculate_impact(&params, &trade);
        
        // Record for model updating
        self.prediction_history.push((result.total_impact_bps, result.total_impact_bps));
        if self.prediction_history.len() > 100 {
            self.prediction_history.remove(0);
        }
        
        result
    }

    /// Update model from actual results
    pub fn update_from_actual(&mut self, pool_address: &str, predicted_bps: f64, actual_bps: f64) {
        if let Some(params) = self.pool_cache.get_mut(pool_address) {
            self.ac_model.update_from_observation(predicted_bps, actual_bps);
        }
    }

    /// Get optimal flash loan size for a pool
    pub fn optimal_flash_loan_size(&self, pool_address: &str, max_slippage_bps: f64, eth_price_usd: f64) -> f64 {
        if let Some(params) = self.pool_cache.get(pool_address) {
            let optimal_usd = self.ac_model.optimal_trade_size(
                params,
                max_slippage_bps,
                true,
            );
            optimal_usd / eth_price_usd
        } else {
            1.0 // Default 1 ETH
        }
    }

    /// Get model statistics
    pub fn get_stats(&self) -> String {
        let avg_error = if self.prediction_history.len() > 1 {
            self.prediction_history.iter()
                .map(|(p, a)| (p - a).abs())
                .sum::<f64>() / self.prediction_history.len() as f64
        } else {
            0.0
        };
        
        format!(
            r#"{{"pools":{},"predictions":{},"avg_error_bps":{:.2},"permanent_coeff":{:.4}}}"#,
            self.pool_cache.len(),
            self.prediction_history.len(),
            avg_error,
            self.ac_model.permanent_impact_coeff
        )
    }
}

impl Default for FlashLoanImpactCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_market_impact_calculation() {
        let model = AlmgrenChrissModel::new();
        let params = MarketImpactParams {
            pool_address: "0xabc".to_string(),
            dex: "uniswap-v3".to_string(),
            token_in: "ETH".to_string(),
            token_out: "USDC".to_string(),
            liquidity_depth_usd: 10_000_000.0,
            daily_volume_usd: 50_000_000.0,
            volatility: 0.02,
            spread_bps: 5.0,
            fee_bps: 30.0,
            block_number: 18000000,
            timestamp_ms: 1000000,
        };
        let trade = TradeCharacteristics {
            trade_size_usd: 100_000.0,
            is_buy: true,
            is_urgent: true,
            max_slippage_bps: 50.0,
        };
        let result = model.calculate_impact(&params, &trade);
        assert!(result.total_impact_usd > 0.0);
        assert!(result.total_impact_bps > 0.0);
    }

    #[test]
    fn test_optimal_trade_size() {
        let model = AlmgrenChrissModel::new();
        let params = MarketImpactParams {
            pool_address: "0xabc".to_string(),
            dex: "uniswap-v3".to_string(),
            token_in: "ETH".to_string(),
            token_out: "USDC".to_string(),
            liquidity_depth_usd: 10_000_000.0,
            daily_volume_usd: 50_000_000.0,
            volatility: 0.02,
            spread_bps: 5.0,
            fee_bps: 30.0,
            block_number: 18000000,
            timestamp_ms: 1000000,
        };
        let optimal = model.optimal_trade_size(&params, 50.0, true);
        assert!(optimal > 0.0);
    }

    #[test]
    fn test_flash_loan_impact_calculator() {
        let mut calc = FlashLoanImpactCalculator::new();
        let result = calc.calculate_flash_loan_impact("0xabc", 100.0, 3000.0);
        assert!(result.total_impact_usd > 0.0);
        assert!(result.confidence > 0.0);
    }
}
