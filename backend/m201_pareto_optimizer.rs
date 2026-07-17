// ==============================================================================
// M201: Multi-Objective Pareto Optimizer
// Purpose: Pareto frontier optimization for multi-objective trading decisions
//          Handles profit, risk, latency, capital efficiency, MEV extraction
// CGM Subsystem: Optimization / Profit
// ==============================================================================

use std::collections::HashMap;
use std::f64::{INFINITY, NEG_INFINITY};
use serde::{Deserialize, Serialize};

/// Number of objectives for multi-objective optimization
pub const NUM_OBJECTIVES: usize = 5;

/// Objective identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ObjectiveId {
    Profit,          // Maximize net profit
    Risk,            // Minimize risk (CVaR, drawdown)
    Latency,         // Minimize execution latency
    CapitalEfficiency, // Maximize capital efficiency
    MevExtraction,   // Maximize MEV extraction (ethical)
}

impl ObjectiveId {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Profit => "PROFIT",
            Self::Risk => "RISK",
            Self::Latency => "LATENCY",
            Self::CapitalEfficiency => "CAPITAL_EFFICIENCY",
            Self::MevExtraction => "MEV_EXTRACTION",
        }
    }

    pub fn all() -> [Self; 5] {
        [
            Self::Profit,
            Self::Risk,
            Self::Latency,
            Self::CapitalEfficiency,
            Self::MevExtraction,
        ]
    }
}

/// Objective weights for weighted sum approach
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectiveWeights {
    pub profit: f64,
    pub risk: f64,
    pub latency: f64,
    pub capital_efficiency: f64,
    pub mev_extraction: f64,
}

impl Default for ObjectiveWeights {
    fn default() -> Self {
        Self {
            profit: 0.40,
            risk: 0.25,
            latency: 0.15,
            capital_efficiency: 0.15,
            mev_extraction: 0.05,
        }
    }
}

impl ObjectiveWeights {
    pub fn normalize(&self) -> Self {
        let sum = self.profit + self.risk + self.latency + self.capital_efficiency + self.mev_extraction;
        if sum > 0.0 {
            Self {
                profit: self.profit / sum,
                risk: self.risk / sum,
                latency: self.latency / sum,
                capital_efficiency: self.capital_efficiency / sum,
                mev_extraction: self.mev_extraction / sum,
            }
        } else {
            Self::default()
        }
    }
}

/// Objective values for a solution
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ObjectiveValues {
    pub profit: f64,
    pub risk: f64,
    pub latency: f64,
    pub capital_efficiency: f64,
    pub mev_extraction: f64,
}

impl ObjectiveValues {
    pub fn new(profit: f64, risk: f64, latency: f64, capital_efficiency: f64, mev_extraction: f64) -> Self {
        Self {
            profit,
            risk,
            latency,
            capital_efficiency,
            mev_extraction,
        }
    }

    pub fn as_vec(&self) -> Vec<f64> {
        vec![
            self.profit,
            self.risk,
            self.latency,
            self.capital_efficiency,
            self.mev_extraction,
        ]
    }

    /// Convert to vector for Pareto operations
    pub fn to_vector(&self) -> Vec<f64> {
        vec![
            self.profit,
            self.risk,
            self.latency,
            self.capital_efficiency,
            self.mev_extraction,
        ]
    }

    /// Create from vector
    pub fn from_vector(v: &[f64]) -> Self {
        Self {
            profit: v.get(0).copied().unwrap_or(0.0),
            risk: v.get(1).copied().unwrap_or(0.0),
            latency: v.get(2).copied().unwrap_or(0.0),
            capital_efficiency: v.get(3).copied().unwrap_or(0.0),
            mev_extraction: v.get(4).copied().unwrap_or(0.0),
        }
    }

    /// Weighted sum scalarization
    pub fn weighted_sum(&self, weights: &ObjectiveWeights) -> f64 {
        let w = weights.normalize();
        w.profit * self.profit
            + w.risk * (1.0 - self.risk) // Minimize risk
            + w.latency * (1.0 / (1.0 + self.latency)) // Minimize latency
            + w.capital_efficiency * self.capital_efficiency
            + w.mev_extraction * self.mev_extraction
    }

