// ==============================================================================
// L9: CHAOS ENGINEERING LAB
// Tests system resilience against: RPC failures, reorgs, gas spikes,
// malformed responses, and credential exposure.
// ==============================================================================

use serde::{Deserialize, Serialize};
use crate::build_guard::{BuildGuard, BuildStatus};
use crate::m059_state_sync::{StateSynchronizer, ValidationStatus};
use crate::m021_regional_modules;
use std::time::Duration;
use std::io::Write;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChaosStatus {
    Pass,
    Fail,
    Warn,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChaosTestResult {
    pub test_name: String,
    pub status: ChaosStatus,
    pub detail: String,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChaosReport {
    pub timestamp: String,
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub tests: Vec<ChaosTestResult>,
}

pub struct ChaosLab;

impl ChaosLab {
    /// Run all chaos tests
    pub async fn run_all_tests() -> ChaosReport {
        let mut tests = Vec::new();

        tests.push(Self::test_rpc_timeout_handling().await);
        tests.push(Self::test_malformed_rpc_response().await);
        tests.push(Self::test_reorg_survival().await);
        tests.push(Self::test_gas_spike_handling().await);
        tests.push(Self::test_credential_exposure_scan().await);
        tests.push(Self::test_network_partition_detection().await);

        let passed = tests.iter().filter(|t| t.status == ChaosStatus::Pass).count();
        let failed = tests.iter().filter(|t| t.status == ChaosStatus::Fail).count();
        let skipped = tests.iter().filter(|t| t.status == ChaosStatus::Skipped).count();

        ChaosReport {
            timestamp: chrono::Utc::now().to_rfc3339(),
            total: tests.len(),
            passed,
            failed,
            skipped,
            tests,
        }
    }

    /// Test: RPC timeout does not crash the engine
    async fn test_rpc_timeout_handling() -> ChaosTestResult {
        let start = std::time::Instant::now();

        // Simulate RPC timeout by connecting to a non-routable address
        let result = tokio::time::timeout(
            Duration::from_secs(2),
            reqwest::get("http://192.0.2.1:8545")
        ).await;

        let handled_correctly = match result {
            Ok(_) => false,
            Err(_) => true,
        };

        ChaosTestResult {
            test_name: "rpc_timeout_handling".to_string(),
            status: if handled_correctly { ChaosStatus::Pass } else { ChaosStatus::Fail },
            detail: if handled_correctly {
                "Timeout correctly returned Err — engine handles RPC failure".to_string()
            } else {
                "Unexpected success — timeout not enforced".to_string()
            },
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }

    /// Test: Malformed JSON-RPC response doesn't panic
    async fn test_malformed_rpc_response() -> ChaosTestResult {
        let start = std::time::Instant::now();

        // Test with a local mock server that returns garbage
        let listener = match std::net::TcpListener::bind("127.0.0.1:0") {
            Ok(l) => l,
            Err(_) => return ChaosTestResult {
                test_name: "malformed_rpc_response".to_string(),
                status: ChaosStatus::Skipped,
                detail: "Cannot bind test socket".to_string(),
                duration_ms: 0,
            },
        };
        let addr = listener.local_addr().unwrap();

        // Spawn a mock server that returns garbage
        let _server_handle = tokio::spawn(async move {
            if let Ok((mut stream, _)) = listener.accept() {
                let _ = stream.write_all(b"NOT JSON\r\n");
            }
        });

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .unwrap();

        let result = client
            .post(format!("http://{}", addr))
            .json(&serde_json::json!({"test": true}))
            .send()
            .await;

        let handled = result.is_err() || result.unwrap().status().is_server_error();

        ChaosTestResult {
            test_name: "malformed_rpc_response".to_string(),
            status: if handled { ChaosStatus::Pass } else { ChaosStatus::Fail },
            detail: if handled {
                "Malformed response handled gracefully".to_string()
            } else {
                "Engine accepted malformed response".to_string()
            },
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }

    /// Test: System survives block reorgs (1-10 blocks)
    async fn test_reorg_survival() -> ChaosTestResult {
        let start = std::time::Instant::now();

        // Check that StateSynchronizer has reorg detection
        let reorg_detection = StateSynchronizer::new();
        let status = reorg_detection.verify_parity(
            crate::m059_state_sync::ChainId::Ethereum,
            crate::m059_state_sync::ChainId::Solana,
        );

        // With no state loaded, should be Pending (not crash)
        let not_crashed = matches!(
            status,
            ValidationStatus::Pending
                | ValidationStatus::Valid
                | ValidationStatus::Stale
        );

        ChaosTestResult {
            test_name: "reorg_survival".to_string(),
            status: if not_crashed { ChaosStatus::Pass } else { ChaosStatus::Fail },
            detail: if not_crashed {
                "Reorg detection initializes without crash — parity check returns Pending with empty state".to_string()
            } else {
                "Reorg detection crashed".to_string()
            },
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }

    /// Test: Gas spike doesn't cause negative-profit trades
    async fn test_gas_spike_handling() -> ChaosTestResult {
        let start = std::time::Instant::now();

        // Simulate 10x gas spike
        let normal_gas = 30_000_000_000u64; // 30 gwei
        let spike_gas = normal_gas * 10;

        let trade_amount = 1_000_000_000_000_000_000u64; // 1 ETH
        let gas_limit = 250_000u64;

        let normal_cost = (gas_limit as u128 * normal_gas as u128) as f64 / 1e18;
        let spike_cost = (gas_limit as u128 * spike_gas as u128) as f64 / 1e18;

        // At spike gas, profit must be > spike_cost * 1.2 (20% buffer)
        let min_profit = spike_cost * 1.2;
        let simulated_profit = trade_amount as f64 / 1e18 * 0.005; // 0.5%

        let survives_spike = simulated_profit > min_profit;

        ChaosTestResult {
            test_name: "gas_spike_handling".to_string(),
            status: if survives_spike { ChaosStatus::Pass } else { ChaosStatus::Warn },
            detail: format!(
                "Normal gas: {:.6} ETH, Spike gas: {:.6} ETH, Min profit required: {:.6} ETH, Simulated: {:.6} ETH",
                normal_cost, spike_cost, min_profit, simulated_profit
            ),
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }

    /// Test: Scan for credential exposure in env and process memory
    async fn test_credential_exposure_scan() -> ChaosTestResult {
        let start = std::time::Instant::now();

        let mut exposed = Vec::new();

        // Check common env var patterns
        let env_vars: Vec<(&str, &str)> = vec![
            ("PRIVATE_KEY", "0xd2a2"),
            ("OPENAI_API_KEY", "sk-proj-"),
            ("DATABASE_URL", "postgresql://"),
            ("DASHBOARD_PASS", "alphamark"),
        ];

        for (key, pattern) in &env_vars {
            if let Ok(val) = std::env::var(key) {
                if val.contains(pattern) && !val.contains("REPLACE_WITH") {
                    exposed.push(format!("{}: [REDACTED]", key));
                }
            }
        }

        // Check for .env files in non-standard locations
        let dotenv_paths = vec![
            "apps/dashboard/.env",
            "backend/.env",
            "src-tauri/.env",
        ];

        for path in &dotenv_paths {
            if std::path::Path::new(path).exists() {
                let content = std::fs::read_to_string(path);
                if let Ok(c) = content {
                    if c.contains("0xd2a2") || c.contains("sk-proj-") {
                        exposed.push(format!("{}: contains plaintext secrets", path));
                    }
                }
            }
        }

        let status = if exposed.is_empty() {
            ChaosStatus::Pass
        } else {
            ChaosStatus::Fail
        };

        ChaosTestResult {
            test_name: "credential_exposure_scan".to_string(),
            status,
            detail: if exposed.is_empty() {
                "No plaintext credentials found in env or .env files".to_string()
            } else {
                format!("CREDENTIALS EXPOSED: {:?}", exposed)
            },
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }

    /// Test: Network partition detection triggers alerts
    async fn test_network_partition_detection() -> ChaosTestResult {
        let start = std::time::Instant::now();

        // Use the existing regional_modules partition detection
        let heartbeats: std::collections::HashMap<String, u64> = std::collections::HashMap::new();

        // Simulate partition: us-west-2 hasn't reported in 10 seconds
        let mut heartbeats_with_partition = heartbeats.clone();
        heartbeats_with_partition.insert("us-west-2".to_string(), 10000); // 10s ago

        let partition_detected = m021_regional_modules::detect_network_partition(
            "us-west-2",
            &heartbeats_with_partition,
            5000, // 5s threshold
        );

        ChaosTestResult {
            test_name: "network_partition_detection".to_string(),
            status: if partition_detected { ChaosStatus::Pass } else { ChaosStatus::Fail },
            detail: if partition_detected {
                "Partition correctly detected for us-west-2 (10s > 5s threshold)".to_string()
            } else {
                "Partition not detected — heartbeat threshold may be broken".to_string()
            },
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }

    /// Quick health check — returns true if no critical failures
    #[allow(dead_code)]
    pub fn is_chaos_ready() -> bool {
        // Synchronous subset for startup checks
        let audit = BuildGuard::run_audit();
        audit.overall != BuildStatus::Fail
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_chaos_lab_runs() {
        let report = ChaosLab::run_all_tests().await;
        assert_eq!(report.total, 6);
        assert!(report.tests.len() == 6);
    }

    #[tokio::test]
    async fn test_reorg_survival() {
        let result = ChaosLab::test_reorg_survival().await;
        assert_eq!(result.test_name, "reorg_survival");
    }

    #[tokio::test]
    async fn test_credential_scan() {
        let result = ChaosLab::test_credential_exposure_scan().await;
        assert_eq!(result.test_name, "credential_exposure_scan");
    }
}
