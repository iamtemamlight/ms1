// ==============================================================================
// M200: Bayesian Optimization Engine
// Purpose: Gaussian Process-based Bayesian Optimization for auto-optimizer
//          Replaces rule-based KPI→dimension mapping with learning-based approach
// CGM Subsystem: Optimization / Growth
// ==============================================================================

use std::collections::VecDeque;
use std::f64::{INFINITY, NEG_INFINITY};
use ndarray::{Array1, Array2, ArrayView1, ArrayView2, Axis};
use serde::{Deserialize, Serialize};

/// Maximum observations to retain in the GP sliding window
const GP_MAX_OBSERVATIONS: usize = 500;

/// Minimum observations before BO can make recommendations
const GP_MIN_OBSERVATIONS: usize = 10;

/// Default UCB exploration parameter (beta)
const DEFAULT_UCB_BETA: f64 = 2.0;

/// RBF Kernel hyperparameter bounds
const KERNEL_LENGTH_SCALE_MIN: f64 = 0.01;
const KERNEL_LENGTH_SCALE_MAX: f64 = 10.0;
const KERNEL_VARIANCE_MIN: f64 = 1e-6;
const KERNEL_VARIANCE_MAX: f64 = 10.0;
const NOISE_VARIANCE_MIN: f64 = 1e-6;
const NOISE_VARIANCE_MAX: f64 = 1.0;

/// Observation record for GP training
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    pub parameters: Vec<f64>,
    pub objective_value: f64,
    pub timestamp_ms: u64,
    pub context_hash: u64,
}

/// Optimization context for market-aware recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationContext {
    pub regime: MarketRegime,
    pub network_congestion: f64,
    pub gas_price_gwei: f64,
    pub pool_liquidity_usd: f64,
    pub fleet_size: u32,
    pub active_runners: u32,
    pub volatility_index: f64,
    pub block_height: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MarketRegime {
    Bull,
    Bear,
    Sideways,
    Volatile,
    Crash,
    Recovery,
}

impl MarketRegime {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Bull => "BULL",
            Self::Bear => "BEAR",
            Self::Sideways => "SIDEWAYS",
            Self::Volatile => "VOLATILE",
            Self::Crash => "CRASH",
            Self::Recovery => "RECOVERY",
        }
    }
}

/// GP hyperparameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpHyperparameters {
    pub length_scale: Vec<f64>,
    pub variance: f64,
    pub noise_variance: f64,
    pub mean: f64,
}

impl Default for GpHyperparameters {
    fn default() -> Self {
        Self {
            length_scale: vec![1.0; 25],
            variance: 1.0,
            noise_variance: 0.01,
            mean: 0.0,
        }
    }
}

/// Gaussian Process surrogate model
#[derive(Debug, Clone)]
pub struct GaussianProcess {
    pub observations: VecDeque<Observation>,
    pub hyperparameters: GpHyperparameters,
    pub input_dim: usize,
    pub ucb_beta: f64,
    pub last_log_likelihood: f64,
}

/// Bayesian Optimizer configuration
#[derive(Debug, Clone)]
pub struct BayesianOptimizerConfig {
    pub input_dim: usize,
    pub parameter_bounds: Vec<(f64, f64)>,
    pub ucb_beta: f64,
    pub exploration_weight: f64,
    pub exploitation_weight: f64,
    pub max_iterations: u32,
    pub convergence_window: usize,
    pub convergence_threshold: f64,
}

impl Default for BayesianOptimizerConfig {
    fn default() -> Self {
        Self {
            input_dim: 25,
            parameter_bounds: vec![(0.5, 1.5); 25],
            ucb_beta: DEFAULT_UCB_BETA,
            exploration_weight: 0.3,
            exploitation_weight: 0.7,
            max_iterations: 100,
            convergence_window: 10,
            convergence_threshold: 0.001,
        }
    }
}