    /// Normalize objectives to [0, 1] range
    pub fn normalize(&self, bounds: &ObjectiveBounds) -> Self {
        Self {
            profit: (self.profit - bounds.profit_min) / (bounds.profit_max - bounds.profit_min + 1e-9),
            risk: (self.risk - bounds.risk_min) / (bounds.risk_max - bounds.risk_min + 1e-9),
            latency: (self.latency - bounds.latency_min) / (bounds.latency_max - bounds.latency_min + 1e-9),
            capital_efficiency: (self.capital_efficiency - bounds.capital_efficiency_min)
                / (bounds.capital_efficiency_max - bounds.capital_efficiency_min + 1e-9),
            mev_extraction: (self.mev_extraction - bounds.mev_extraction_min)
                / (bounds.mev_extraction_max - bounds.mev_extraction_min + 1e-9),
        }
    }
}

/// Objective value bounds for normalization
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ObjectiveBounds {
    pub profit_min: f64,
    pub profit_max: f64,
    pub risk_min: f64,
    pub risk_max: f64,
    pub latency_min: f64,
    pub latency_max: f64,
    pub capital_efficiency_min: f64,
    pub capital_efficiency_max: f64,
    pub mev_extraction_min: f64,
    pub mev_extraction_max: f64,
}

impl ObjectiveBounds {
    pub fn update(&mut self, values: &ObjectiveValues) {
        self.profit_min = self.profit_min.min(values.profit);
        self.profit_max = self.profit_max.max(values.profit);
        self.risk_min = self.risk_min.min(values.risk);
        self.risk_max = self.risk_max.max(values.risk);
        self.latency_min = self.latency_min.min(values.latency);
        self.latency_max = self.latency_max.max(values.latency);
        self.capital_efficiency_min = self.capital_efficiency_min.min(values.capital_efficiency);
        self.capital_efficiency_max = self.capital_efficiency_max.max(values.capital_efficiency);
        self.mev_extraction_min = self.mev_extraction_min.min(values.mev_extraction);
        self.mev_extraction_max = self.mev_extraction_max.max(values.mev_extraction);
    }

    pub fn expand(&mut self, other: &ObjectiveBounds) {
        self.profit_min = self.profit_min.min(other.profit_min);
        self.profit_max = self.profit_max.max(other.profit_max);
        self.risk_min = self.risk_min.min(other.risk_min);
        self.risk_max = self.risk_max.max(other.risk_max);
        self.latency_min = self.latency_min.min(other.latency_min);
        self.latency_max = self.latency_max.max(other.latency_max);
        self.capital_efficiency_min = self.capital_efficiency_min.min(other.capital_efficiency_min);
        self.capital_efficiency_max = self.capital_efficiency_max.max(other.capital_efficiency_max);
        self.mev_extraction_min = self.mev_extraction_min.min(other.mev_extraction_min);
        self.mev_extraction_max = self.mev_extraction_max.max(other.mev_extraction_max);
    }
}

/// Pareto solution with parameters and objectives
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParetoSolution {
    pub id: u64,
    pub parameters: Vec<f64>,
    pub objectives: ObjectiveValues,
    pub normalized_objectives: Option<ObjectiveValues>,
    pub domination_rank: usize,
    pub crowding_distance: f64,
    pub constraint_violation: f64,
    pub timestamp_ms: u64,
}

impl ParetoSolution {
    pub fn new(id: u64, parameters: Vec<f64>, objectives: ObjectiveValues) -> Self {
        Self {
            id,
            parameters,
            objectives,
            normalized_objectives: None,
            domination_rank: 0,
            crowding_distance: 0.0,
            constraint_violation: 0.0,
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
        }
    }

    /// Check if this solution dominates another (all objectives better or equal, at least one strictly better)
    pub fn dominates(&self, other: &Self, _weights: &ObjectiveWeights) -> bool {
        let a = self.objectives.to_vector();
        let b = other.objectives.to_vector();

        let mut at_least_one_better = false;
        for i in 0..a.len() {
            let a_better = match i {
                0 | 3 | 4 => a[i] >= b[i], // Maximize: profit, capital_efficiency, mev_extraction
                1 | 2 => a[i] <= b[i],      // Minimize: risk, latency
                _ => a[i] >= b[i],
            };
            if a[i] != b[i] {
                if !a_better {
                    return false;
                }
                at_least_one_better = true;
            }
        }
        at_least_one_better
    }

    /// Compute constraint violation (0 = feasible)
    pub fn compute_constraint_violation(&self, bounds: &ObjectiveBounds) -> f64 {
        let mut violation = 0.0;
        if self.objectives.profit < bounds.profit_min {
            violation += (bounds.profit_min - self.objectives.profit).abs();
        }
        if self.objectives.risk > bounds.risk_max {
            violation += (self.objectives.risk - bounds.risk_max).abs();
        }
        if self.objectives.latency > bounds.latency_max {
            violation += (self.objectives.latency - bounds.latency_max).abs();
        }
        violation
    }
}

