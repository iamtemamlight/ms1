// ==============================================================================
// ETHICS & GUARDRAILS ENGINE - Sovereign Safety Boundaries for Autonomous Trading
// ==============================================================================

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use crate::m021_regional_modules::{
    SECURITY_CAPTAIN_HALT, FINANCIAL_SUPERVISOR_HALT, STRATEGIC_SUPERVISOR_HALT,
    ARCHITECTURE_SUPERVISOR_HALT, QUANTITATIVE_SUPERVISOR_HALT, RISK_SUPERVISOR_HALT,
    OPTIMIZATION_SUPERVISOR_HALT, PREDICTIVE_SUPERVISOR_HALT, RESEARCH_SUPERVISOR_HALT,
    INNOVATION_SUPERVISOR_HALT, COMPLIANCE_SUPERVISOR_HALT
};

/// Global Circuit Breaker State
static ETHICS_ENABLED: AtomicBool = AtomicBool::new(true);
static EMERGENCY_HALT: AtomicBool = AtomicBool::new(false);

/// Profit/Loss Limits (in ETH - scaled by 1000 for precision)
/// NOTE: These are defaults and MUST be overridden at runtime based on capital size.
/// Recommended: Daily loss limit = 5% of trading capital.
static DAILY_PROFIT_CAP: AtomicU64 = AtomicU64::new(10_000_000); // 10 ETH default
static HOURLY_PROFIT_CAP: AtomicU64 = AtomicU64::new(1_000_000);   // 1 ETH default
static DAILY_LOSS_LIMIT: AtomicU64 = AtomicU64::new(1_000_000);    // 1 ETH default (5% of 20 ETH capital)
static MAX_POSITION_SIZE: AtomicU64 = AtomicU64::new(50_000);      // 0.05 ETH max per trade

