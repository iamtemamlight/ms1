// ==============================================================================
// M135: Flash Loan Governance Governor
// Purpose: Enhances existing governance modules (M012 Risk, M014/M033 Audit,
//          M050/M078 Governance Engines, M132-M134 Dual Audit) with
//          flash-loan-native, pre-trade risk gating, a signed exception
//          protocol, AI explainability, and Delegation-of-Authority rules.
//          Implements governance4.md points §1, §2(L2), §3, §4, §5, §7, §8.
// CGM Subsystem: Security / Quality
// ==============================================================================

use std::collections::HashMap;
use sha2::{Sha256, Digest};
use hex;

#[derive(Debug, Clone)]
pub struct ModuleMetrics {
    pub executions: u64,
    pub successes: u64,
    pub failures: u64,
    pub last_execution: Option<String>,
    pub average_latency_ms: f64,
    // Flash-loan governance-specific counters
    pub opportunities_evaluated: u64,
    pub opportunities_blocked: u64,
    pub exceptions_requested: u64,
    pub exceptions_approved: u64,
    pub daily_loss_running_eth: f64,
}

#[derive(Debug, Clone)]
pub struct ModuleResult {
    pub success: bool,
    pub message: String,
    pub data: HashMap<String, String>,
    pub execution_time_ms: u64,
}

// -----------------------------------------------------------------------------
// §3 / §5 Separation of Duties + Delegation of Authority (DoA) permission tiers
// Mirrors governance4.md Summary #1 (Developer/Operator/Treasury/Auditor/Admin).
// -----------------------------------------------------------------------------
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionRole {
    Developer,   // May NOT touch hot wallets / private keys (§3)
    Operator,    // Owns daily flash-loan risk (1st Line), may NOT change contract logic
    Treasury,    // May NOT self-approve reconciliations (§3) -> requires 2nd approver
    Auditor,     // Read-only 3rd Line access
    Admin,       // System admin may NOT authorize financial transactions (§3)
}

impl PermissionRole {
    pub fn can_authorize_capital(&self) -> bool {
        // §3: System Admins shall NOT authorize financial transactions. Only the
        // Treasury role (governance-approved) may authorize capital deployment.
        matches!(self, PermissionRole::Treasury)
    }
    pub fn can_modify_contract_logic(&self) -> bool {
        // §3: Traders/Operators shall NOT modify core flash loan contract logic.
        matches!(self, PermissionRole::Developer | PermissionRole::Admin)
    }
}

// -----------------------------------------------------------------------------
// Flash-loan-native risk policy (governance4.md §4, §7)
// Extends M012 RiskCalculator with limits specific to flash-loan arbitrage.
// -----------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct FlashLoanPolicy {
    pub max_flash_loan_size_eth: f64,
    pub max_slippage_bps: u64,
    pub max_gas_cost_eth: f64,
    pub max_daily_loss_eth: f64,
    pub per_pool_exposure_limit_eth: f64,
    // §6 Treasury: capital above this threshold requires multi-sig (governance4.md Summary #2)
    pub capital_multisig_threshold_eth: f64,
}

// SAFE FLOOR ONLY — used solely when the encrypted vault (M055) / runtime env is
// unavailable. The active policy in production is loaded via `from_env()`.
impl Default for FlashLoanPolicy {
    fn default() -> Self {
        Self {
            max_flash_loan_size_eth: 5_000_000.0,
            max_slippage_bps: 50,          // 0.50%
            max_gas_cost_eth: 0.05,
            max_daily_loss_eth: 25_000.0,
            per_pool_exposure_limit_eth: 1_000_000.0,
            capital_multisig_threshold_eth: 250_000.0,
        }
    }
}