/// Bayesian Optimizer recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRecommendation {
    pub parameters: Vec<f64>,
    pub expected_improvement: f64,
    pub confidence_interval: (f64, f64),
    pub acquisition_value: f64,
    pub exploration_score: f64,
    pub exploitation_score: f64,
    pub regime_alignment: f64,
    pub rationale: String,
}

/// Optimization result with metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    pub recommendation: OptimizationRecommendation,
    pub iteration: u32,
    pub gp_uncertainty: f64,
    pub convergence_indicator: f64,
    pub regime_shift_detected: bool,
    pub warm_start_used: bool,
}

impl GaussianProcess {
    pub fn new(input_dim: usize) -> Self {
        Self {
            observations: VecDeque::with_capacity(GP_MAX_OBSERVATIONS),
            hyperparameters: GpHyperparameters::default(),
            input_dim,
            ucb_beta: DEFAULT_UCB_BETA,
            last_log_likelihood: 0.0,
        }
    }

    /// Add observation and update GP
    pub fn add_observation(&mut self, obs: Observation) {
        if self.observations.len() >= GP_MAX_OBSERVATIONS {
            self.observations.pop_front();
        }
        self.observations.push_back(obs);
        self.optimize_hyperparameters();
    }

    /// RBF (ARD) Kernel: k(x, x') = variance * exp(-0.5 * sum_i (x_i - x'_i)^2 / l_i^2)
    pub fn rbf_kernel(&self, x1: &[f64], x2: &[f64]) -> f64 {
        let mut sum = 0.0;
        for i in 0..self.input_dim {
            let diff = x1[i] - x2[i];
            let ls = self.hyperparameters.length_scale[i].max(1e-6);
            sum += (diff * diff) / (ls * ls);
        }
        self.hyperparameters.variance * (-0.5 * sum).exp()
    }

    /// Compute full covariance matrix K(X, X)
    pub fn covariance_matrix(&self, X: &Array2<f64>) -> Array2<f64> {
        let n = X.nrows();
        let mut K = Array2::zeros((n, n));
        for i in 0..n {
            for j in i..n {
                let row_i = X.row(i).to_vec();
                let row_j = X.row(j).to_vec();
                let val = if i == j {
                    self.rbf_kernel(&row_i, &row_j) + self.hyperparameters.noise_variance
                } else {
                    self.rbf_kernel(&row_i, &row_j)
                };
                K[[i, j]] = val;
                K[[j, i]] = val;
            }
        }
        K
    }

    /// Compute cross-covariance between two input sets
    pub fn cross_covariance(&self, X1: &Array2<f64>, X2: &Array2<f64>) -> Array2<f64> {
        let n1 = X1.nrows();
        let n2 = X2.nrows();
        let mut K = Array2::zeros((n1, n2));
        for i in 0..n1 {
            for j in 0..n2 {
                let row_i = X1.row(i).to_vec();
                let row_j = X2.row(j).to_vec();
                K[[i, j]] = self.rbf_kernel(&row_i, &row_j);
            }
        }
        K
    }

