// ==============================================================================
// UPGRADE4 REWRITE: Private Mempool / Flashbots Bundle Submission
// Module 6: Pre-baked byte arrays + AVX-2 vector register patching
// ==============================================================================

use std::time::Duration;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::simd_state::TransactionTemplate;

// -----------------------------------------------------------------------------
// Public types (kept for external API compatibility)
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedBundle {
    pub transactions: Vec<String>,
    pub target_block: Option<u64>,
    pub max_block: Option<u64>,
    pub replacement_uid: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleResponse {
    pub bundle_hash: String,
    pub accepted: bool,
    pub target_block: u64,
    pub error: Option<String>,
}

// -----------------------------------------------------------------------------
// PrivateMempool — network wrapper + hot-path payload generation
// -----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct PrivateMempool {
    client: Client,
    flashbots_relay_url: String,
    flashbots_auth_key: String,
    fallback_rpc: String,
    tx_template: TransactionTemplate,
}

impl PrivateMempool {
    pub fn new(
        flashbots_relay_url: impl Into<String>,
        flashbots_auth_key: impl Into<String>,
        fallback_rpc: impl Into<String>,
    ) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
            flashbots_relay_url: flashbots_relay_url.into(),
            flashbots_auth_key: flashbots_auth_key.into(),
            fallback_rpc: fallback_rpc.into(),
            tx_template: TransactionTemplate::new(),
        }
    }

    /// Build transaction template during idle time (non-hot-path).
    pub fn build_tx_template(&mut self, skeleton: &[u8; 512]) {
        self.tx_template.build(skeleton);
    }

    /// Patch gas fields into pre-baked template (Module 6 hot path).
    #[inline(always)]
    pub fn patch_gas_fields(&mut self, gas_limit: u64, max_fee: u64, priority_fee: u64) {
        #[cfg(target_feature = "avx2")]
        unsafe {
            self.tx_template.patch_gas_fields_avx2(gas_limit, max_fee, priority_fee);
        }
        #[cfg(not(target_feature = "avx2"))]
        {
            self.tx_template.patch_gas_fields_scalar(gas_limit, max_fee, priority_fee);
        }
    }

    /// Get patched transaction bytes for transmission.
    #[inline(always)]
    pub fn get_patched_tx(&self) -> &[u8; 512] {
        &self.tx_template.bytes
    }

    // -------------------------------------------------------------------------
    // Network methods (non-critical path, retained for compatibility)
    // -------------------------------------------------------------------------

    pub async fn send_bundle(&self, bundle: SignedBundle) -> Result<BundleResponse, String> {
        let body = self.build_send_bundle_body(&bundle);
        let resp = self.client
            .post(&self.flashbots_relay_url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Flashbots relay unreachable: {}", e))?;

        let json: serde_json::Value = resp.json().await
            .map_err(|e| format!("JSON parse failed: {}", e))?;

        let bundle_hash = json.get("bundleHash")
            .and_then(|h| h.as_str())
            .unwrap_or("unknown")
            .to_string();

        let error = json.get("error")
            .and_then(|e| e.as_str())
            .map(|s| s.to_string());

        Ok(BundleResponse {
            bundle_hash,
            accepted: error.is_none(),
            target_block: bundle.target_block.unwrap_or(0),
            error,
        })
    }

    pub async fn send_bundle_protect(&self, bundle: SignedBundle) -> Result<BundleResponse, String> {
        let body = self.build_send_bundle_body(&bundle);
        let resp = self.client
            .post(&self.flashbots_relay_url)
            .header("X-Flashbots-Signature", &self.flashbots_auth_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Flashbots Protect unreachable: {}", e))?;

        let json: serde_json::Value = resp.json().await
            .map_err(|e| format!("JSON parse failed: {}", e))?;

        let bundle_hash = json.get("bundleHash")
            .and_then(|h| h.as_str())
            .unwrap_or("unknown")
            .to_string();

        let error = json.get("error")
            .and_then(|e| e.as_str())
            .map(|s| s.to_string());

        Ok(BundleResponse {
            bundle_hash,
            accepted: error.is_none(),
            target_block: bundle.target_block.unwrap_or(0),
            error,
        })
    }

    pub async fn send_public_fallback(&self, signed_tx: &str) -> Result<String, String> {
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_sendRawTransaction",
            "params": [signed_tx],
            "id": 1
        });

        let resp = self.client
            .post(&self.fallback_rpc)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Fallback RPC failed: {}", e))?;

        let json: serde_json::Value = resp.json().await
            .map_err(|e| format!("JSON parse failed: {}", e))?;

        let tx_hash = json.get("result")
            .and_then(|r| r.as_str())
            .ok_or("No tx hash in response")?;

        Ok(tx_hash.to_string())
    }

    pub async fn get_bundle_status(&self, bundle_hash: &str) -> Result<bool, String> {
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_getBundleStatus",
            "params": [{"bundleHash": bundle_hash}],
            "id": 1
        });

        let resp = self.client
            .post(&self.flashbots_relay_url)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Bundle status check failed: {}", e))?;

        let json: serde_json::Value = resp.json().await
            .map_err(|e| format!("JSON parse failed: {}", e))?;

        let status = json.get("result")
            .and_then(|r| r.get("status"))
            .and_then(|s| s.as_str())
            .unwrap_or("unknown");

        Ok(status == "included" || status == "pending")
    }

    fn build_send_bundle_body(&self, bundle: &SignedBundle) -> serde_json::Value {
        let txs: Vec<serde_json::Value> = bundle.transactions.iter().map(|t| serde_json::json!(t)).collect();
        let mut params: Vec<serde_json::Value> = vec![serde_json::json!(txs)];

        if let Some(target) = bundle.target_block {
            params.push(serde_json::json!({
                "blockNumber": format!("0x{:x}", target),
            }));
        } else {
            params.push(serde_json::json!({
                "blockNumber": "latest",
            }));
        }

        if let Some(uid) = &bundle.replacement_uid {
            params.push(serde_json::json!({
                "replacementUuid": uid,
            }));
        }

        serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_sendBundle",
            "params": params,
            "id": 1
        })
    }
}

// -----------------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tx_template_patch() {
        let mut pm = PrivateMempool::new("http://localhost", "key", "http://fallback");
        let mut skeleton = [0u8; 512];
        skeleton[64..72].copy_from_slice(&0xDEADu64.to_le_bytes());
        skeleton[96..104].copy_from_slice(&0xBEEFu64.to_le_bytes());
        skeleton[128..136].copy_from_slice(&0xCAFEu64.to_le_bytes());
        pm.build_tx_template(&skeleton);
        pm.patch_gas_fields(0x1234, 0x5678, 0x9ABC);

        let gas = u64::from_le_bytes([
            pm.tx_template.bytes[64], pm.tx_template.bytes[65], pm.tx_template.bytes[66],
            pm.tx_template.bytes[67], pm.tx_template.bytes[68], pm.tx_template.bytes[69],
            pm.tx_template.bytes[70], pm.tx_template.bytes[71],
        ]);
        assert_eq!(gas, 0x1234);
    }
}
