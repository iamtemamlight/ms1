// ==============================================================================
// MODULE 54 AI SPECIALIST AGENT - Auto Optimization Specialist V2
// Purpose: Autonomous AI agent with Bayesian Optimization, Multi-Objective Pareto,
//          Predictive Gas, Market Impact, and Regime Detection
// 72-KPI DIMENSION: KPI-Driven Auto-Tuning with ML-based optimization
// ==============================================================================

use std::sync::atomic::{AtomicU64, Ordering};
use serde::{Deserialize, Serialize};
use crate::m200_bayesian_optimizer::{BayesianOptimizer, OptimizationContext};
use crate::m201_pareto_optimizer::{MultiObjectiveOptimizer, ObjectiveValues};
use crate::m202_gas_predictor::{PredictiveGasModel, GasParameters, GasStrategy};
use crate::m203_market_impact::FlashLoanImpactCalculator;
use crate::m204_regime_detector::{RegimeDetector, MarketRegime as DetectorMarketRegime};

#[derive(Default)]
pub struct SubcategoryMeasurements {
    pub alpha_profit_gain_30s: f64,
    pub alpha_trades_gain_30s: f64,
    pub velocity_latency_gain_30s: f64,
    pub velocity_throughput_gain_30s: f64,
    pub shield_violation_delta_30s: f64,
    pub efficiency_gas_save_30s: f64,
    pub continuity_sync_gain_30s: f64,
    pub market_opportunity_gain_30s: f64,
}

pub struct AutoOptimizationAgent {
    pub enabled: bool,
    pub profit_target: f64,
    pub npm_floor: f64,
    pub risk_mode: f64,
    pub optimization_cycles: u64,
    pub npm_violations: u64,
    pub last_action: String,
    pub kpi_drove_tune: bool,
    pub kpi_tune_cycles: u64,
    pub pillar_scores_scaled: [AtomicU64; 6],
    pub kpi_deviations_scaled: [AtomicU64; 72],
    pub predictive_trigger_active: bool,
    pub rapid_decline_flag: bool,
    pub last_profit_slope: f64,
    pub profit_per_minute_target: f64,
    pub profit_per_30s_target: f64,
    pub pillar_gains_30s: [AtomicU64; 6],
    pub subcategory_measurements: SubcategoryMeasurements,

    // V2: Bayesian Optimization engine
    pub bayesian_optimizer: BayesianOptimizer,
    pub pareto_optimizer: MultiObjectiveOptimizer,
    pub gas_predictor: PredictiveGasModel,
    pub impact_calculator: FlashLoanImpactCalculator,
    pub regime_detector: RegimeDetector,
    pub current_regime: DetectorMarketRegime,
    pub last_recommendation: Option<crate::m200_bayesian_optimizer::OptimizationResult>,
    pub optimization_mode: OptimizationMode,
    pub convergence_counter: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptimizationMode {
    RuleBased,      // Legacy KPI-driven tuning
    Bayesian,       // Bayesian Optimization
    Pareto,         // Multi-objective Pareto
    Hybrid,         // Bayesian + Pareto combined
}

impl OptimizationMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::RuleBased => "RULE_BASED",
            Self::Bayesian => "BAYESIAN",
            Self::Pareto => "PARETO",
            Self::Hybrid => "HYBRID",
        }
    }
}

impl AutoOptimizationAgent {
    pub fn new(profit_target: f64, npm_floor: f64, risk_mode: f64) -> Self {
        Self {
            enabled: true,
            profit_target,
            npm_floor,
            risk_mode,
            optimization_cycles: 0,
            npm_violations: 0,
            last_action: "IDLE".into(),
            kpi_drove_tune: false,
            kpi_tune_cycles: 0,
            pillar_scores_scaled: [
                AtomicU64::new(2500),
                AtomicU64::new(2000),
                AtomicU64::new(1500),
                AtomicU64::new(1500),
                AtomicU64::new(1000),
                AtomicU64::new(1500),
            ],
            kpi_deviations_scaled: [const { AtomicU64::new(0) }; 72],
            predictive_trigger_active: false,
            rapid_decline_flag: false,
            last_profit_slope: 0.0,
            profit_per_minute_target: profit_target / 1440.0,
            profit_per_30s_target: profit_target / 2880.0,
            pillar_gains_30s: [const { AtomicU64::new(0) }; 6],
            subcategory_measurements: SubcategoryMeasurements::default(),

            // V2 components
            bayesian_optimizer: BayesianOptimizer::for_trading_engine(),
            pareto_optimizer: MultiObjectiveOptimizer::for_trading(),
            gas_predictor: PredictiveGasModel::new(),
            impact_calculator: FlashLoanImpactCalculator::new(),
            regime_detector: RegimeDetector::new(),
            current_regime: DetectorMarketRegime::Sideways,
            last_recommendation: None,
            optimization_mode: OptimizationMode::Hybrid,
            convergence_counter: 0,
        }
    }

