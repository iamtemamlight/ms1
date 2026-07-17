// ==============================================================================
// MODULE: Multi-Objective Solver Extension
// Purpose: Extends Newton-Raphson to optimize profit + risk + latency simultaneously
//          using weighted sum approach with Pareto frontier calculation
// ==============================================================================

use serde::{Deserialize, Serialize};

/// Multi-objective optimization weights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiObjectiveWeights {
    pub profit_weight: f64,    // Weight for profit maximization
    pub risk_weight: f64,     // Weight for risk minimization
    pub latency_weight: f64,   // Weight for latency minimization
}

impl Default for MultiObjectiveWeights {
    fn default() -> Self {
        Self {
            profit_weight: 0.5,
            risk_weight: 0.3,
            latency_weight: 0.2,
        }
    }
}

impl MultiObjectiveWeights {
    pub fn validate(&self) -> bool {
        let sum = self.profit_weight + self.risk_weight + self.latency_weight;
        (sum - 1.0).abs() < 1e-6 && 
        self.profit_weight >= 0.0 &&
        self.risk_weight >= 0.0 &&
        self.latency_weight >= 0.0
    }

    pub fn normalize(&mut self) {
        let sum = self.profit_weight + self.risk_weight + self.latency_weight;
        if sum > 0.0 {
            self.profit_weight /= sum;
            self.risk_weight /= sum;
            self.latency_weight /= sum;
        }
    }
}

/// Objective function values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectiveValues {
    pub profit: f64,
    pub risk: f64,      // Higher = more risky
    pub latency_us: f64,
}

impl ObjectiveValues {
    pub fn new(profit: f64, risk: f64, latency_us: f64) -> Self {
        Self { profit, risk, latency_us }
    }

    /// Combine multiple objectives into single scalar using weighted sum
    pub fn weighted_sum(&self, weights: &MultiObjectiveWeights) -> f64 {
        // Maximize profit (positive), minimize risk (negative), minimize latency (negative)
        weights.profit_weight * self.profit 
            - weights.risk_weight * self.risk 
            - weights.latency_weight * self.latency_us
    }
}

/// Pareto frontier solution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParetoSolution {
    pub q_value: f64,
    pub objectives: ObjectiveValues,
    pub rank: u32,           // Pareto rank (0 = best)
    pub crowding_distance: f64,  // Diversity measure
}

/// Multi-objective solver using NSGA-II inspired approach
pub struct MultiObjectiveSolver {
    pub max_iterations: u32,
    pub tolerance: f64,
    pub population_size: usize,
    pub weights: MultiObjectiveWeights,
}

impl MultiObjectiveSolver {
    pub fn new(weights: MultiObjectiveWeights) -> Self {
        Self {
            max_iterations: 50,
            tolerance: 1e-6,
            population_size: 100,
            weights,
        }
    }

    /// Optimize for multiple objectives simultaneously
    pub fn solve<F: Fn(f64) -> ObjectiveValues>(
        &self, 
        objective_fn: F, 
        initial_q: f64,
    ) -> Option<ParetoSolution> {
        // Generate initial population around initial_q
        let mut population: Vec<(f64, ObjectiveValues)> = Vec::new();
        
        for i in 0..self.population_size {
            let q = initial_q * (0.5 + (i as f64 / self.population_size as f64));
            let objectives = objective_fn(q);
            population.push((q, objectives));
        }

        // Evaluate and find Pareto front
        let pareto_front = self.fast_non_dominated_sort(&population);
        
        if pareto_front.is_empty() {
            return None;
        }

        // Find best solution using weighted sum
        let mut best_solution: Option<ParetoSolution> = None;
        let mut best_score = f64::NEG_INFINITY;

        for (q, objectives) in &pareto_front {
            let score = objectives.weighted_sum(&self.weights);
            if score > best_score {
                best_score = score;
                let solution = ParetoSolution {
                    q_value: *q,
                    objectives: objectives.clone(),
                    rank: 0,
                    crowding_distance: 0.0,
                };
                best_solution = Some(solution);
            }
        }

        // Refine best solution using gradient descent
        if let Some(mut best) = best_solution {
            best = self.local_search(objective_fn, best);
            return Some(best);
        }

        best_solution
    }

    /// Fast non-dominated sorting (NSGA-II)
    fn fast_non_dominated_sort(
        &self, 
        population: &[(f64, ObjectiveValues)]
    ) -> Vec<(f64, ObjectiveValues)> {
        let mut pareto_front: Vec<(f64, ObjectiveValues)> = Vec::new();
        
        for (i, (qi, obji)) in population.iter().enumerate() {
            let mut is_dominated = false;
            
            for (j, (_qj, objj)) in population.iter().enumerate() {
                if i == j { continue; }
                
                // Check if obji is dominated by objj
                // (i.e., objj is >= obji in all objectives and > in at least one)
                if objj.profit >= obji.profit 
                    && objj.risk <= obji.risk 
                    && objj.latency_us <= obji.latency_us
                    && (objj.profit > obji.profit 
                        || objj.risk < obji.risk 
                        || objj.latency_us < obji.latency_us)
                {
                    is_dominated = true;
                    break;
                }
            }
            
            if !is_dominated {
                pareto_front.push((*qi, obji.clone()));
            }
        }

        pareto_front
    }

