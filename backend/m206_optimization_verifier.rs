// ==============================================================================
// M206: Optimization Bounds Formal Verifier
// Purpose: Formal verification of optimization parameter bounds
//          Ensures safety constraints are never violated during optimization
// CGM Subsystem: Optimization / Security
// ==============================================================================

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Parameter bound definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterBound {
    pub name: String,
    pub min: f64,
    pub max: f64,
    pub hard: bool, // Hard bounds cannot be violated; soft bounds can with penalty
    pub description: String,
}

impl ParameterBound {
    pub fn new(name: &str, min: f64, max: f64, hard: bool, description: &str) -> Self {
        Self {
            name: name.to_string(),
            min,
            max,
            hard,
            description: description.to_string(),
        }
    }

    pub fn hard(name: &str, min: f64, max: f64, description: &str) -> Self {
        Self::new(name, min, max, true, description)
    }

    pub fn soft(name: &str, min: f64, max: f64, description: &str) -> Self {
        Self::new(name, min, max, false, description)
    }

    pub fn is_satisfied(&self, value: f64) -> bool {
        value >= self.min && value <= self.max
    }

    pub fn violation_magnitude(&self, value: f64) -> f64 {
        if value < self.min {
            self.min - value
        } else if value > self.max {
            value - self.max
        } else {
            0.0
        }
    }
}

/// Optimization safety constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationConstraints {
    pub parameter_bounds: HashMap<String, ParameterBound>,
    pub max_daily_loss_eth: f64,
    pub max_position_size_eth: f64,
    pub max_flash_loan_size_eth: f64,
    pub min_profit_threshold_eth: f64,
    pub max_slippage_bps: u64,
    pub min_npm_floor: f64,
    pub max_consecutive_losses: u32,
}

impl Default for OptimizationConstraints {
    fn default() -> Self {
        let mut bounds = HashMap::new();
        bounds.insert("corridor_width".to_string(), ParameterBound::soft("corridor_width", 0.5, 2.0, "Corridor width multiplier"));
        bounds.insert("bribe_amount".to_string(), ParameterBound::soft("bribe_amount", 0.5, 2.0, "Bribe amount multiplier"));
        bounds.insert("block_phase".to_string(), ParameterBound::hard("block_phase", 0.0, 1.0, "Block phase timing [0-1]"));
        bounds.insert("bundle_size".to_string(), ParameterBound::soft("bundle_size", 0.5, 2.0, "Bundle size multiplier"));
        bounds.insert("flash_loan_size".to_string(), ParameterBound::soft("flash_loan_size", 0.5, 2.0, "Flash loan size multiplier"));
        bounds.insert("competitor_response".to_string(), ParameterBound::hard("competitor_response", 0.0, 1.0, "Competitor response [0-1]"));
        bounds.insert("regional_variant".to_string(), ParameterBound::soft("regional_variant", 0.5, 2.0, "Regional variant multiplier"));
        bounds.insert("route_efficiency".to_string(), ParameterBound::soft("route_efficiency", 0.5, 2.0, "Route efficiency multiplier"));
        bounds.insert("shield_routing".to_string(), ParameterBound::soft("shield_routing", 0.5, 2.0, "Shield routing multiplier"));
        bounds.insert("slippage_buffer".to_string(), ParameterBound::soft("slippage_buffer", 0.5, 2.0, "Slippage buffer multiplier"));
        bounds.insert("capital_allocation".to_string(), ParameterBound::soft("capital_allocation", 0.5, 2.0, "Capital allocation multiplier"));
        bounds.insert("multi_hop_legs".to_string(), ParameterBound::soft("multi_hop_legs", 0.5, 2.0, "Multi-hop legs multiplier"));
        bounds.insert("execution_timing".to_string(), ParameterBound::hard("execution_timing", 0.0, 1.0, "Execution timing [0-1]"));
        bounds.insert("gas_priority".to_string(), ParameterBound::hard("gas_priority", 0.0, 1.0, "Gas priority [0-1]"));
        bounds.insert("mev_protection".to_string(), ParameterBound::hard("mev_protection", 0.0, 1.0, "MEV protection level [0-1]"));
        bounds.insert("liquidity_depth_factor".to_string(), ParameterBound::soft("liquidity_depth_factor", 0.5, 2.0, "Liquidity depth factor"));
        bounds.insert("pool_tier_selection".to_string(), ParameterBound::soft("pool_tier_selection", 0.5, 2.0, "Pool tier selection"));
        bounds.insert("rpc_failover_threshold".to_string(), ParameterBound::hard("rpc_failover_threshold", 0.0, 1.0, "RPC failover threshold [0-1]"));
        bounds.insert("mempool_monitoring".to_string(), ParameterBound::hard("mempool_monitoring", 0.0, 1.0, "Mempool monitoring [0-1]"));
        bounds.insert("chain_selection".to_string(), ParameterBound::soft("chain_selection", 0.5, 2.0, "Chain selection multiplier"));
        bounds.insert("regional_routing".to_string(), ParameterBound::soft("regional_routing", 0.5, 2.0, "Regional routing multiplier"));
        bounds.insert("gas_cycle_phase".to_string(), ParameterBound::soft("gas_cycle_phase", 0.5, 2.0, "Gas cycle phase"));
        bounds.insert("runner_capacity".to_string(), ParameterBound::soft("runner_capacity", 0.5, 2.0, "Runner capacity multiplier"));
        bounds.insert("jit_liquidity_factor".to_string(), ParameterBound::soft("jit_liquidity_factor", 0.5, 2.0, "JIT liquidity factor"));
        bounds.insert("solver_tolerance".to_string(), ParameterBound::soft("solver_tolerance", 0.5, 2.0, "Solver tolerance"));

        Self {
            parameter_bounds: bounds,
            max_daily_loss_eth: 25_000.0,
            max_position_size_eth: 5_000_000.0,
            max_flash_loan_size_eth: 5_000_000.0,
            min_profit_threshold_eth: 0.001,
            max_slippage_bps: 50,
            min_npm_floor: 1.5,
            max_consecutive_losses: 5,
        }
    }
}

/// Verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub verified: bool,
    pub parameter_violations: Vec<ParameterViolation>,
    pub constraint_violations: Vec<ConstraintViolation>,
    pub safety_score: f64,
    pub recommendation: String,
}

/// Parameter bound violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterViolation {
    pub parameter: String,
    pub value: f64,
    pub bound_min: f64,
    pub bound_max: f64,
    pub is_hard: bool,
    pub violation_magnitude: f64,
}

/// Constraint violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintViolation {
    pub constraint: String,
    pub value: f64,
    pub limit: f64,
    pub severity: ViolationSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl ViolationSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "LOW",
            Self::Medium => "MEDIUM",
            Self::High => "HIGH",
            Self::Critical => "CRITICAL",
        }
    }
}

/// Optimization Bounds Formal Verifier
#[derive(Debug, Clone)]
pub struct OptimizationBoundsVerifier {
    pub constraints: OptimizationConstraints,
    pub verification_history: Vec<VerificationResult>,
    pub total_verifications: u64,
    pub total_violations: u64,
    pub hard_violations: u64,
}

impl OptimizationBoundsVerifier {
    pub fn new(constraints: OptimizationConstraints) -> Self {
        Self {
            constraints,
            verification_history: Vec::new(),
            total_verifications: 0,
            total_violations: 0,
            hard_violations: 0,
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(OptimizationConstraints::default())
    }

    /// Verify optimization parameters against bounds
    pub fn verify(&mut self, parameters: &HashMap<String, f64>) -> VerificationResult {
        self.total_verifications += 1;
        let mut parameter_violations = Vec::new();
        let mut constraint_violations = Vec::new();
        let mut hard_violation_count = 0;

        // Check parameter bounds
        for (name, bound) in &self.constraints.parameter_bounds {
            if let Some(&value) = parameters.get(name) {
                if !bound.is_satisfied(value) {
                    let violation = ParameterViolation {
                        parameter: name.clone(),
                        value,
                        bound_min: bound.min,
                        bound_max: bound.max,
                        is_hard: bound.hard,
                        violation_magnitude: bound.violation_magnitude(value),
                    };
                    parameter_violations.push(violation);
                    
                    if bound.hard {
                        hard_violation_count += 1;
                    }
                }
            }
        }

        // Check high-level constraints
        if let Some(&flash_loan_size) = parameters.get("flash_loan_size") {
            let scaled_size = flash_loan_size * self.constraints.max_flash_loan_size_eth;
            if scaled_size > self.constraints.max_flash_loan_size_eth {
                constraint_violations.push(ConstraintViolation {
                    constraint: "max_flash_loan_size".to_string(),
                    value: scaled_size,
                    limit: self.constraints.max_flash_loan_size_eth,
                    severity: ViolationSeverity::Critical,
                });
            }
        }

        if let Some(&slippage) = parameters.get("slippage_buffer") {
            let scaled_slippage = slippage * 100.0; // Approximate bps
            if scaled_slippage > self.constraints.max_slippage_bps as f64 {
                constraint_violations.push(ConstraintViolation {
                    constraint: "max_slippage_bps".to_string(),
                    value: scaled_slippage,
                    limit: self.constraints.max_slippage_bps as f64,
                    severity: ViolationSeverity::High,
                });
            }
        }

        // Calculate safety score
        let total_checks = self.constraints.parameter_bounds.len() + 4;
        let passed_checks = total_checks - parameter_violations.len() - constraint_violations.len();
        let safety_score = passed_checks as f64 / total_checks as f64;

        // Generate recommendation
        let recommendation = if hard_violation_count > 0 {
            "REJECT: Hard bound violations detected".to_string()
        } else if !parameter_violations.is_empty() {
            "ADJUST: Soft bound violations detected, apply penalty".to_string()
        } else if !constraint_violations.is_empty() {
            "REJECT: Constraint violations detected".to_string()
        } else {
            "APPROVE: All bounds satisfied".to_string()
        };

        let verified = hard_violation_count == 0 && constraint_violations.is_empty();

        let result = VerificationResult {
            verified,
            parameter_violations: parameter_violations.clone(),
            constraint_violations: constraint_violations.clone(),
            safety_score,
            recommendation,
        };

        // Update statistics
        self.total_violations += (parameter_violations.len() + constraint_violations.len()) as u64;
        self.hard_violations += hard_violation_count;
        self.verification_history.push(result.clone());

        // Limit history
        if self.verification_history.len() > 1000 {
            self.verification_history.remove(0);
        }

        result
    }

    /// Verify with automatic correction
    pub fn verify_and_correct(&mut self, parameters: &mut HashMap<String, f64>) -> VerificationResult {
        let result = self.verify(parameters);
        
        // Auto-correct soft bound violations
        for violation in &result.parameter_violations {
            if !violation.is_hard {
                let bound = &self.constraints.parameter_bounds[&violation.parameter];
                let corrected = violation.value.clamp(bound.min, bound.max);
                parameters.insert(violation.parameter.clone(), corrected);
                tracing::warn!(
                    "BoundsVerifier: Corrected {} from {} to {}",
                    violation.parameter,
                    violation.value,
                    corrected
                );
            }
        }

        result
    }

    /// Get bound for a parameter
    pub fn get_bound(&self, parameter: &str) -> Option<&ParameterBound> {
        self.constraints.parameter_bounds.get(parameter)
    }

    /// Update bound
    pub fn update_bound(&mut self, parameter: &str, bound: ParameterBound) {
        self.constraints.parameter_bounds.insert(parameter.to_string(), bound);
    }

    /// Get verification statistics
    pub fn get_statistics(&self) -> VerificationStatistics {
        let avg_safety_score = if self.verification_history.is_empty() {
            0.0
        } else {
            self.verification_history.iter()
                .map(|r| r.safety_score)
                .sum::<f64>() / self.verification_history.len() as f64
        };

        VerificationStatistics {
            total_verifications: self.total_verifications,
            total_violations: self.total_violations,
            hard_violations: self.hard_violations,
            avg_safety_score,
            violation_rate: if self.total_verifications > 0 {
                self.total_violations as f64 / self.total_verifications as f64
            } else {
                0.0
            },
        }
    }

    /// Reset verifier state
    pub fn reset(&mut self) {
        self.verification_history.clear();
        self.total_verifications = 0;
        self.total_violations = 0;
        self.hard_violations = 0;
    }
}

/// Verification statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VerificationStatistics {
    pub total_verifications: u64,
    pub total_violations: u64,
    pub hard_violations: u64,
    pub avg_safety_score: f64,
    pub violation_rate: f64,
}