    /// GP predictive distribution: returns (mean, variance) for test points
    pub fn predict(&self, X_test: &Array2<f64>) -> (Array1<f64>, Array1<f64>) {
        let n = self.observations.len() as f64;
        if n < GP_MIN_OBSERVATIONS as f64 {
            return (Array1::zeros(X_test.nrows()), Array1::ones(X_test.nrows()) * 0.5);
        }

        let X_train = self.training_matrix();
        let y_train = self.training_vector();
        let K = self.covariance_matrix(&X_train);
        let K_star = self.cross_covariance(&X_train, X_test);

        // Cholesky decomposition: K = L * L^T
        let K_regularized = &K + Array2::eye(K.nrows()) * 1e-6;
        let L = match Self::cholesky(&K_regularized) {
            Some(l) => l,
            None => return (Array1::zeros(X_test.nrows()), Array1::ones(X_test.nrows()) * 0.5),
        };

        // Alpha = L^T \ (L \ y)
        let mut alpha = Self::solve_triangular(&L, &y_train, true);
        alpha = Self::solve_triangular(&L, &alpha, false);

        // Mean: mu = K_star^T * alpha
        let mut means = Array1::zeros(X_test.nrows());
        for i in 0..X_test.nrows() {
            for j in 0..K_star.nrows() {
                means[i] += K_star[[j, i]] * alpha[j];
            }
            means[i] += self.hyperparameters.mean;
        }

        // Variance: var = k(x,x) - sum_i sum_j v_i * K_ij * v_j
        // where v = L \ k_star
        let mut variances = Array1::ones(X_test.nrows());
        for i in 0..X_test.nrows() {
            let k_star_col = K_star.column(i);
            let mut v = Self::solve_triangular(&L, &k_star_col.to_owned(), true);
            let mut vT_K_v = 0.0;
            for j in 0..v.len() {
                for k in 0..v.len() {
                    vT_K_v += v[j] * K[[j, k]] * v[k];
                }
            }
            let k_self = self.rbf_kernel(
                &X_test.row(i).to_vec(),
                &X_test.row(i).to_vec(),
            );
            variances[i] = k_self - vT_K_v + self.hyperparameters.noise_variance;
            if variances[i] < 1e-6 {
                variances[i] = 1e-6;
            }
        }

        (means, variances)
    }

    /// Cholesky decomposition for positive definite matrix
    fn cholesky(A: &Array2<f64>) -> Option<Array2<f64>> {
        let n = A.nrows();
        let mut L = Array2::zeros((n, n));
        for i in 0..n {
            for j in 0..=i {
                let mut sum = if i == j { A[[i, i]] } else { 0.0 };
                for k in 0..j {
                    sum -= L[[i, k]] * L[[j, k]];
                }
                if i == j {
                    if sum <= 0.0 {
                        return None;
                    }
                    L[[i, j]] = sum.sqrt();
                } else {
                    L[[i, j]] = sum / L[[j, j]];
                }
            }
        }
        Some(L)
    }

    /// Solve L * x = b (lower triangular) or L^T * x = b (upper triangular)
    fn solve_triangular(L: &Array2<f64>, b: &Array1<f64>, transpose: bool) -> Array1<f64> {
        let n = L.nrows();
        let mut x = Array1::zeros(n);
        if transpose {
            for i in (0..n).rev() {
                let mut sum = b[i];
                for j in (i + 1)..n {
                    sum -= L[[j, i]] * x[j];
                }
                x[i] = sum / L[[i, i]];
            }
        } else {
            for i in 0..n {
                let mut sum = b[i];
                for j in 0..i {
                    sum -= L[[i, j]] * x[j];
                }
                x[i] = sum / L[[i, i]];
            }
        }
        x
    }

    /// Upper Confidence Bound acquisition function
    pub fn ucb(&self, mean: f64, variance: f64) -> f64 {
        mean + self.ucb_beta * variance.sqrt()
    }

    /// Expected Improvement acquisition function
    pub fn expected_improvement(&self, mean: f64, variance: f64, best_f: f64) -> f64 {
        if variance < 1e-9 {
            return 0.0;
        }
        let z = (mean - best_f) / variance.sqrt();
        let pdf = (-0.5 * z * z).exp() / (2.0 * std::f64::consts::PI).sqrt();
        let cdf = 0.5 * (1.0 + z / (1.0 + 0.0001 * z.abs()).tanh());
        (mean - best_f) * cdf + variance.sqrt() * pdf
    }