impl FlashLoanPolicy {
    /// Load live policy from the encrypted vault (M055) exposed as runtime env vars.
    /// Every value is data-driven; no value is assumed. Missing vars fall back to the
    /// safe floor (Default) and are logged so operators can detect misconfiguration.
    pub fn from_env() -> Self {
        let floor = FlashLoanPolicy::default();
        let get_f64 = |k: &str, fallback: f64| -> f64 {
            std::env::var(k)
                .ok()
                .and_then(|v| v.parse::<f64>().ok())
                .unwrap_or_else(|| {
                    tracing::warn!("M135 policy env {} not set; using safe floor {}", k, fallback);
                    fallback
                })
        };
        let get_u64 = |k: &str, fallback: u64| -> u64 {
            std::env::var(k)
                .ok()
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or_else(|| {
                    tracing::warn!("M135 policy env {} not set; using safe floor {}", k, fallback);
                    fallback
                })
        };
        Self {
            max_flash_loan_size_eth: get_f64("M135_MAX_FLASH_LOAN_ETH", floor.max_flash_loan_size_eth),
            max_slippage_bps: get_u64("M135_MAX_SLIPPAGE_BPS", floor.max_slippage_bps),
            max_gas_cost_eth: get_f64("M135_MAX_GAS_COST_ETH", floor.max_gas_cost_eth),
            max_daily_loss_eth: get_f64("M135_MAX_DAILY_LOSS_ETH", floor.max_daily_loss_eth),
            per_pool_exposure_limit_eth: get_f64("M135_PER_POOL_EXPOSURE_ETH", floor.per_pool_exposure_limit_eth),
            capital_multisig_threshold_eth: get_f64("M135_CAPITAL_MULTISIG_ETH", floor.capital_multisig_threshold_eth),
        }
    }
}

// -----------------------------------------------------------------------------
// §8 AI Constitution — Explainability + no self-authorization of capital.
// Every arb recommendation carries an auditable rationale string.
// -----------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct FlashLoanOpportunity {
    pub opportunity_id: String,
    pub pool: String,
    pub loan_size_eth: f64,
    pub expected_profit_eth: f64,
    pub slippage_bps: u64,
    pub gas_cost_eth: f64,
    pub risk_score: f64, // 0.0 (low) .. 1.0 (critical)
    // §8 Explainability: human-readable reason the AI suggests this trade.
    pub recommendation_rationale: String,
    pub proposed_by: PermissionRole,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    Approve,        // Within all policies -> bot may execute (1st Line owns risk)
    Exception,      // Exceeds a policy -> requires signed exception + governance approval
    Deny,           // Hard breach (e.g. daily loss cap) -> blocked regardless
}

#[derive(Debug, Clone)]
pub struct FlashLoanVerdict {
    pub verdict: Verdict,
    pub opportunity_id: String,
    pub policy_compliance_rate: f64, // §9 Governance KPI: fraction of checks passed
    pub failed_checks: Vec<String>,
    pub explainability: String,      // §8 auditable one-line reason
    pub requires_multisig: bool,     // §6
    pub signature: String,           // §4/§7 signed decision context
    pub payload_hash: String,
    pub authorization_trail: Vec<String>,
}

// -----------------------------------------------------------------------------
// §7 Exception Protocol — over-threshold trades must justify + be signed.
// -----------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct ExceptionRequest {
    pub opportunity_id: String,
    pub business_justification: String,
    pub additional_controls: String,
    pub residual_risks: String,
    pub requested_by: PermissionRole,
    pub approved_by: Option<PermissionRole>, // None until governance signs off
    pub signature: String,
    pub payload_hash: String,
}

#[derive(Debug)]
pub struct FlashLoanGovernor {
    pub enabled: bool,
    pub metrics: ModuleMetrics,
    pub config: HashMap<String, String>,
    pub policy: FlashLoanPolicy,
    pub pool_exposure: HashMap<String, f64>,
    pub exceptions: Vec<ExceptionRequest>,
    // Signing secret for decision/exception payloads (§4/§7 no verbal overrides).
    pub signing_secret: String,
}

impl FlashLoanGovernor {
    pub fn new() -> Self {
        let mut config = HashMap::new();
        config.insert("module".to_string(), "M135".to_string());
        config.insert("line_of_defense".to_string(), "L2-RiskAndCompliance".to_string());
        Self {
            enabled: true,
            metrics: ModuleMetrics {
                executions: 0,
                successes: 0,
                failures: 0,
                last_execution: None,
                average_latency_ms: 0.0,
                opportunities_evaluated: 0,
                opportunities_blocked: 0,
                exceptions_requested: 0,
                exceptions_approved: 0,
                daily_loss_running_eth: 0.0,
            },
            config,
            // Policy is loaded from the encrypted vault (M055) / runtime env — never the
            // hardcoded Default. Default is only a safe floor used when vault is unavailable.
            policy: FlashLoanPolicy::from_env(),
            pool_exposure: HashMap::new(),
            exceptions: Vec::new(),
            // In production this is loaded from the encrypted vault (M055), not hardcoded.
            signing_secret: std::env::var("GOVERNOR_SIGNING_SECRET")
                .unwrap_or_else(|_| "allbright-flash-loan-governor-v1".to_string()),
        }
    }

