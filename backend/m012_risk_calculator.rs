// ==============================================================================
// M012: Risk Calculator
// Purpose: Calculate and aggregate risk metrics for portfolio and positions
// CGM Subsystem: Security
// ==============================================================================

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct RiskMetric {
    pub name: String,
    pub value: f64,
    pub threshold: f64,
    pub severity: RiskSeverity,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct PositionRisk {
    pub asset: String,
    pub size: f64,
    pub leverage: f64,
    pub var_1d: f64,
    pub var_7d: f64,
    pub beta: f64,
    pub liquidity_score: f64,
}

#[derive(Debug, Clone)]
pub struct PortfolioRisk {
    pub total_exposure: f64,
    pub net_exposure: f64,
    pub gross_exposure: f64,
    pub var_1d: f64,
    pub var_7d: f64,
    pub expected_shortfall: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
}

#[derive(Debug)]
pub struct RiskCalculator {
    pub enabled: bool,
    pub metrics: Vec<RiskMetric>,
    pub positions: HashMap<String, PositionRisk>,
    pub risk_limits: HashMap<String, f64>,
    pub total_calculations: u64,
    pub breaches_detected: u64,
    pub last_calculation: Option<String>,
}

impl RiskCalculator {
    pub fn new() -> Self {
        let mut risk_limits = HashMap::new();
        risk_limits.insert("max_position_size".to_string(), 100_000.0);
        risk_limits.insert("max_leverage".to_string(), 3.0);
        risk_limits.insert("max_var_1d".to_string(), 5_000.0);
        risk_limits.insert("max_drawdown".to_string(), 0.15);

        Self {
            enabled: true,
            metrics: Vec::new(),
            positions: HashMap::new(),
            risk_limits,
            total_calculations: 0,
            breaches_detected: 0,
            last_calculation: None,
        }
    }

    pub fn add_position(&mut self, position: PositionRisk) {
        self.positions.insert(position.asset.clone(), position);
    }

    pub fn calculate_position_risk(&mut self, asset: &str) -> Option<RiskMetric> {
        if let Some(pos) = self.positions.get(asset) {
            self.total_calculations += 1;
            self.last_calculation = Some(chrono::Utc::now().to_rfc3339());

            let mut metric = RiskMetric {
                name: format!("{}_risk", asset),
                value: pos.var_1d,
                threshold: self.risk_limits.get("max_var_1d").copied().unwrap_or(5000.0),
                severity: RiskSeverity::Low,
                description: format!("1-day VaR for {}", asset),
            };

            if pos.var_1d > metric.threshold {
                metric.severity = RiskSeverity::Critical;
                self.breaches_detected += 1;
            } else if pos.var_1d > metric.threshold * 0.7 {
                metric.severity = RiskSeverity::High;
            } else if pos.var_1d > metric.threshold * 0.4 {
                metric.severity = RiskSeverity::Medium;
            }

            self.metrics.push(metric.clone());
            Some(metric)
        } else {
            None
        }
    }

    pub fn calculate_portfolio_risk(&self) -> PortfolioRisk {
        let total_exposure: f64 = self.positions.values().map(|p| p.size * p.leverage).sum();
        let net_exposure: f64 = self.positions.values().map(|p| p.size * (p.leverage - 1.0)).sum();
        
        let var_1d: f64 = self.positions.values().map(|p| p.var_1d).sum();
        let var_7d: f64 = self.positions.values().map(|p| p.var_7d).sum();
        
        let total_liquidity: f64 = self.positions.values().map(|p| p.liquidity_score).sum();
        let avg_liquidity = if self.positions.is_empty() { 0.0 } else { total_liquidity / self.positions.len() as f64 };
        
        let expected_shortfall = var_1d * 1.2;
        let sharpe_ratio = if var_1d > 0.0 { 0.15 / (var_1d / total_exposure) } else { 0.0 };
        let max_drawdown = var_7d / total_exposure;

        PortfolioRisk {
            total_exposure,
            net_exposure,
            gross_exposure: total_exposure,
            var_1d,
            var_7d,
            expected_shortfall,
            sharpe_ratio,
            max_drawdown,
        }
    }

    pub fn check_risk_breaches(&self) -> Vec<RiskMetric> {
        let portfolio = self.calculate_portfolio_risk();
        let mut breaches = Vec::new();

        if portfolio.total_exposure > self.risk_limits.get("max_position_size").copied().unwrap_or(100_000.0) {
            breaches.push(RiskMetric {
                name: "total_exposure".to_string(),
                value: portfolio.total_exposure,
                threshold: self.risk_limits.get("max_position_size").copied().unwrap_or(100_000.0),
                severity: RiskSeverity::Critical,
                description: "Total portfolio exposure exceeds limit".to_string(),
            });
        }

        if portfolio.max_drawdown > self.risk_limits.get("max_drawdown").copied().unwrap_or(0.15) {
            breaches.push(RiskMetric {
                name: "max_drawdown".to_string(),
                value: portfolio.max_drawdown,
                threshold: self.risk_limits.get("max_drawdown").copied().unwrap_or(0.15),
                severity: RiskSeverity::High,
                description: "Maximum drawdown threshold breached".to_string(),
            });
        }

        breaches
    }

    pub fn get_risk_summary(&self) -> String {
        let portfolio = self.calculate_portfolio_risk();
        let breaches = self.check_risk_breaches();
        
        format!(
            r#"{{"total_exposure":{},"var_1d":{},"var_7d":{},"max_drawdown":{:.3},"breaches":{}}}"#,
            portfolio.total_exposure,
            portfolio.var_1d,
            portfolio.var_7d,
            portfolio.max_drawdown,
            breaches.len()
        )
    }
}