    /// Optimize GP hyperparameters using gradient-free optimization
    fn optimize_hyperparameters(&mut self) {
        if self.observations.len() < GP_MIN_OBSERVATIONS {
            return;
        }

        let X = self.training_matrix();
        let y = self.training_vector();
        let n = X.nrows();

        // Compute current log marginal likelihood
        let current_lml = self.log_marginal_likelihood(&X, &y);
        self.last_log_likelihood = current_lml;

        // Simple grid search for length scale (per dimension)
        let mut best_lml = current_lml;
        let mut best_ls = self.hyperparameters.length_scale.clone();

        for dim in 0..self.input_dim.min(5) {
            let candidates = [
                self.hyperparameters.length_scale[dim] * 0.5,
                self.hyperparameters.length_scale[dim],
                self.hyperparameters.length_scale[dim] * 2.0,
            ];
            for &candidate in &candidates {
                if candidate < KERNEL_LENGTH_SCALE_MIN || candidate > KERNEL_LENGTH_SCALE_MAX {
                    continue;
                }
                let mut test_hp = self.hyperparameters.length_scale.clone();
                test_hp[dim] = candidate;
                let test_lml = self.compute_lml_with_params(&X, &y, &test_hp);
                if test_lml > best_lml {
                    best_lml = test_lml;
                    best_ls[dim] = candidate;
                }
            }
        }

        if best_lml > current_lml {
            self.hyperparameters.length_scale = best_ls;
            self.last_log_likelihood = best_lml;
        }
    }

    /// Compute log marginal likelihood with given length scales
    fn compute_lml_with_params(&self, X: &Array2<f64>, y: &Array1<f64>, ls: &[f64]) -> f64 {
        let n = X.nrows();
        let mut K = Array2::zeros((n, n));
        for i in 0..n {
            for j in i..n {
                let mut sum = 0.0;
                for d in 0..self.input_dim {
                    let diff = X[[i, d]] - X[[j, d]];
                    sum += (diff * diff) / (ls[d].max(1e-6) * ls[d].max(1e-6));
                }
                let val = self.hyperparameters.variance * (-0.5 * sum).exp();
                K[[i, j]] = if i == j { val + self.hyperparameters.noise_variance } else { val };
                K[[j, i]] = K[[i, j]];
            }
        }

        let K_reg = &K + Array2::eye(n) * 1e-6;
        if let Some(L) = Self::cholesky(&K_reg) {
            let mut alpha = Self::solve_triangular(&L, y, true);
            alpha = Self::solve_triangular(&L, &alpha, false);
            let data_fit = -0.5 * y.dot(&alpha);
            let complexity = -L.diag().mapv(|x| x.ln()).sum();
            let constant = -0.5 * n as f64 * (2.0 * std::f64::consts::PI).ln();
            data_fit + complexity + constant
        } else {
            NEG_INFINITY
        }
    }

    /// Log marginal likelihood
    fn log_marginal_likelihood(&self, X: &Array2<f64>, y: &Array1<f64>) -> f64 {
        self.compute_lml_with_params(X, y, &self.hyperparameters.length_scale)
    }

    /// Build training matrix from observations
    fn training_matrix(&self) -> Array2<f64> {
        let n = self.observations.len();
        let d = self.input_dim;
        let mut data = Vec::with_capacity(n * d);
        for obs in &self.observations {
            for &val in &obs.parameters {
                data.push(val);
            }
        }
        Array2::from_shape_vec((n, d), data).unwrap_or_else(|_| Array2::zeros((0, d)))
    }

    /// Build training vector from observations
    fn training_vector(&self) -> Array1<f64> {
        let n = self.observations.len();
        let mut data = Vec::with_capacity(n);
        for obs in &self.observations {
            data.push(obs.objective_value);
        }
        Array1::from_vec(data)
    }

    /// Get best observed objective value
    pub fn best_objective(&self) -> f64 {
        self.observations.iter()
            .map(|o| o.objective_value)
            .fold(NEG_INFINITY, f64::max)
    }

    /// Get observation count
    pub fn observation_count(&self) -> usize {
        self.observations.len()
    }
}

/// Bayesian Optimizer - main interface
#[derive(Debug, Clone)]
pub struct BayesianOptimizer {
    pub gp: GaussianProcess,
    pub config: BayesianOptimizerConfig,
    pub iteration: u32,
    pub convergence_history: VecDeque<f64>,
    pub last_recommendation: Option<OptimizationRecommendation>,
    pub regime_shift_history: VecDeque<u64>,
}