    // §6 Treasury: capital move above threshold requires multi-sig confirmation.
    pub fn requires_multisig(&self, capital_eth: f64) -> bool {
        capital_eth > self.policy.capital_multisig_threshold_eth
    }

    // -------------------------------------------------------------------------
    // §1 / §2(L2) Pre-trade risk gating. Runs BEFORE the bot signs, independent
    // of the trading engine (1st Line). Returns a signed verdict.
    // -------------------------------------------------------------------------
    pub fn evaluate_opportunity(&mut self, opp: &FlashLoanOpportunity) -> FlashLoanVerdict {
        let start = std::time::Instant::now();
        self.metrics.executions += 1;
        self.metrics.opportunities_evaluated += 1;

        let mut failed_checks: Vec<String> = Vec::new();

        if opp.loan_size_eth > self.policy.max_flash_loan_size_eth {
            failed_checks.push(format!(
                "flash_loan_size {} > max {}",
                opp.loan_size_eth, self.policy.max_flash_loan_size_eth
            ));
        }
        if opp.slippage_bps > self.policy.max_slippage_bps {
            failed_checks.push(format!(
                "slippage_bps {} > max {}",
                opp.slippage_bps, self.policy.max_slippage_bps
            ));
        }
        if opp.gas_cost_eth > self.policy.max_gas_cost_eth {
            failed_checks.push(format!(
                "gas_cost_eth {} > max {}",
                opp.gas_cost_eth, self.policy.max_gas_cost_eth
            ));
        }
        let current_pool_exposure = self.pool_exposure.get(&opp.pool).copied().unwrap_or(0.0);
        if current_pool_exposure + opp.loan_size_eth > self.policy.per_pool_exposure_limit_eth {
            failed_checks.push(format!(
                "pool_exposure {} + {} > limit {}",
                current_pool_exposure, opp.loan_size_eth, self.policy.per_pool_exposure_limit_eth
            ));
        }
        // Hard breach: daily loss cap (§4 Capital/Liquidity Limits). Cannot be overridden.
        if self.metrics.daily_loss_running_eth + (opp.gas_cost_eth - opp.expected_profit_eth).max(0.0)
            > self.policy.max_daily_loss_eth
        {
            failed_checks.push(format!(
                "daily_loss_cap {} exceeded",
                self.policy.max_daily_loss_eth
            ));
        }

        let total_checks = 5usize;
        let passed = total_checks - failed_checks.len();
        let compliance_rate = passed as f64 / total_checks as f64;

        let verdict = if failed_checks.iter().any(|c| c.contains("daily_loss_cap")) {
            self.metrics.opportunities_blocked += 1;
            Verdict::Deny
        } else if failed_checks.is_empty() {
            if opp.proposed_by.can_authorize_capital() || !self.requires_multisig(opp.loan_size_eth) {
                Verdict::Approve
            } else {
                // §8: AI may not self-authorize capital above threshold -> needs governance.
                Verdict::Exception
            }
        } else {
            Verdict::Exception
        };

        let explainability = match verdict {
            Verdict::Approve => format!(
                "APPROVE — Profit {:.4} ETH, Risk {:.2}, Slippage {} bps, {} checks passed",
                opp.expected_profit_eth, opp.risk_score, opp.slippage_bps, passed
            ),
            Verdict::Exception => format!(
                "EXCEPTION REQUIRED — {} policy breach(es): {:?}. {}",
                failed_checks.len(), failed_checks, opp.recommendation_rationale
            ),
            Verdict::Deny => format!(
                "DENY — hard policy breach: {:?}. {}",
                failed_checks, opp.recommendation_rationale
            ),
        };

        let requires_msig = self.requires_multisig(opp.loan_size_eth) && verdict != Verdict::Deny;

        let payload = format!(
            "{}|{:?}|{:.4}|{}|{}",
            opp.opportunity_id, verdict, compliance_rate, explainability, chrono::Utc::now().to_rfc3339()
        );
        let (payload_hash, signature) = self.sign(&payload);

        let authorization_trail = vec![
            format!("L1-operator:{}", role_name(opp.proposed_by)),
            format!("L2-governor:M135"),
        ];

        if verdict == Verdict::Approve {
            self.metrics.successes += 1;
            *self.pool_exposure.entry(opp.pool.clone()).or_default() += opp.loan_size_eth;
        }

        self.metrics.last_execution = Some(chrono::Utc::now().to_rfc3339());
        self.metrics.average_latency_ms =
            (self.metrics.average_latency_ms * (self.metrics.executions as f64 - 1.0)
                + start.elapsed().as_millis() as f64)
                / self.metrics.executions as f64;

        FlashLoanVerdict {
            verdict,
            opportunity_id: opp.opportunity_id.clone(),
            policy_compliance_rate: compliance_rate,
            failed_checks,
            explainability,
            requires_multisig: requires_msig,
            signature,
            payload_hash,
            authorization_trail,
        }
    }

