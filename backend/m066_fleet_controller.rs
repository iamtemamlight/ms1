#![allow(dead_code)]
use std::sync::Arc;

mod signer {
    pub struct AppState {
        pub k8s_manager: crate::m082_k8s_manager::K8sManager,
        pub signer_ip: String,
    }
}

use signer::AppState;

use crate::metrics::ChainMetrics;
use futures::StreamExt;

/// Orchestrates the fleet scaling based on ROI metrics.
pub async fn rebalance_fleet(state: Arc<AppState>, metrics: Vec<ChainMetrics>, total_target: usize) -> Result<(), String> {
    let total_score: f64 = metrics.iter().map(|m| m.score).sum();
    if total_score <= 0.0 {
        return Err("No profitable chains detected. Scaling to minimum.".into());
    }

    for metric in metrics {
        // Calculate target count for this specific chain
        let target_for_chain = ((metric.score / total_score) * total_target as f64) as usize;
        
        let current_runners = state.k8s_manager.get_runners_for_chain(&metric.chain).await
            .map_err(|e| format!("K8s error on chain {}: {}", metric.chain, e))?;
        
        let current_count = current_runners.len();
        println!("Rebalancing {}: current={}, target={}", metric.chain, current_count, target_for_chain);

        if current_count < target_for_chain {
            // Scale up
            let to_spawn = target_for_chain - current_count;
            
            // Use a buffered stream to avoid overwhelming the K8s API during large scale-ups
            let results = futures::stream::iter(0..to_spawn).map(|_| {
                let id = format!("{:x}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0));
                let state = state.clone();
                let chain = metric.chain.clone();
                async move {
                    match state.k8s_manager.spawn_runner(&id, &chain, &state.signer_ip).await {
                        Ok(_) => {
                            println!("Fleet controller: spawned runner {} on {}", id, chain);
                            Ok(())
                        }
                        Err(e) => Err(e)
                    }
                }
            })
            .buffer_unordered(10)
            .collect::<Vec<_>>().await;
            
            println!("Fleet controller: scaled up {} runners for {}", to_spawn, metric.chain);

            for res in results {
                if let Err(e) = res {
                    eprintln!("Scaling Error (Spawn): {}", e);
                }
            }
        } else if current_count > target_for_chain {
            // Scale down
            let to_terminate = current_count - target_for_chain;
            let results = futures::stream::iter(0..to_terminate).map(|i| {
                let state = state.clone();
                let runner_id = current_runners[i].clone();
                async move {
                    match state.k8s_manager.terminate_runner(&runner_id).await {
                        Ok(_) => {
                            println!("Fleet controller: terminated runner {}", runner_id);
                            Ok(())
                        }
                        Err(e) => Err(e)
                    }
                }
            })
            .buffer_unordered(10) // Protect K8s control plane from request spikes
            .collect::<Vec<_>>().await;

            println!("Fleet controller: scaled down {} runners for {}", to_terminate, metric.chain);

            for res in results {
                if let Err(e) = res {
                    eprintln!("Scaling Error (Terminate): {}", e);
                }
            }
        }
    }

    Ok(())
}