/// Rate Limiting
static _MAX_TRADES_PER_MINUTE: AtomicU64 = AtomicU64::new(60);
static MAX_CONSECUTIVE_LOSSES: AtomicU64 = AtomicU64::new(5);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
    Halted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthicsViolation {
    pub violation_type: String,
    pub severity: RiskLevel,
    pub description: String,
    pub blocked_action: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeAuthorization {
    pub approved: bool,
    pub risk_level: RiskLevel,
    pub reason: String,
    pub profit_remaining_eth: f64,
    pub loss_headroom_eth: f64,
}

#[derive(Debug, Clone, Default)]
pub struct EthicsMetrics {
    pub trades_approved: u64,
    pub trades_blocked: u64,
    pub violations_logged: u64,
    pub emergency_halt_count: u64,
    pub total_profit_capped: u64,
    pub losses_prevented_eth: u64,
}

// Main Ethics Engine
pub struct EthicsEngine {
    pub enabled: bool,
    pub daily_profit_accrued: AtomicU64,
    pub hourly_profit_accrued: AtomicU64,
    pub daily_loss_accrued: AtomicU64,
    pub consecutive_losses: AtomicU64,
    pub emergency_halt: bool,
    pub metrics: EthicsMetrics,
}

/// M61-M65 Specialist Agent Gating Matrix
#[derive(Debug, Clone, Copy)]
pub enum SpecialistAgent {
    Security,
    Financial,
    Strategic,
    Architecture,
    Quantitative,
    Risk,
    Optimization,
    Predictive,
    Research,
    Innovation,
    Compliance,
}

impl EthicsEngine {
    /// Set realistic limits based on capital size (in ETH)
    /// Daily loss limit = 5% of capital
    /// Max position = 1% of capital
    pub fn initialize_from_capital(&mut self, capital_eth: f64) {
        let daily_loss = (capital_eth * 0.05).max(0.01);
        let max_position = (capital_eth * 0.01).max(0.001);
        let daily_profit = (capital_eth * 0.10).max(0.01);
        let hourly_profit = (capital_eth * 0.02).max(0.001);

        DAILY_LOSS_LIMIT.store((daily_loss * 1_000_f64) as u64, Ordering::SeqCst);
        MAX_POSITION_SIZE.store((max_position * 1_000_f64) as u64, Ordering::SeqCst);
        DAILY_PROFIT_CAP.store((daily_profit * 1_000_f64) as u64, Ordering::SeqCst);
        HOURLY_PROFIT_CAP.store((hourly_profit * 1_000_f64) as u64, Ordering::SeqCst);

        tracing::info!(
            "Guardrails initialized: capital={:.4} ETH, daily_loss_limit={:.4}, max_position={:.4}",
            capital_eth, daily_loss, max_position
        );
    }

    /// Create new EthicsEngine with production limits
    pub fn new() -> Self {
        DAILY_PROFIT_CAP.store(150_000_000, Ordering::SeqCst);
        DAILY_LOSS_LIMIT.store(50_000_000, Ordering::SeqCst);
        MAX_POSITION_SIZE.store(100_000_000, Ordering::SeqCst);
        MAX_CONSECUTIVE_LOSSES.store(5, Ordering::SeqCst);
        EMERGENCY_HALT.store(false, Ordering::SeqCst);
        ETHICS_ENABLED.store(true, Ordering::SeqCst);
        SECURITY_CAPTAIN_HALT.store(false, Ordering::SeqCst);
        FINANCIAL_SUPERVISOR_HALT.store(false, Ordering::SeqCst);
        STRATEGIC_SUPERVISOR_HALT.store(false, Ordering::SeqCst);
        ARCHITECTURE_SUPERVISOR_HALT.store(false, Ordering::SeqCst);
        QUANTITATIVE_SUPERVISOR_HALT.store(false, Ordering::SeqCst);
        RISK_SUPERVISOR_HALT.store(false, Ordering::SeqCst);
        OPTIMIZATION_SUPERVISOR_HALT.store(false, Ordering::SeqCst);
        PREDICTIVE_SUPERVISOR_HALT.store(false, Ordering::SeqCst);
        RESEARCH_SUPERVISOR_HALT.store(false, Ordering::SeqCst);
        INNOVATION_SUPERVISOR_HALT.store(false, Ordering::SeqCst);
        COMPLIANCE_SUPERVISOR_HALT.store(false, Ordering::SeqCst);
        Self {
            enabled: true,
            daily_profit_accrued: AtomicU64::new(0),
            hourly_profit_accrued: AtomicU64::new(0),
            daily_loss_accrued: AtomicU64::new(0),
            consecutive_losses: AtomicU64::new(0),
            emergency_halt: false,
            metrics: EthicsMetrics::default(),
        }
    }

    /// Enable/disable ethics enforcement
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        ETHICS_ENABLED.store(enabled, Ordering::SeqCst);
    }

    /// Check if engine is enabled
    pub fn is_enabled(&self) -> bool {
        ETHICS_ENABLED.load(Ordering::SeqCst)
    }

    /// Set daily profit cap (in ETH)
    pub fn set_daily_profit_cap(&mut self, cap_eth: f64) {
        DAILY_PROFIT_CAP.store((cap_eth * 1_000_0_f64) as u64, Ordering::SeqCst);
    }

    /// Set daily loss limit (in ETH)
    pub fn set_daily_loss_limit(&mut self, limit_eth: f64) {
        DAILY_LOSS_LIMIT.store((limit_eth * 1_000_f64) as u64, Ordering::SeqCst);
    }

    /// Set max position size (in ETH)
    pub fn set_max_position(&mut self, size_eth: f64) {
        MAX_POSITION_SIZE.store((size_eth * 1_000_f64) as u64, Ordering::SeqCst);
    }

    /// Check if emergency halt is active
    pub fn is_halted(&self) -> bool {
        // Local Emergency Halt
        if EMERGENCY_HALT.load(Ordering::SeqCst) || self.emergency_halt {
            return true;
        }

        // Aggregate Specialist Supervisor Check (Unified Intelligence Gating)
        // This allows any of the 10 Specialists to veto execution in 1 CPU cycle.
        SECURITY_CAPTAIN_HALT.load(Ordering::SeqCst)
            || FINANCIAL_SUPERVISOR_HALT.load(Ordering::SeqCst)
            || STRATEGIC_SUPERVISOR_HALT.load(Ordering::SeqCst)
            || ARCHITECTURE_SUPERVISOR_HALT.load(Ordering::SeqCst)
            || QUANTITATIVE_SUPERVISOR_HALT.load(Ordering::SeqCst)
            || RISK_SUPERVISOR_HALT.load(Ordering::SeqCst)
            || OPTIMIZATION_SUPERVISOR_HALT.load(Ordering::SeqCst)
            || PREDICTIVE_SUPERVISOR_HALT.load(Ordering::SeqCst)
            || RESEARCH_SUPERVISOR_HALT.load(Ordering::SeqCst)
            || INNOVATION_SUPERVISOR_HALT.load(Ordering::SeqCst)
            || COMPLIANCE_SUPERVISOR_HALT.load(Ordering::SeqCst)
    }

    /// Trigger a halt from a specific specialist agent
    pub fn trigger_specialist_halt(&self, agent: SpecialistAgent, state: bool) {
        let flag = match agent {
            SpecialistAgent::Security => &SECURITY_CAPTAIN_HALT,
            SpecialistAgent::Financial => &FINANCIAL_SUPERVISOR_HALT,
            SpecialistAgent::Strategic => &STRATEGIC_SUPERVISOR_HALT,
            SpecialistAgent::Architecture => &ARCHITECTURE_SUPERVISOR_HALT,
            SpecialistAgent::Quantitative => &QUANTITATIVE_SUPERVISOR_HALT,
            SpecialistAgent::Risk => &RISK_SUPERVISOR_HALT,
            SpecialistAgent::Optimization => &OPTIMIZATION_SUPERVISOR_HALT,
            SpecialistAgent::Predictive => &PREDICTIVE_SUPERVISOR_HALT,
            SpecialistAgent::Research => &RESEARCH_SUPERVISOR_HALT,
            SpecialistAgent::Innovation => &INNOVATION_SUPERVISOR_HALT,
            SpecialistAgent::Compliance => &COMPLIANCE_SUPERVISOR_HALT,
        };
        flag.store(state, Ordering::SeqCst);
    }


    /// Trigger emergency halt - stops all fleet operations
    pub fn trigger_emergency_halt(&mut self, reason: &str) -> EthicsViolation {
        self.emergency_halt = true;
        EMERGENCY_HALT.store(true, Ordering::SeqCst);
        self.metrics.emergency_halt_count += 1;

        EthicsViolation {
            violation_type: "EMERGENCY_HALT".to_string(),
            severity: RiskLevel::Halted,
            description: reason.to_string(),
            blocked_action: "ALL_TRADES".to_string(),
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    /// Resume operations after halt
    pub fn resume_operations(&mut self) {
        self.emergency_halt = false;
        EMERGENCY_HALT.store(false, Ordering::SeqCst);
        self.consecutive_losses.store(0, Ordering::SeqCst);
    }

    /// Authorize a trade - returns approval status with reasoning
    pub fn authorize_trade(
        &mut self,
        position_size_eth: f64,
        expected_profit_eth: f64,
        expected_loss_eth: f64,
    ) -> TradeAuthorization {
        // Check emergency halt
        if self.is_halted() {
            return TradeAuthorization {
                approved: false,
                risk_level: RiskLevel::Halted,
                reason: format!(
                    "HALT_ACTIVE [SEC:{}|FIN:{}|RISK:{}]",
                    SECURITY_CAPTAIN_HALT.load(Ordering::SeqCst),
                    FINANCIAL_SUPERVISOR_HALT.load(Ordering::SeqCst),
                    RISK_SUPERVISOR_HALT.load(Ordering::SeqCst)
                ),
                profit_remaining_eth: 0.0,
                loss_headroom_eth: 0.0,
            };
        }

        let daily_cap = DAILY_PROFIT_CAP.load(Ordering::SeqCst) as f64 / 1_000_f64;
        let _hourly_cap = HOURLY_PROFIT_CAP.load(Ordering::SeqCst) as f64 / 1_000_f64;
        let daily_loss_limit = DAILY_LOSS_LIMIT.load(Ordering::SeqCst) as f64 / 1_000_f64;
        let max_position = MAX_POSITION_SIZE.load(Ordering::SeqCst) as f64 / 1_000_f64;

        // 1. Check position size limit
        if position_size_eth > max_position {
            self.metrics.trades_blocked += 1;
            return TradeAuthorization {
                approved: false,
                risk_level: RiskLevel::Critical,
                reason: format!("Position size {} ETH exceeds max {} ETH", position_size_eth, max_position),
                profit_remaining_eth: daily_cap,
                loss_headroom_eth: daily_loss_limit,
            };
        }

        // 2. Check daily profit cap
        let current_daily = self.daily_profit_accrued.load(Ordering::SeqCst) as f64 / 1_000_f64;
        if current_daily + expected_profit_eth > daily_cap {
            self.metrics.trades_blocked += 1;
            self.metrics.total_profit_capped += 1;
            return TradeAuthorization {
                approved: false,
                risk_level: RiskLevel::High,
                reason: "DAILY_PROFIT_CAP_REACHED".to_string(),
                profit_remaining_eth: (daily_cap - current_daily).max(0.0),
                loss_headroom_eth: daily_loss_limit,
            };
        }

        // 3. Check daily loss limit
        let current_loss = self.daily_loss_accrued.load(Ordering::SeqCst) as f64 / 1_000_f64;
        if current_loss >= daily_loss_limit {
            self.metrics.trades_blocked += 1;
            self.trigger_emergency_halt("DAILY_LOSS_LIMIT_EXCEEDED");
            return TradeAuthorization {
                approved: false,
                risk_level: RiskLevel::Halted,
                reason: "DAILY_LOSS_LIMIT_EXCEEDED".to_string(),
                profit_remaining_eth: 0.0,
                loss_headroom_eth: 0.0,
            };
        }

        // 4. Check expected loss would exceed limit
        if expected_loss_eth > (daily_loss_limit - current_loss) {
            self.metrics.trades_blocked += 1;
            self.metrics.losses_prevented_eth += expected_loss_eth as u64;
            return TradeAuthorization {
                approved: false,
                risk_level: RiskLevel::High,
                reason: "WOULD_EXCEED_LOSS_LIMIT".to_string(),
                profit_remaining_eth: daily_cap - current_daily,
                loss_headroom_eth: daily_loss_limit - current_loss,
            };
        }

        // 5. Check consecutive losses for circuit breaker
        let consec_losses = self.consecutive_losses.load(Ordering::SeqCst) as u64;
        let max_consec = MAX_CONSECUTIVE_LOSSES.load(Ordering::SeqCst);
        if consec_losses >= max_consec {
            // Additional trade - but flag as HIGH risk
            return TradeAuthorization {
                approved: false,
                risk_level: RiskLevel::Critical,
                reason: format!("CONSECUTIVE_LOSS_LIMIT ({}/{})", consec_losses, max_consec),
                profit_remaining_eth: daily_cap - current_daily,
                loss_headroom_eth: daily_loss_limit - current_loss,
            };
        }

        // Approval with risk assessment
        let risk = if expected_profit_eth / expected_loss_eth.max(0.001) < 1.5 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        };

        self.metrics.trades_approved += 1;

        TradeAuthorization {
            approved: true,
            risk_level: risk,
            reason: "AUTHORIZED".to_string(),
            profit_remaining_eth: daily_cap - current_daily,
            loss_headroom_eth: daily_loss_limit - current_loss,
        }
    }

    /// Record profit (call after successful trade)
    pub fn record_profit(&mut self, profit_eth: f64) {
        let units = (profit_eth * 1_000_f64) as u64;
        self.daily_profit_accrued.fetch_add(units, Ordering::SeqCst);
        self.hourly_profit_accrued.fetch_add(units, Ordering::SeqCst);
        
        // Reset consecutive losses on profit
        if profit_eth > 0.0 {
            self.consecutive_losses.store(0, Ordering::SeqCst);
        }
    }

    /// Record loss (call after losing trade)
    pub fn record_loss(&mut self, loss_eth: f64) {
        let units = (loss_eth * 1_000_f64) as u64;
        self.daily_loss_accrued.fetch_add(units, Ordering::SeqCst);
        self.consecutive_losses.fetch_add(1, Ordering::SeqCst);
    }

    /// Get current profit headroom
    pub fn profit_headroom(&self) -> f64 {
        let daily_cap = DAILY_PROFIT_CAP.load(Ordering::SeqCst) as f64 / 1_000_f64;
        let current = self.daily_profit_accrued.load(Ordering::SeqCst) as f64 / 1_000_f64;
        (daily_cap - current).max(0.0)
    }

    /// Get current loss headroom
    pub fn loss_headroom(&self) -> f64 {
        let daily_limit = DAILY_LOSS_LIMIT.load(Ordering::SeqCst) as f64 / 1_000_f64;
        let current = self.daily_loss_accrued.load(Ordering::SeqCst) as f64 / 1_000_f64;
        (daily_limit - current).max(0.0)
    }

    /// Reset daily counters (call at start of each day)
    pub fn reset_daily(&mut self) {
        self.daily_profit_accrued.store(0, Ordering::SeqCst);
        self.daily_loss_accrued.store(0, Ordering::SeqCst);
    }

    /// Reset hourly counters (call at start of each hour)
    pub fn reset_hourly(&mut self) {
        self.hourly_profit_accrued.store(0, Ordering::SeqCst);
    }

    /// Get metrics snapshot
    pub fn get_metrics(&self) -> EthicsMetrics {
        self.metrics.clone()
    }
}

impl Default for EthicsEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profit_cap() {
        let mut engine = EthicsEngine::new();
        let auth = engine.authorize_trade(10.0, 5.0, 2.0);
        assert!(auth.approved);
    }

    #[test]
    fn test_loss_limit_halt() {
        let mut engine = EthicsEngine::new();
        engine.set_daily_loss_limit(10.0);
        engine.record_loss(11.0);
        let auth = engine.authorize_trade(1.0, 1.0, 1.0);
        assert!(!auth.approved);
    }
}