/// Pareto front with non-dominated solutions
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ParetoFront {
    pub solutions: Vec<ParetoSolution>,
    pub bounds: ObjectiveBounds,
    pub generation: u64,
}

impl ParetoFront {
    pub fn new() -> Self {
        Self {
            solutions: Vec::new(),
            bounds: ObjectiveBounds::default(),
            generation: 0,
        }
    }

    /// Add solution and update Pareto front
    pub fn add_solution(&mut self, solution: ParetoSolution) {
        self.bounds.update(&solution.objectives);
        self.solutions.push(solution);
    }

    /// Compute non-dominated sorting (NSGA-II style)
    pub fn compute_pareto_fronts(&mut self) {
        let n = self.solutions.len();
        if n == 0 {
            return;
        }

        // Normalize objectives
        for sol in &mut self.solutions {
            sol.normalized_objectives = Some(sol.objectives.normalize(&self.bounds));
        }

        // Non-dominated sorting
        let mut fronts: Vec<Vec<usize>> = Vec::new();
        let mut is_dominated = vec![false; n];
        let mut domination_count = vec![0; n];
        let mut dominated_solutions = vec![Vec::new(); n];

        for i in 0..n {
            for j in 0..n {
                if i == j {
                    continue;
                }
                if self.solutions[i].dominates(&self.solutions[j], &ObjectiveWeights::default()) {
                    dominated_solutions[i].push(j);
                } else if self.solutions[j].dominates(&self.solutions[i], &ObjectiveWeights::default()) {
                    domination_count[i] += 1;
                }
            }
            if domination_count[i] == 0 {
                is_dominated[i] = false;
                if fronts.is_empty() {
                    fronts.push(vec![i]);
                } else {
                    fronts[0].push(i);
                }
            }
        }

        let mut front_idx = 0;
        while front_idx < fronts.len() {
            let mut next_front = Vec::new();
            for &i in &fronts[front_idx] {
                self.solutions[i].domination_rank = front_idx;
                for &j in &dominated_solutions[i] {
                    domination_count[j] -= 1;
                    if domination_count[j] == 0 {
                        next_front.push(j);
                    }
                }
            }
            if !next_front.is_empty() {
                fronts.push(next_front);
            }
            front_idx += 1;
        }

        // Compute crowding distance for each front
        for front in &fronts {
            self.compute_crowding_distance(front);
        }

        self.generation += 1;
    }

    /// Compute crowding distance for diversity preservation
    fn compute_crowding_distance(&mut self, front: &[usize]) {
        let n = front.len();
        if n <= 2 {
            for &idx in front {
                self.solutions[idx].crowding_distance = INFINITY;
            }
            return;
        }

        let mut distances = vec![0.0; n];
        let objectives = ObjectiveId::all();

        for obj in objectives {
            // Sort front by objective value
            let mut sorted: Vec<usize> = front.to_vec();
            sorted.sort_by(|&a, &b| {
                let val_a = self.solutions[a].objectives.as_vec()[Self::objective_index(obj)];
                let val_b = self.solutions[b].objectives.as_vec()[Self::objective_index(obj)];
                val_a.partial_cmp(&val_b).unwrap_or(std::cmp::Ordering::Equal)
            });

            let min_val = self.solutions[sorted[0]].objectives.as_vec()[Self::objective_index(obj)];
            let max_val = self.solutions[sorted[n - 1]].objectives.as_vec()[Self::objective_index(obj)];
            let range = max_val - min_val;

            if range > 0.0 {
                distances[sorted[0]] = INFINITY;
                distances[sorted[n - 1]] = INFINITY;
                for i in 1..n - 1 {
                    let idx = sorted[i];
                    let prev_val = self.solutions[sorted[i - 1]].objectives.as_vec()[Self::objective_index(obj)];
                    let next_val = self.solutions[sorted[i + 1]].objectives.as_vec()[Self::objective_index(obj)];
                    distances[idx] += (next_val - prev_val) / range;
                }
            }
        }

        for (i, &idx) in front.iter().enumerate() {
            self.solutions[idx].crowding_distance = distances[i];
        }
    }

    /// Get Pareto-optimal solutions (first front)
    pub fn pareto_optimal(&self) -> Vec<&ParetoSolution> {
        self.solutions.iter()
            .filter(|s| s.domination_rank == 0)
            .collect()
    }