impl BayesianOptimizer {
    pub fn new(config: BayesianOptimizerConfig) -> Self {
        let convergence_window = config.convergence_window;
        let input_dim = config.input_dim;
        let gp = GaussianProcess::new(input_dim);
        Self {
            gp,
            config,
            iteration: 0,
            convergence_history: VecDeque::with_capacity(convergence_window),
            last_recommendation: None,
            regime_shift_history: VecDeque::with_capacity(10),
        }
    }

    /// Create optimizer with default configuration for 25 trading dimensions
    pub fn for_trading_engine() -> Self {
        let config = BayesianOptimizerConfig {
            input_dim: 25,
            parameter_bounds: vec![
                (0.5, 2.0),   // d0: Corridor Width
                (0.5, 2.0),   // d1: Bribe Amount
                (0.0, 1.0),   // d2: Block Phase
                (0.5, 2.0),   // d3: Bundle Size
                (0.5, 2.0),   // d4: Flash Loan Size
                (0.0, 1.0),   // d5: Competitor Response
                (0.5, 2.0),   // d6: Regional Variant
                (0.5, 2.0),   // d7: Route Efficiency
                (0.5, 2.0),   // d8: Shield Routing
                (0.5, 2.0),   // d9: Slippage Buffer
                (0.5, 2.0),   // d10: Capital Allocation
                (0.5, 2.0),   // d11: Multi-Hop Legs
                (0.0, 1.0),   // d12: Execution Timing
                (0.0, 1.0),   // d13: Gas Priority
                (0.0, 1.0),   // d14: MEV Protection Level
                (0.5, 2.0),   // d15: Liquidity Depth Factor
                (0.5, 2.0),   // d16: Pool Tier Selection
                (0.0, 1.0),   // d17: RPC Failover Threshold
                (0.0, 1.0),   // d18: Mempool Monitoring
                (0.5, 2.0),   // d19: Chain Selection
                (0.5, 2.0),   // d20: Regional Routing
                (0.5, 2.0),   // d21: Gas Cycle Phase
                (0.5, 2.0),   // d22: Runner Capacity
                (0.5, 2.0),   // d23: JIT Liquidity Factor
                (0.5, 2.0),   // d24: Solver Tolerance
            ],
            ucb_beta: 2.5,
            exploration_weight: 0.35,
            exploitation_weight: 0.65,
            max_iterations: 200,
            convergence_window: 15,
            convergence_threshold: 0.0005,
            ..Default::default()
        };
        Self::new(config)
    }

    /// Record an observation and update the GP
    pub fn observe(&mut self, parameters: Vec<f64>, objective_value: f64, ctx: &OptimizationContext) {
        if parameters.len() != self.config.input_dim {
            tracing::warn!(
                "BO: parameter dimension mismatch: expected {}, got {}",
                self.config.input_dim,
                parameters.len()
            );
            return;
        }

        let obs = Observation {
            parameters,
            objective_value,
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
            context_hash: Self::hash_context(ctx),
        };

        self.gp.add_observation(obs);
        self.iteration += 1;

        // Track convergence
        self.convergence_history.push_back(objective_value);
        if self.convergence_history.len() > self.config.convergence_window {
            self.convergence_history.pop_front();
        }
    }