    // ========================================================================
    // V2: ML-Driven Optimization Entry Points
    // ========================================================================

    /// Run Bayesian Optimization cycle
    pub fn run_bayesian_optimization(&mut self, ctx: &OptimizationContext) -> crate::m200_bayesian_optimizer::OptimizationResult {
        let result = self.bayesian_optimizer.recommend(ctx);
        self.last_recommendation = Some(result.clone());
        
        if self.bayesian_optimizer.is_converged() {
            self.convergence_counter += 1;
            tracing::info!("BO: Converged {} cycles", self.convergence_counter);
        }
        
        result
    }

    /// Run Multi-Objective Pareto optimization
    pub fn run_pareto_optimization(
        &mut self,
        parameters: Vec<f64>,
        objectives: ObjectiveValues,
    ) -> ParetoResult {
        let solution = self.pareto_optimizer.evaluate(parameters, objectives, 0.0);
        let best = self.pareto_optimizer.select_best();
        
        ParetoResult {
            solution,
            best_solution: best.cloned(),
            pareto_size: self.pareto_optimizer.front_size(),
            converged: self.pareto_optimizer.is_converged(),
        }
    }

    /// Get gas prediction for current market conditions
    pub fn predict_gas(&mut self, gas_params: &GasParameters, horizon_blocks: usize) -> crate::m202_gas_predictor::GasPricePrediction {
        self.gas_predictor.record_gas(gas_params);
        self.gas_predictor.predict(horizon_blocks)
    }

    /// Calculate market impact for flash loan
    pub fn calculate_flash_loan_impact(
        &mut self,
        pool_address: &str,
        loan_size_eth: f64,
        eth_price_usd: f64,
    ) -> crate::m203_market_impact::MarketImpactResult {
        self.impact_calculator.calculate_flash_loan_impact(pool_address, loan_size_eth, eth_price_usd)
    }

    /// Detect current market regime
    pub fn detect_regime(&mut self, features: crate::m204_regime_detector::MarketFeatures, current_block: u64) -> crate::m204_regime_detector::RegimeDetectionResult {
        let result = self.regime_detector.observe(features, current_block);
        self.current_regime = result.current_regime;
        result
    }

    /// Run full optimization cycle (V2 enhanced)
    pub fn run_optimization_cycle_v2(
        &mut self,
        kpi_scores: &[f64; 72],
        profit_per_30s: f64,
        gas_params: &GasParameters,
        market_features: &crate::m204_regime_detector::MarketFeatures,
        current_block: u64,
    ) -> OptimizationCycleResult {
        self.optimization_cycles += 1;

        // 1. Detect regime
        let regime_result = self.detect_regime(market_features.clone(), current_block);
        let ctx = OptimizationContext::from_regime_result(&regime_result);

        // 2. Run Bayesian Optimization
        let bo_result = self.run_bayesian_optimization(&ctx);

        // 3. Run Pareto optimization with objectives
        let objectives = ObjectiveValues::new(
            profit_per_30s * 2880.0, // Daily profit
            self.npm_floor - self.npm_violations as f64 * 0.01, // Risk proxy
            0.0198, // Latency (us)
            0.95,   // Capital efficiency
            0.1,    // MEV extraction
        );
        let pareto_result = self.run_pareto_optimization(
            bo_result.recommendation.parameters.clone(),
            objectives,
        );

        // 4. Predict gas costs
        let gas_prediction = self.predict_gas(gas_params, 12);

        // 5. Update Bayesian optimizer with outcome
        let objective_value = self.composite_objective(profit_per_30s, &gas_prediction, &regime_result);
        self.bayesian_optimizer.observe(
            bo_result.recommendation.parameters.clone(),
            objective_value,
            &ctx,
        );

        // 6. Determine action
        let action = self.determine_action(&bo_result, &pareto_result, &gas_prediction);

        OptimizationCycleResult {
            cycle: self.optimization_cycles,
            regime: regime_result.current_regime,
            regime_confidence: regime_result.confidence,
            bayesian_result: bo_result.clone(),
            pareto_result,
            gas_prediction,
            action,
            parameters: bo_result.recommendation.parameters.clone(),
            mode: self.optimization_mode,
        }
    }

