// ==============================================================================
// M021: Flash Loan Verifier
// Purpose: Verify flash loan repayment and settlement correctness
// CGM Subsystem: Quality
// ==============================================================================

use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct ModuleMetrics {
    pub executions: u64,
    pub successes: u64,
    pub failures: u64,
    pub last_execution: Option<String>,
    pub average_latency_ms: f64,
    pub loans_verified: u64,
    pub repayments_confirmed: u64,
}

#[derive(Debug, Clone)]
pub struct ModuleResult {
    pub success: bool,
    pub message: String,
    pub data: HashMap<String, String>,
    pub execution_time_ms: u64,
    pub verification_passed: bool,
    pub loan_delta: f64,
}

#[derive(Debug, Clone)]
pub struct FlashLoanVerification {
    pub loan_amount_eth: f64,
    pub fee_paid_eth: f64,
    pub repayment_amount_eth: f64,
    pub final_balance_eth: f64,
    pub expected_balance_eth: f64,
    pub verified: bool,
    pub timestamp: String,
}

#[derive(Debug)]
pub struct M21 {
    pub enabled: bool,
    pub metrics: ModuleMetrics,
    pub config: HashMap<String, String>,
    pub verifications: Vec<FlashLoanVerification>,
}

impl M21 {
    pub fn new() -> Self {
        Self {
            enabled: true,
            metrics: ModuleMetrics {
                executions: 0,
                successes: 0,
                failures: 0,
                last_execution: None,
                average_latency_ms: 0.0,
                loans_verified: 0,
                repayments_confirmed: 0,
            },
            config: HashMap::new(),
            verifications: Vec::new(),
        }
    }

    pub fn verify_repayment(
        &mut self,
        loan_amount_eth: f64,
        fee_paid_eth: f64,
        repayment_amount_eth: f64,
        final_balance_eth: f64,
        expected_balance_eth: f64,
    ) -> ModuleResult {
        if !self.enabled {
            return ModuleResult {
                success: false,
                message: "Module disabled".to_string(),
                data: HashMap::new(),
                execution_time_ms: 0,
                verification_passed: false,
                loan_delta: 0.0,
            };
        }

        let start = Instant::now();
        self.metrics.executions += 1;

        // Flash loan: borrow → trade → repay + fee
        // Expected final balance = expected - fee (since we borrowed and repaid)
        // Actually: we borrowed N, made profit P, repaid N+fee, net profit P-fee
        let expected_after_loan = expected_balance_eth + loan_amount_eth;
        let expected_after_trade_and_repay = expected_after_loan - repayment_amount_eth - fee_paid_eth;
        
        let loan_delta = (final_balance_eth - expected_after_trade_and_repay).abs();
        let tolerance = expected_after_trade_and_repay.abs() * 0.001; // 0.1% tolerance
        
        let verification_passed = loan_delta <= tolerance;

        if verification_passed {
            self.metrics.successes += 1;
            self.metrics.loans_verified += 1;
            self.metrics.repayments_confirmed += 1;
        } else {
            self.metrics.failures += 1;
        }

        let verification = FlashLoanVerification {
            loan_amount_eth,
            fee_paid_eth,
            repayment_amount_eth,
            final_balance_eth,
            expected_balance_eth,
            verified: verification_passed,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        if self.verifications.len() > 10000 {
            self.verifications.remove(0);
        }
        self.verifications.push(verification);

        let mut data = HashMap::new();
        data.insert("loan_amount_eth".to_string(), loan_amount_eth.to_string());
        data.insert("fee_paid_eth".to_string(), fee_paid_eth.to_string());
        data.insert("repayment_amount_eth".to_string(), repayment_amount_eth.to_string());
        data.insert("final_balance_eth".to_string(), final_balance_eth.to_string());

        let elapsed = start.elapsed().as_millis() as u64;
        self.metrics.last_execution = Some(chrono::Utc::now().to_rfc3339());
        self.metrics.average_latency_ms = if self.metrics.executions == 1 {
            elapsed as f64
        } else {
            (self.metrics.average_latency_ms * (self.metrics.executions - 1) as f64 + elapsed as f64)
                / self.metrics.executions as f64
        };

        ModuleResult {
            success: verification_passed,
            message: if verification_passed {
                "Flash loan repayment verified".to_string()
            } else {
                format!("Flash loan repayment FAILED: delta {} ETH", loan_delta)
            },
            data,
            execution_time_ms: elapsed,
            verification_passed,
            loan_delta,
        }
    }

    pub fn execute(&mut self) -> ModuleResult {
        if !self.enabled {
            return ModuleResult {
                success: false,
                message: "Module disabled".to_string(),
                data: HashMap::new(),
                execution_time_ms: 0,
                verification_passed: false,
                loan_delta: 0.0,
            };
        }

        let start = Instant::now();
        self.metrics.executions += 1;

        let success_count = self.verifications.iter().filter(|v| v.verified).count();

        let result = ModuleResult {
            success: true,
            message: format!("M021 executed: {} verifications, {} successful", 
                self.verifications.len(), success_count),
            data: HashMap::new(),
            execution_time_ms: start.elapsed().as_millis() as u64,
            verification_passed: true,
            loan_delta: 0.0,
        };

        self.metrics.successes += 1;
        self.metrics.last_execution = Some(chrono::Utc::now().to_rfc3339());
        result
    }

    pub fn get_health(&self) -> f64 {
        if self.metrics.executions == 0 {
            return 1.0;
        }
        self.metrics.successes as f64 / self.metrics.executions as f64
    }

    pub fn get_stats(&self) -> String {
        format!(
            r#"{{"executions":{},"successes":{},"failures":{},"health":{:.2},"loans_verified":{},"repayments_confirmed":{}}}"#,
            self.metrics.executions,
            self.metrics.successes,
            self.metrics.failures,
            self.get_health(),
            self.metrics.loans_verified,
            self.metrics.repayments_confirmed
        )
    }
}