    /// Select solution using weighted Tchebycheff approach
    pub fn select_weighted_tchebycheff(&self, weights: &ObjectiveWeights) -> Option<&ParetoSolution> {
        let w = weights.normalize();
        let mut best_idx = None;
        let mut best_value = NEG_INFINITY;

        for (idx, sol) in self.solutions.iter().enumerate() {
            if sol.domination_rank > 0 {
                continue;
            }
            let normalized = sol.objectives.normalize(&self.bounds);
            let profit_term = w.profit * normalized.profit;
            let risk_term = w.risk * (1.0 - normalized.risk);
            let latency_term = w.latency * (1.0 - normalized.latency);
            let capital_term = w.capital_efficiency * normalized.capital_efficiency;
            let mev_term = w.mev_extraction * normalized.mev_extraction;
            let value = profit_term.min(risk_term).min(latency_term).min(capital_term).min(mev_term);
            
            if value > best_value {
                best_value = value;
                best_idx = Some(idx);
            }
        }

        best_idx.and_then(|i| self.solutions.get(i))
    }

    /// Select solution by minimizing distance to ideal point
    pub fn select_ideal_point(&self, weights: &ObjectiveWeights) -> Option<&ParetoSolution> {
        let w = weights.normalize();
        let mut best_idx = None;
        let mut best_distance = INFINITY;

        for (idx, sol) in self.solutions.iter().enumerate() {
            if sol.domination_rank > 0 {
                continue;
            }
            let normalized = sol.objectives.normalize(&self.bounds);
            let ideal_profit = 1.0;
            let ideal_risk = 0.0;
            let ideal_latency = 0.0;
            let ideal_capital = 1.0;
            let ideal_mev = 1.0;

            let dist = w.profit * (normalized.profit - ideal_profit).powi(2)
                + w.risk * (normalized.risk - ideal_risk).powi(2)
                + w.latency * (normalized.latency - ideal_latency).powi(2)
                + w.capital_efficiency * (normalized.capital_efficiency - ideal_capital).powi(2)
                + w.mev_extraction * (normalized.mev_extraction - ideal_mev).powi(2);

            if dist < best_distance {
                best_distance = dist;
                best_idx = Some(idx);
            }
        }

        best_idx.and_then(|i| self.solutions.get(i))
    }

    /// Get objective values as vector
    fn objective_index(obj: ObjectiveId) -> usize {
        match obj {
            ObjectiveId::Profit => 0,
            ObjectiveId::Risk => 1,
            ObjectiveId::Latency => 2,
            ObjectiveId::CapitalEfficiency => 3,
            ObjectiveId::MevExtraction => 4,
        }
    }

    /// Prune solutions to maintain diversity
    pub fn prune(&mut self, max_solutions: usize) {
        if self.solutions.len() <= max_solutions {
            return;
        }

        // Keep solutions from better fronts, then by crowding distance
        self.solutions.sort_by(|a, b| {
            a.domination_rank.cmp(&b.domination_rank)
                .then(b.crowding_distance.partial_cmp(&a.crowding_distance).unwrap_or(std::cmp::Ordering::Equal))
        });

        self.solutions.truncate(max_solutions);
    }
}

/// Multi-Objective Optimizer
#[derive(Debug, Clone)]
pub struct MultiObjectiveOptimizer {
    pub pareto_front: ParetoFront,
    pub weights: ObjectiveWeights,
    pub max_solutions: usize,
    pub total_evaluations: u64,
    pub convergence_history: Vec<f64>,
}

impl MultiObjectiveOptimizer {
    pub fn new(max_solutions: usize) -> Self {
        Self {
            pareto_front: ParetoFront::new(),
            weights: ObjectiveWeights::default(),
            max_solutions,
            total_evaluations: 0,
            convergence_history: Vec::new(),
        }
    }

    /// Create optimizer with trading-specific weights
    pub fn for_trading() -> Self {
        Self {
            pareto_front: ParetoFront::new(),
            weights: ObjectiveWeights {
                profit: 0.40,
                risk: 0.30,
                latency: 0.15,
                capital_efficiency: 0.10,
                mev_extraction: 0.05,
            },
            max_solutions: 100,
            total_evaluations: 0,
            convergence_history: Vec::new(),
        }
    }

    /// Evaluate a solution and add to Pareto front
    pub fn evaluate(
        &mut self,
        parameters: Vec<f64>,
        objectives: ObjectiveValues,
        constraint_violation: f64,
    ) -> ParetoSolution {
        self.total_evaluations += 1;
        let id = self.total_evaluations;
        let mut solution = ParetoSolution::new(id, parameters, objectives);
        solution.constraint_violation = constraint_violation;

        self.pareto_front.add_solution(solution.clone());
        self.pareto_front.compute_pareto_fronts();

        // Track convergence (hypervolume approximation)
        let hypervolume = self.approximate_hypervolume();
        self.convergence_history.push(hypervolume);
        if self.convergence_history.len() > 100 {
            self.convergence_history.remove(0);
        }

        // Prune if needed
        self.pareto_front.prune(self.max_solutions);

        solution
    }