    /// Get next recommendation using UCB acquisition
    pub fn recommend(&mut self, ctx: &OptimizationContext) -> OptimizationResult {
        if self.gp.observation_count() < GP_MIN_OBSERVATIONS {
            return self.random_recommendation(ctx, "insufficient_data");
        }

        let best_f = self.gp.best_objective();
        let convergence = self.compute_convergence();
        let regime_shift = self.detect_regime_shift(ctx);

        // Multi-start optimization of acquisition function
        let mut best_acq = NEG_INFINITY;
        let mut best_params = vec![1.0; self.config.input_dim];
        let num_starts = 32;

        for start in 0..num_starts {
            let mut x = self.sample_candidate(start, ctx);
            let (mut val, _) = self.evaluate_acquisition_ucb(&x, best_f);
            
            // Local optimization via coordinate descent
            for _ in 0..20 {
                let mut improved = false;
                for dim in 0..self.config.input_dim {
                    let (lo, hi) = self.config.parameter_bounds[dim];
                    let step = (hi - lo) * 0.05;
                    for &delta in &[-step, 0.0, step] {
                        let mut candidate = x.clone();
                        candidate[dim] = (candidate[dim] + delta).max(lo).min(hi);
                        let (candidate_val, _) = self.evaluate_acquisition_ucb(&candidate, best_f);
                        if candidate_val > val {
                            val = candidate_val;
                            x = candidate;
                            improved = true;
                        }
                    }
                }
                if !improved {
                    break;
                }
            }

            if val > best_acq {
                best_acq = val;
                best_params = x;
            }
        }

        let (mean, variance) = self.gp.predict(&Array2::from_shape_vec(
            (1, self.config.input_dim),
            best_params.clone(),
        ).unwrap());
        
        let predicted_mean = mean[0];
        let predicted_std = variance[0].sqrt();
        let confidence_interval = (
            predicted_mean - 1.96 * predicted_std,
            predicted_mean + 1.96 * predicted_std,
        );

        let exploration_score = predicted_std / (predicted_mean.abs() + 0.001);
        let exploitation_score = predicted_mean / (best_f.abs() + 0.001);
        let regime_alignment = self.compute_regime_alignment(&best_params, ctx);

        let recommendation = OptimizationRecommendation {
            parameters: best_params.clone(),
            expected_improvement: self.gp.expected_improvement(predicted_mean, predicted_std.powi(2), best_f),
            confidence_interval,
            acquisition_value: best_acq,
            exploration_score,
            exploitation_score,
            regime_alignment,
            rationale: Self::generate_rationale(
                predicted_mean,
                predicted_std,
                exploration_score,
                regime_alignment,
                ctx,
            ),
        };

        let result = OptimizationResult {
            recommendation: recommendation.clone(),
            iteration: self.iteration,
            gp_uncertainty: predicted_std,
            convergence_indicator: convergence,
            regime_shift_detected: regime_shift,
            warm_start_used: false,
        };

        self.last_recommendation = Some(recommendation);
        result
    }

    /// Evaluate acquisition function (UCB)
    fn evaluate_acquisition_ucb(&self, x: &[f64], best_f: f64) -> (f64, (f64, f64)) {
        let X_test = Array2::from_shape_vec(
            (1, self.config.input_dim),
            x.to_vec(),
        ).unwrap();
        let (mean, variance) = self.gp.predict(&X_test);
        let acq = self.gp.ucb(mean[0], variance[0]);
        (acq, (mean[0], variance[0]))
    }

    /// Sample candidate parameters (Latin Hypercube style)
    fn sample_candidate(&self, seed: usize, ctx: &OptimizationContext) -> Vec<f64> {
        let mut rng_state = seed as f64;
        let mut candidate = Vec::with_capacity(self.config.input_dim);
        for dim in 0..self.config.input_dim {
            let (lo, hi) = self.config.parameter_bounds[dim];
            rng_state = (rng_state * 9301.0 + 49297.0) % 233280.0;
            let ratio = rng_state / 233280.0;
            candidate.push(lo + ratio * (hi - lo));
        }
        candidate
    }

    /// Compute convergence indicator (variance of recent objectives)
    fn compute_convergence(&self) -> f64 {
        if self.convergence_history.len() < 2 {
            return 1.0;
        }
        let mean: f64 = self.convergence_history.iter().sum::<f64>() / self.convergence_history.len() as f64;
        let variance: f64 = self.convergence_history.iter()
            .map(|&v| (v - mean).powi(2))
            .sum::<f64>() / self.convergence_history.len() as f64;
        variance.sqrt() / (mean.abs() + 0.001)
    }