impl Default for OptimizationBoundsVerifier {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parameter_bound() {
        let bound = ParameterBound::hard("test", 0.0, 1.0, "Test bound");
        assert!(bound.is_satisfied(0.5));
        assert!(!bound.is_satisfied(1.5));
        assert_eq!(bound.violation_magnitude(1.5), 0.5);
    }

    #[test]
    fn test_verification() {
        let mut verifier = OptimizationBoundsVerifier::with_defaults();
        let mut params = HashMap::new();
        params.insert("corridor_width".to_string(), 0.3); // Below min
        
        let result = verifier.verify(&params);
        assert!(!result.verified);
        assert!(!result.parameter_violations.is_empty());
    }

    #[test]
    fn test_verification_and_correction() {
        let mut verifier = OptimizationBoundsVerifier::with_defaults();
        let mut params = HashMap::new();
        params.insert("corridor_width".to_string(), 0.3); // Below min 0.5
        
        let result = verifier.verify_and_correct(&mut params);
        assert!(params["corridor_width"] >= 0.5);
    }

    #[test]
    fn test_safe_parameters() {
        let mut verifier = OptimizationBoundsVerifier::with_defaults();
        let mut params = HashMap::new();
        for (name, bound) in &verifier.constraints.parameter_bounds {
            params.insert(name.clone(), (bound.min + bound.max) / 2.0);
        }
        
        let result = verifier.verify(&params);
        assert!(result.verified);
        assert!(result.safety_score > 0.9);
    }
}