    /// Approximate hypervolume of Pareto front (2D projection)
    fn approximate_hypervolume(&self) -> f64 {
        let pareto = self.pareto_front.pareto_optimal();
        if pareto.is_empty() {
            return 0.0;
        }

        // Sort by profit (descending)
        let mut sorted: Vec<f64> = pareto.iter()
            .map(|s| s.objectives.profit)
            .collect();
        sorted.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));

        // Approximate area under profit curve
        let mut area = 0.0;
        for i in 0..sorted.len().saturating_sub(1) {
            area += (sorted[i] + sorted[i + 1]) / 2.0;
        }
        area
    }

    /// Select best solution using current weights
    pub fn select_best(&self) -> Option<&ParetoSolution> {
        self.pareto_front.select_weighted_tchebycheff(&self.weights)
    }

    /// Select solution closest to ideal point
    pub fn select_ideal(&self) -> Option<&ParetoSolution> {
        self.pareto_front.select_ideal_point(&self.weights)
    }

    /// Update objective weights
    pub fn set_weights(&mut self, weights: ObjectiveWeights) {
        self.weights = weights.normalize();
    }

    /// Get Pareto front size
    pub fn front_size(&self) -> usize {
        self.pareto_front.pareto_optimal().len()
    }

    /// Check convergence
    pub fn is_converged(&self) -> bool {
        if self.convergence_history.len() < 20 {
            return false;
        }
        let recent: f64 = self.convergence_history.iter().rev().take(10).sum::<f64>() / 10.0;
        let older: f64 = self.convergence_history.iter().take(10).sum::<f64>() / 10.0;
        (older - recent).abs() < 0.001
    }

    /// Get diversity metric (average crowding distance on Pareto front)
    pub fn diversity_metric(&self) -> f64 {
        let pareto = self.pareto_front.pareto_optimal();
        if pareto.len() <= 2 {
            return 1.0;
        }
        let total_cd: f64 = pareto.iter().map(|s| s.crowding_distance).sum();
        total_cd / pareto.len() as f64
    }

    /// Get statistics
    pub fn get_stats(&self) -> String {
        format!(
            r#"{{"evaluations":{},"pareto_size":{},"diversity":{:.4},"converged":{}}}"#,
            self.total_evaluations,
            self.front_size(),
            self.diversity_metric(),
            self.is_converged()
        )
    }
}

impl Default for MultiObjectiveOptimizer {
    fn default() -> Self {
        Self::for_trading()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pareto_domination() {
        let a = ParetoSolution::new(1, vec![], ObjectiveValues::new(10.0, 0.1, 0.5, 0.8, 0.2));
        let b = ParetoSolution::new(2, vec![], ObjectiveValues::new(5.0, 0.2, 0.8, 0.6, 0.1));
        assert!(a.dominates(&b, &ObjectiveWeights::default()));
        assert!(!b.dominates(&a, &ObjectiveWeights::default()));
    }

    #[test]
    fn test_weighted_sum() {
        let obj = ObjectiveValues::new(10.0, 0.1, 0.5, 0.8, 0.2);
        let weights = ObjectiveWeights::default();
        let score = obj.weighted_sum(&weights);
        assert!(score > 0.0);
    }

    #[test]
    fn test_multi_objective_evaluation() {
        let mut moo = MultiObjectiveOptimizer::for_trading();
        let obj = ObjectiveValues::new(10.0, 0.1, 0.5, 0.8, 0.2);
        moo.evaluate(vec![1.0; 25], obj, 0.0);
        assert_eq!(moo.total_evaluations, 1);
        assert_eq!(moo.front_size(), 1);
    }

    #[test]
    fn test_pareto_front_computation() {
        let mut moo = MultiObjectiveOptimizer::for_trading();
        for i in 0..20 {
            let obj = ObjectiveValues::new(
                5.0 + i as f64,
                0.5 - i as f64 * 0.02,
                0.1 + i as f64 * 0.01,
                0.5 + i as f64 * 0.02,
                0.1 + i as f64 * 0.01,
            );
            moo.evaluate(vec![1.0; 25], obj, 0.0);
        }
        assert!(moo.front_size() > 0);
    }
}