    /// Compute composite objective value for BO
    fn composite_objective(&self, profit_per_30s: f64, gas: &crate::m202_gas_predictor::GasPricePrediction, regime: &crate::m204_regime_detector::RegimeDetectionResult) -> f64 {
        let profit_component = profit_per_30s * 2880.0; // Daily profit
        let gas_penalty = gas.expected_cost_eth * 100.0; // Scale gas cost
        let risk_penalty = self.npm_violations as f64 * 0.1;
        let regime_bonus = regime.recommended_weights.exploitation * 0.5;
        
        profit_component - gas_penalty - risk_penalty + regime_bonus
    }

    /// Determine action based on optimization results
    fn determine_action(
        &self,
        bo: &crate::m200_bayesian_optimizer::OptimizationResult,
        pareto: &ParetoResult,
        gas: &crate::m202_gas_predictor::GasPricePrediction,
    ) -> String {
        if gas.recommended_strategy == crate::m202_gas_predictor::GasStrategy::Skip {
            return "SKIP_TRADES".to_string();
        }
        if bo.regime_shift_detected {
            return "REGIME_ADAPT".to_string();
        }
        if bo.gp_uncertainty > 0.4 {
            return "EXPLORE".to_string();
        }
        if pareto.pareto_size < 3 {
            return "EXPLORE".to_string();
        }
        "EXECUTE".to_string()
    }

    // ========================================================================
    // Legacy Compatibility: Rule-Based Methods (preserved for backward compat)
    // ========================================================================

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn get_npm_floor(&self) -> f64 {
        self.npm_floor
    }

    pub fn get_profit_target(&self) -> f64 {
        self.profit_target
    }

    pub fn get_risk_mode(&self) -> f64 {
        self.risk_mode
    }

    pub fn adjust_npm_for_risk(&mut self) -> f64 {
        match self.risk_mode as u64 {
            0 => self.npm_floor * 1.5,
            2 => self.npm_floor * 0.8,
            _ => self.npm_floor,
        }
    }

    pub fn check_npm_compliance(&mut self, current_npm: f64) -> bool {
        let adjusted_floor = self.adjust_npm_for_risk();
        if current_npm < adjusted_floor {
            self.npm_violations += 1;
            false
        } else {
            true
        }
    }

    pub fn optimize_dimension_from_kpi(&mut self, dimension_index: usize, kpi_score_pct: f64) -> f64 {
        let base_adjustment = 1.0;
        let kpi_factor = (kpi_score_pct / 100.0).max(0.5).min(1.5);
        match dimension_index {
            0..=24 => base_adjustment * kpi_factor,
            _ => base_adjustment,
        }
    }

    pub fn run_kpi_autotune(&mut self) -> Vec<(usize, f64)> {
        self.kpi_tune_cycles += 1;
        self.kpi_drove_tune = true;
        let mut adjustments = Vec::new();
        for i in 0..6 {
            let score = self.pillar_scores_scaled[i].load(Ordering::Relaxed) as f64 / 100.0;
            let adj = self.optimize_dimension_from_kpi(i, score);
            adjustments.push((i, adj));
        }
        for i in 6..25 {
            let adj = self.optimize_dimension_from_kpi(i, 1.0);
            adjustments.push((i, adj));
        }
        adjustments
    }