    /// Detect regime shift by comparing recent context hashes
    fn detect_regime_shift(&mut self, ctx: &OptimizationContext) -> bool {
        let current_hash = Self::hash_context(ctx);
        let shifted = self.regime_shift_history.back().map_or(true, |&last| {
            last != current_hash
        });
        if shifted {
            self.regime_shift_history.push_back(current_hash);
            if self.regime_shift_history.len() > 10 {
                self.regime_shift_history.pop_front();
            }
        }
        shifted
    }

    /// Compute alignment with current market regime
    fn compute_regime_alignment(&self, params: &[f64], ctx: &OptimizationContext) -> f64 {
        let mut alignment = 0.0;
        match ctx.regime {
            MarketRegime::Volatile | MarketRegime::Crash => {
                // In volatile/crash: favor smaller positions, higher shield, lower risk
                if params.len() > 4 {
                    alignment += (1.0 - params[4]) * 0.3; // Lower flash loan size
                    alignment += params[8] * 0.3; // Higher shield routing
                    alignment += params[16] * 0.2; // Conservative pool tier
                }
            }
            MarketRegime::Bull | MarketRegime::Recovery => {
                // In bull/recovery: favor larger positions, higher velocity
                if params.len() > 4 {
                    alignment += params[4] * 0.3; // Higher flash loan size
                    alignment += params[2] * 0.2; // Aggressive block phase
                }
            }
            MarketRegime::Sideways => {
                // In sideways: favor efficiency, MEV protection
                if params.len() > 10 {
                    alignment += params[9] * 0.2; // Slippage buffer
                    alignment += params[14] * 0.3; // MEV protection
                }
            }
            MarketRegime::Bear => {
                // In bear: favor capital preservation
                if params.len() > 8 {
                    alignment += (1.0 - params[4]) * 0.4; // Smaller positions
                    alignment += params[8] * 0.3; // Higher shield
                }
            }
        }
        alignment.clamp(0.0, 1.0)
    }

    /// Generate human-readable rationale
    fn generate_rationale(
        mean: f64,
        std: f64,
        exploration: f64,
        regime_alignment: f64,
        ctx: &OptimizationContext,
    ) -> String {
        let uncertainty = if std > 0.3 { "high" } else if std > 0.15 { "moderate" } else { "low" };
        let confidence = if exploration > 0.5 { "exploratory" } else { "exploitative" };
        format!(
            "Regime={} | Predicted={:.4}±{:.4} ({}) uncertainty | {} action | Regime fit={:.0}% | Congestion={:.0}%",
            ctx.regime.as_str(),
            mean,
            std,
            uncertainty,
            confidence,
            regime_alignment * 100.0,
            ctx.network_congestion * 100.0
        )
    }

    /// Fallback: random recommendation when insufficient data
    fn random_recommendation(&mut self, ctx: &OptimizationContext, reason: &str) -> OptimizationResult {
        let mut params = Vec::with_capacity(self.config.input_dim);
        for dim in 0..self.config.input_dim {
            let (lo, hi) = self.config.parameter_bounds[dim];
            let mid = (lo + hi) / 2.0;
            let range = (hi - lo) / 2.0;
            params.push(mid + (ctx.network_congestion - 0.5) * range * 0.3);
        }

        let recommendation = OptimizationRecommendation {
            parameters: params.clone(),
            expected_improvement: 0.0,
            confidence_interval: (0.0, 0.0),
            acquisition_value: 0.0,
            exploration_score: 1.0,
            exploitation_score: 0.0,
            regime_alignment: 0.5,
            rationale: format!("Warm start: {} observations collected", self.gp.observation_count()),
        };

        OptimizationResult {
            recommendation,
            iteration: self.iteration,
            gp_uncertainty: 0.5,
            convergence_indicator: 1.0,
            regime_shift_detected: false,
            warm_start_used: true,
        }
    }

