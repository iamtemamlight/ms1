// ==============================================================================
// M008: MEV Protection Engine
// Purpose: Protect transactions from MEV extraction (front-running, sandwich attacks)
// CGM Subsystem: Security
// ==============================================================================

use std::collections::HashSet;
use crate::m003_transaction_batcher::Transaction;

#[derive(Debug, Clone)]
pub struct MevThreat {
    pub threat_type: MevThreatType,
    pub confidence: f64,
    pub estimated_loss_wei: u64,
    pub attacker_address: Option<String>,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MevThreatType {
    FrontRun,
    Sandwich,
    BackRun,
    TimeBandit,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtectionLevel {
    None,
    Basic,
    Standard,
    Maximum,
}

#[derive(Debug, Clone)]
pub struct ProtectionResult {
    pub protected: bool,
    pub level: ProtectionLevel,
    pub threats_detected: usize,
    pub estimated_savings_wei: u64,
    pub actions_taken: Vec<String>,
}

#[derive(Debug)]
pub struct MevProtectionEngine {
    pub enabled: bool,
    pub default_protection: ProtectionLevel,
    pub known_bots: HashSet<String>,
    pub protected_txs: u64,
    pub threats_blocked: u64,
    pub total_savings_wei: u64,
    pub last_scan: Option<String>,
}

impl MevProtectionEngine {
    pub fn new() -> Self {
        let mut known_bots = HashSet::new();
        known_bots.insert("0x0000000000000000000000000000000000000000".to_string());

        Self {
            enabled: true,
            default_protection: ProtectionLevel::Standard,
            known_bots,
            protected_txs: 0,
            threats_blocked: 0,
            total_savings_wei: 0,
            last_scan: None,
        }
    }

    pub fn analyze_transaction(&mut self, tx: &Transaction) -> Vec<MevThreat> {
        if !self.enabled {
            return vec![];
        }

        self.last_scan = Some(chrono::Utc::now().to_rfc3339());
        let mut threats = Vec::new();

        if self.is_known_bot(&tx.to) {
            threats.push(MevThreat {
                threat_type: MevThreatType::FrontRun,
                confidence: 0.95,
                estimated_loss_wei: tx.value / 100,
                attacker_address: Some(tx.to.clone()),
                description: "Known MEV bot detected as recipient".to_string(),
            });
        }

        if tx.priority > 200 && tx.value > 1_000_000_000_000_000_000u64 {
            threats.push(MevThreat {
                threat_type: MevThreatType::Sandwich,
                confidence: 0.7,
                estimated_loss_wei: tx.value / 50,
                attacker_address: None,
                description: "High-value transaction susceptible to sandwich attack".to_string(),
            });
        }

        threats
    }

    pub fn protect_transaction(&mut self, tx: &Transaction) -> ProtectionResult {
        if !self.enabled {
            return ProtectionResult {
                protected: false,
                level: ProtectionLevel::None,
                threats_detected: 0,
                estimated_savings_wei: 0,
                actions_taken: vec!["Protection disabled".to_string()],
            };
        }

        self.protected_txs += 1;
        let threats = self.analyze_transaction(tx);
        let threats_detected = threats.len();

        let mut actions_taken = Vec::new();
        let mut estimated_savings = 0u64;
        let mut protected = true;

        for threat in &threats {
            match threat.threat_type {
                MevThreatType::FrontRun => {
                    actions_taken.push("Route through private mempool".to_string());
                    estimated_savings += threat.estimated_loss_wei;
                }
                MevThreatType::Sandwich => {
                    actions_taken.push("Apply slippage protection".to_string());
                    actions_taken.push("Use Flashbots relay".to_string());
                    estimated_savings += threat.estimated_loss_wei;
                }
                MevThreatType::BackRun => {
                    actions_taken.push("Delay final confirmation".to_string());
                }
                MevThreatType::TimeBandit => {
                    actions_taken.push("Increase gas premium".to_string());
                }
                MevThreatType::Unknown => {
                    actions_taken.push("Apply standard protection".to_string());
                }
            }
        }

        if threats_detected > 0 {
            self.threats_blocked += threats_detected as u64;
            self.total_savings_wei += estimated_savings;
        }

        ProtectionResult {
            protected,
            level: self.default_protection,
            threats_detected,
            estimated_savings_wei: estimated_savings,
            actions_taken,
        }
    }

    pub fn add_known_bot(&mut self, address: String) {
        self.known_bots.insert(address);
    }

    pub fn get_stats(&self) -> String {
        format!(
            r#"{{"protected_txs":{},"threats_blocked":{},"total_savings_wei":{},"known_bots":{}}}"#,
            self.protected_txs,
            self.threats_blocked,
            self.total_savings_wei,
            self.known_bots.len()
        )
    }

    fn is_known_bot(&self, address: &str) -> bool {
        self.known_bots.contains(address)
    }
}