    pub fn evaluate_profit_gap(&self, current_daily_alpha: f64) -> f64 {
        (self.profit_target - current_daily_alpha) / self.profit_target
    }

    pub fn evaluate_realtime_profit_gap(&self, profit_per_minute: f64) -> f64 {
        (self.profit_per_minute_target - profit_per_minute) / self.profit_per_minute_target
    }

    pub fn evaluate_30s_profit_gap(&self, profit_per_30s: f64) -> f64 {
        (self.profit_per_30s_target - profit_per_30s) / self.profit_per_30s_target
    }

    pub fn update_kpi_deviation(&mut self, kpi_index: usize, deviation_scaled: u64) {
        if kpi_index < 72 {
            self.kpi_deviations_scaled[kpi_index].store(deviation_scaled, Ordering::Relaxed);
        }
    }

    pub fn detect_rapid_decline(&mut self, current_slope: f64, baseline_slope: f64) -> bool {
        let decline_rate = (baseline_slope - current_slope) / baseline_slope.max(0.001);
        self.rapid_decline_flag = decline_rate > 0.15;
        self.rapid_decline_flag
    }

    pub fn signal_alpha_copilot(&mut self, profit_gap_pct: f64, realtime_gap_pct: f64) -> &'static str {
        if realtime_gap_pct > 0.10 { "ADJUST_STRATEGY" }
        else if profit_gap_pct > 0.20 { "ADJUST_STRATEGY" }
        else if self.rapid_decline_flag && realtime_gap_pct > 0.05 { "PREEMPTIVE_REBALANCE" }
        else if self.npm_violations > 10 { "REDUCE_EXPOSURE" }
        else { "MAINTAIN" }
    }

    pub fn get_dimension_adjustment_for_kpi(&self, kpi_index: usize, deviation_pct: f64) -> (usize, f64) {
        let dimension = match kpi_index {
            0..=11 => self.map_alpha_kpi_to_dimension(kpi_index, deviation_pct),
            12..=23 => self.map_velocity_kpi_to_dimension(kpi_index, deviation_pct),
            24..=35 => self.map_shield_kpi_to_dimension(kpi_index, deviation_pct),
            36..=47 => self.map_efficiency_kpi_to_dimension(kpi_index, deviation_pct),
            48..=59 => self.map_continuity_kpi_to_dimension(kpi_index, deviation_pct),
            60..=71 => self.map_market_kpi_to_dimension(kpi_index, deviation_pct),
            _ => (0, 1.0),
        };
        dimension
    }

    pub fn update_pillar_gain_30s(&mut self, pillar_index: usize, gain_pct_scaled: u64) {
        if pillar_index < 6 {
            self.pillar_gains_30s[pillar_index].store(gain_pct_scaled, Ordering::Relaxed);
        }
    }

    pub fn update_subcategory_measurements(&mut self, measurements: SubcategoryMeasurements) {
        self.subcategory_measurements = measurements;
    }

    pub fn get_subcategory_measurements(&self) -> &SubcategoryMeasurements {
        &self.subcategory_measurements
    }

    pub fn get_pillar_gains_30s(&self) -> [f64; 6] {
        [
            self.pillar_gains_30s[0].load(Ordering::Relaxed) as f64 / 100.0,
            self.pillar_gains_30s[1].load(Ordering::Relaxed) as f64 / 100.0,
            self.pillar_gains_30s[2].load(Ordering::Relaxed) as f64 / 100.0,
            self.pillar_gains_30s[3].load(Ordering::Relaxed) as f64 / 100.0,
            self.pillar_gains_30s[4].load(Ordering::Relaxed) as f64 / 100.0,
            self.pillar_gains_30s[5].load(Ordering::Relaxed) as f64 / 100.0,
        ]
    }

    fn map_alpha_kpi_to_dimension(&self, kpi: usize, deviation: f64) -> (usize, f64) {
        match kpi {
            0..=2 => (0, 0.8 + deviation * 0.4),
            3..=5 => (1, 1.0 + deviation * 0.25),
            6..=8 => (3, 0.9 + deviation * 0.2),
            9..=11 => (4, 1.0 + deviation * 0.35),
            _ => (0, 1.0),
        }
    }

    fn map_velocity_kpi_to_dimension(&self, kpi: usize, deviation: f64) -> (usize, f64) {
        match kpi {
            12..=15 => (2, 0.9 + deviation * 0.2),
            16..=19 => (24, 1.0 + deviation * 0.15),
            20..=23 => (23, 1.0 + deviation * 0.25),
            _ => (7, 1.0 + deviation * 0.1),
        }
    }

    fn map_shield_kpi_to_dimension(&self, kpi: usize, deviation: f64) -> (usize, f64) {
        match kpi {
            24..=27 => (8, 1.1 + deviation * 0.3),
            28..=31 => (16, 0.95 + deviation * 0.25),
            32..=35 => (6, 1.0 + deviation * 0.2),
            _ => (8, 1.0),
        }
    }

    fn map_efficiency_kpi_to_dimension(&self, kpi: usize, deviation: f64) -> (usize, f64) {
        match kpi {
            36..=38 => (10, 1.0 + deviation * 0.2),
            39 => (11, 1.0 + deviation * 0.15),
            40..=47 => (21, 0.9 + deviation * 0.25),
            _ => (9, 1.0 + deviation * 0.1),
        }
    }

    fn map_continuity_kpi_to_dimension(&self, kpi: usize, deviation: f64) -> (usize, f64) {
        match kpi {
            48..=51 => (22, 1.0 + deviation * 0.25),
            52..=59 => (20, 1.0 + deviation * 0.2),
            _ => (18, 1.0),
        }
    }

    fn map_market_kpi_to_dimension(&self, kpi: usize, deviation: f64) -> (usize, f64) {
        match kpi {
            60..=63 => (2, 1.0 + deviation * 0.15),
            64..=67 => (1, 1.0 + deviation * 0.2),
            68..=71 => (3, 1.0 + deviation * 0.1),
            _ => (0, 1.0),
        }
    }
}