    // -------------------------------------------------------------------------
    // §7 Exception Protocol: over-threshold trade requests governance sign-off.
    // Business Justification + Additional Controls + Residual Risks are mandatory.
    // Produces a signed, non-repudiable record (no verbal overrides).
    // -------------------------------------------------------------------------
    pub fn request_exception(
        &mut self,
        opportunity_id: &str,
        business_justification: &str,
        additional_controls: &str,
        residual_risks: &str,
        requested_by: PermissionRole,
    ) -> Result<ExceptionRequest, String> {
        if business_justification.trim().is_empty()
            || additional_controls.trim().is_empty()
            || residual_risks.trim().is_empty()
        {
            return Err("§7 Exception Protocol: justification, additional controls, and residual risks are all required".to_string());
        }
        self.metrics.exceptions_requested += 1;

        let payload = format!(
            "{}|{}|{}|{}|{}|{}",
            opportunity_id,
            business_justification,
            additional_controls,
            residual_risks,
            role_name(requested_by),
            chrono::Utc::now().to_rfc3339()
        );
        let (payload_hash, signature) = self.sign(&payload);

        let req = ExceptionRequest {
            opportunity_id: opportunity_id.to_string(),
            business_justification: business_justification.to_string(),
            additional_controls: additional_controls.to_string(),
            residual_risks: residual_risks.to_string(),
            requested_by,
            approved_by: None,
            signature,
            payload_hash,
        };
        self.exceptions.push(req.clone());
        Ok(req)
    }

    // §4 DoA: Capital/Liquidity Limits — approval above threshold by governance.
    pub fn approve_exception(&mut self, opportunity_id: &str, approved_by: PermissionRole) -> Result<(), String> {
        if !approved_by.can_authorize_capital() {
            return Err("§4 DoA: approver role lacks capital-authorization authority".to_string());
        }
        if let Some(req) = self.exceptions.iter_mut().find(|e| e.opportunity_id == opportunity_id) {
            req.approved_by = Some(approved_by);
            self.metrics.exceptions_approved += 1;
            Ok(())
        } else {
            Err(format!("No pending exception for opportunity {}", opportunity_id))
        }
    }

    // Record realized loss to enforce the daily loss cap across trades.
    pub fn record_realized_pnl(&mut self, net_profit_eth: f64) {
        if net_profit_eth < 0.0 {
            self.metrics.daily_loss_running_eth += net_profit_eth.abs();
        }
    }

    fn sign(&self, payload: &str) -> (String, String) {
        let mut hasher = Sha256::new();
        hasher.update(payload.as_bytes());
        hasher.update(self.signing_secret.as_bytes());
        let digest = hasher.finalize();
        let payload_hash = hex::encode(digest);
        // Signature = hash signed with the governor secret; verified downstream by
        // M014/M033 audit trail / AuditFramework immutable records.
        let mut sig_hasher = Sha256::new();
        sig_hasher.update(payload_hash.as_bytes());
        sig_hasher.update(self.signing_secret.as_bytes());
        (payload_hash, hex::encode(sig_hasher.finalize()))
    }

    pub fn get_health(&self) -> f64 {
        if self.metrics.executions == 0 {
            return 1.0;
        }
        self.metrics.successes as f64 / self.metrics.executions as f64
    }

    pub fn get_stats(&self) -> String {
        format!(
            r#"{{"executions":{},"successes":{},"failures":{},"health":{:.2},"evaluated":{},"blocked":{},"exceptions":{},"daily_loss_eth":{:.4}}}"#,
            self.metrics.executions,
            self.metrics.successes,
            self.metrics.failures,
            self.get_health(),
            self.metrics.opportunities_evaluated,
            self.metrics.opportunities_blocked,
            self.metrics.exceptions_requested,
            self.metrics.daily_loss_running_eth,
        )
    }
}

fn role_name(r: PermissionRole) -> &'static str {
    match r {
        PermissionRole::Developer => "Developer",
        PermissionRole::Operator => "Operator",
        PermissionRole::Treasury => "Treasury",
        PermissionRole::Auditor => "Auditor",
        PermissionRole::Admin => "Admin",
    }
}