    /// Simple hash of optimization context for regime tracking
    fn hash_context(ctx: &OptimizationContext) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        ctx.regime.as_str().hash(&mut hasher);
        ((ctx.network_congestion * 1000.0).round() as u64).hash(&mut hasher);
        (ctx.gas_price_gwei.round() as u64 / 5).hash(&mut hasher);
        ((ctx.volatility_index * 1000.0).round() as u64).hash(&mut hasher);
        hasher.finish()
    }

    /// Get convergence status
    pub fn is_converged(&self) -> bool {
        if self.convergence_history.len() < self.config.convergence_window {
            return false;
        }
        self.compute_convergence() < self.config.convergence_threshold
    }

    /// Get GP uncertainty (mean predictive variance)
    pub fn gp_uncertainty(&self) -> f64 {
        if self.gp.observation_count() < GP_MIN_OBSERVATIONS {
            return 0.5;
        }
        let test_points = self.generate_test_points(20);
        let (_, variances) = self.gp.predict(&test_points);
        variances.mean().unwrap_or(0.5)
    }

    /// Generate test points for uncertainty estimation
    fn generate_test_points(&self, count: usize) -> Array2<f64> {
        let mut data = Vec::with_capacity(count * self.config.input_dim);
        for i in 0..count {
            for dim in 0..self.config.input_dim {
                let (lo, hi) = self.config.parameter_bounds[dim];
                let val = lo + (i as f64 / count as f64) * (hi - lo);
                data.push(val);
            }
        }
        Array2::from_shape_vec((count, self.config.input_dim), data).unwrap_or_else(|_| {
            Array2::zeros((count, self.config.input_dim))
        })
    }

    /// Reset optimizer state (for regime change)
    pub fn reset(&mut self) {
        self.gp.observations.clear();
        self.iteration = 0;
        self.convergence_history.clear();
        self.last_recommendation = None;
        self.gp.hyperparameters = GpHyperparameters::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gp_rbf_kernel() {
        let gp = GaussianProcess::new(2);
        let k = gp.rbf_kernel(&[0.0, 0.0], &[1.0, 1.0]);
        assert!(k > 0.0 && k <= 1.0);
    }

    #[test]
    fn test_gp_covariance() {
        let gp = GaussianProcess::new(2);
        let X = Array2::from_shape_vec(
            (3, 2),
            vec![0.0, 0.0, 1.0, 1.0, 2.0, 2.0],
        ).unwrap();
        let K = gp.covariance_matrix(&X);
        assert_eq!(K.nrows(), 3);
        assert_eq!(K.ncols(), 3);
    }

    #[test]
    fn test_bayesian_optimizer_creation() {
        let bo = BayesianOptimizer::for_trading_engine();
        assert_eq!(bo.config.input_dim, 25);
        assert_eq!(bo.config.parameter_bounds.len(), 25);
    }

    #[test]
    fn test_observation_recording() {
        let mut bo = BayesianOptimizer::for_trading_engine();
        let ctx = OptimizationContext {
            regime: MarketRegime::Sideways,
            network_congestion: 0.3,
            gas_price_gwei: 25.0,
            pool_liquidity_usd: 5_000_000.0,
            fleet_size: 100,
            active_runners: 95,
            volatility_index: 0.2,
            block_height: 18000000,
        };

        for i in 0..15 {
            let params = vec![1.0; 25];
            let objective = 0.1 + (i as f64) * 0.01;
            bo.observe(params, objective, &ctx);
        }

        assert_eq!(bo.gp.observation_count(), 15);
        assert!(!bo.is_converged());
    }

    #[test]
    fn test_recommendation_generation() {
        let mut bo = BayesianOptimizer::for_trading_engine();
        let ctx = OptimizationContext {
            regime: MarketRegime::Sideways,
            network_congestion: 0.3,
            gas_price_gwei: 25.0,
            pool_liquidity_usd: 5_000_000.0,
            fleet_size: 100,
            active_runners: 95,
            volatility_index: 0.2,
            block_height: 18000000,
        };

        for i in 0..15 {
            let params = vec![1.0; 25];
            let objective = 0.1 + (i as f64) * 0.01;
            bo.observe(params, objective, &ctx);
        }

        let result = bo.recommend(&ctx);
        assert_eq!(result.recommendation.parameters.len(), 25);
        assert!(result.gp_uncertainty >= 0.0);
    }
}