    /// Local search refinement around best solution
    fn local_search<F: Fn(f64) -> ObjectiveValues>(
        &self,
        objective_fn: F,
        mut best: ParetoSolution,
    ) -> ParetoSolution {
        let step_sizes = [0.1, 0.05, 0.01, 0.005, 0.001];
        
        for &step in &step_sizes {
            let q_candidates = [
                best.q_value + step,
                best.q_value - step,
                best.q_value * (1.0 + step),
                best.q_value * (1.0 - step),
            ];
            
            for q in q_candidates {
                if q <= 0.0 { continue; }
                
                let objectives = objective_fn(q);
                let score = objectives.weighted_sum(&self.weights);
                let current_score = best.objectives.weighted_sum(&self.weights);
                
                if score > current_score {
                    best.q_value = q;
                    best.objectives = objectives;
                }
            }
        }
        
        best
    }

    /// Calculate Pareto frontier (all non-dominated solutions)
    pub fn compute_pareto_frontier<F: Fn(f64) -> ObjectiveValues>(
        &self,
        objective_fn: F,
        q_range: (f64, f64),
        steps: usize,
    ) -> Vec<ParetoSolution> {
        let mut solutions: Vec<(f64, ObjectiveValues)> = Vec::new();
        
        let step_size = (q_range.1 - q_range.0) / steps as f64;
        for i in 0..=steps {
            let q = q_range.0 + (i as f64) * step_size;
            let objectives = objective_fn(q);
            solutions.push((q, objectives));
        }

        // Find Pareto front
        let pareto_front = self.fast_non_dominated_sort(&solutions);
        
        pareto_front
            .into_iter()
            .enumerate()
            .map(|(i, (q, objectives))| ParetoSolution {
                q_value: q,
                objectives,
                rank: 0,
                crowding_distance: i as f64,
            })
            .collect()
    }

    /// Get optimal Q value for given constraints
    pub fn solve_with_constraints<F: Fn(f64) -> ObjectiveValues>(
        &self,
        objective_fn: F,
        initial_q: f64,
        max_risk: f64,
        max_latency_us: f64,
    ) -> Option<ParetoSolution> {
        // Use compute_pareto_frontier to get all solutions, then find feasible one
        let frontier = self.compute_pareto_frontier(objective_fn, (initial_q * 0.5, initial_q * 2.0), 50);
        
        // First try to find a solution that meets constraints
        for sol in &frontier {
            if sol.objectives.risk <= max_risk 
                && sol.objectives.latency_us <= max_latency_us 
            {
                return Some(sol.clone());
            }
        }
        
        // If no solution meets constraints, find one with minimum violation
        if !frontier.is_empty() {
            let mut best = None;
            let mut best_violation = f64::MAX;
            
            for sol in &frontier {
                let risk_violation = (sol.objectives.risk - max_risk).max(0.0);
                let latency_violation = (sol.objectives.latency_us - max_latency_us).max(0.0);
                let total_violation = risk_violation * 1000.0 + latency_violation;
                
                if total_violation < best_violation {
                    best_violation = total_violation;
                    best = Some(sol.clone());
                }
            }
            
            return best;
        }
        
        None
    }
}

/// Convenience function using default weights (profit-focused)
pub fn solve_multi_objective<F: Fn(f64) -> ObjectiveValues>(
    objective_fn: F,
    initial_q: f64,
) -> Option<ParetoSolution> {
    let weights = MultiObjectiveWeights::default();
    let solver = MultiObjectiveSolver::new(weights);
    solver.solve(objective_fn, initial_q)
}

/// Convenience function with custom weights
pub fn solve_multi_objective_custom<F: Fn(f64) -> ObjectiveValues>(
    objective_fn: F,
    initial_q: f64,
    profit_w: f64,
    risk_w: f64,
    latency_w: f64,
) -> Option<ParetoSolution> {
    let mut weights = MultiObjectiveWeights {
        profit_weight: profit_w,
        risk_weight: risk_w,
        latency_weight: latency_w,
    };
    weights.normalize();
    
    let solver = MultiObjectiveSolver::new(weights);
    solver.solve(objective_fn, initial_q)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weights_validation() {
        let weights = MultiObjectiveWeights::default();
        assert!(weights.validate());
    }

    #[test]
    fn test_objective_weighted_sum() {
        let obj = ObjectiveValues::new(100.0, 0.2, 20.0);
        let weights = MultiObjectiveWeights::default();
        
        let score = obj.weighted_sum(&weights);
        // 0.5*100 - 0.3*0.2 - 0.2*20 = 50 - 0.06 - 4 = 45.94
        assert!(score > 45.0 && score < 46.0);
    }

    #[test]
    fn test_multi_objective_solver() {
        let objective_fn = |q: f64| {
            ObjectiveValues::new(
                q * 0.95,
                q * 0.001,
                15.0 + q * 0.5,
            )
        };

        let solver = MultiObjectiveSolver::new(MultiObjectiveWeights::default());
        let solution = solver.solve(objective_fn, 1000.0);

        assert!(solution.is_some());
        let sol = solution.unwrap();
        assert!(sol.q_value > 0.0);
    }

    #[test]
    fn test_pareto_frontier() {
        let objective_fn = |q: f64| {
            ObjectiveValues::new(q, q * 0.001, 15.0 + q * 0.001)
        };

        let solver = MultiObjectiveSolver::new(MultiObjectiveWeights::default());
        let frontier = solver.compute_pareto_frontier(objective_fn, (100.0, 2000.0), 10);

        assert!(!frontier.is_empty());
    }
}
