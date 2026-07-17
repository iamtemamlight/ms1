#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use reqwest::Client;
use std::time::{Duration, Instant};

#[derive(Debug, Serialize, Deserialize)]
pub struct ChainMetrics {
    pub chain: String,
    pub score: f64,
}

/// Queries Prometheus for the profit/gas ratio per chain over the last hour.
pub async fn fetch_chain_metrics(prometheus_url: &str) -> Result<Vec<ChainMetrics>, Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::new();
    // Query: Profit per hour divided by Gas cost per hour
    let query = r#"sum by (chain) (rate(runner_profit_usd[1h])) / sum by (chain) (rate(runner_gas_cost_usd[1h]))"#;
    let url = format!("{}/api/v1/query", prometheus_url);
    
    let resp: serde_json::Value = client.get(&url)
        .timeout(Duration::from_secs(5))
        .query(&[("query", query)])
        .send().await?
        .json().await?;

    let mut metrics = Vec::new();
    if let Some(results) = resp.get("data").and_then(|d| d.get("result")).and_then(|r| r.as_array()) {
        for res in results {
            let chain = res.get("metric")
                .and_then(|m| m.get("chain"))
                .and_then(|c| c.as_str())
                .unwrap_or("unknown").to_string();
            
            let value_str = res.get("value")
                .and_then(|v| v.get(1))
                .and_then(|v| v.as_str())
                .unwrap_or("0");
            let score: f64 = value_str.parse().unwrap_or(0.0);
            metrics.push(ChainMetrics { chain, score });
        }
    }
    Ok(metrics)
}

/// Measures the latency of a blockchain RPC endpoint.
pub async fn check_rpc_latency(chain: &str, url: &str) -> crate::ChainHealth {
    let client = Client::new();
    let start = Instant::now();
    
    // Simple eth_blockNumber request to measure round-trip time
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_blockNumber",
        "params": [],
        "id": 1
    });

    let resp = client.post(url)
        .timeout(Duration::from_secs(3))
        .json(&body)
        .send().await;

    let latency = start.elapsed().as_millis() as u64;
    let healthy = resp.is_ok();

    crate::ChainHealth {
        chain: chain.to_string(),
        latency_ms: if healthy { latency } else { 0 },
        healthy,
    }
}