/// Result of a full optimization cycle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationCycleResult {
    pub cycle: u64,
    pub regime: DetectorMarketRegime,
    pub regime_confidence: f64,
    pub bayesian_result: crate::m200_bayesian_optimizer::OptimizationResult,
    pub pareto_result: ParetoResult,
    pub gas_prediction: crate::m202_gas_predictor::GasPricePrediction,
    pub action: String,
    pub parameters: Vec<f64>,
    pub mode: OptimizationMode,
}

/// Result of Pareto optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParetoResult {
    pub solution: crate::m201_pareto_optimizer::ParetoSolution,
    pub best_solution: Option<crate::m201_pareto_optimizer::ParetoSolution>,
    pub pareto_size: usize,
    pub converged: bool,
}

// ==============================================================================
// Implementation of OptimizationContext constructor helpers
// ==============================================================================

impl OptimizationContext {
    fn detector_to_bayesian(regime: DetectorMarketRegime) -> crate::m200_bayesian_optimizer::MarketRegime {
        match regime {
            DetectorMarketRegime::Bull => crate::m200_bayesian_optimizer::MarketRegime::Bull,
            DetectorMarketRegime::Bear => crate::m200_bayesian_optimizer::MarketRegime::Bear,
            DetectorMarketRegime::Sideways => crate::m200_bayesian_optimizer::MarketRegime::Sideways,
            DetectorMarketRegime::Volatile => crate::m200_bayesian_optimizer::MarketRegime::Volatile,
            DetectorMarketRegime::Crash => crate::m200_bayesian_optimizer::MarketRegime::Crash,
            DetectorMarketRegime::Recovery => crate::m200_bayesian_optimizer::MarketRegime::Recovery,
        }
    }

    pub fn from_regime_result(regime: &crate::m204_regime_detector::RegimeDetectionResult) -> Self {
        Self {
            regime: Self::detector_to_bayesian(regime.current_regime),
            network_congestion: 0.3,
            gas_price_gwei: 25.0,
            pool_liquidity_usd: 5_000_000.0,
            fleet_size: 100,
            active_runners: 95,
            volatility_index: 0.2,
            block_height: 18000000,
        }
    }
}
