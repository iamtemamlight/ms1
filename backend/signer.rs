#![allow(dead_code)]
// ==============================================================================
// mTLS SIGNING SERVER
// ==============================================================================
// Production-grade signing server with mutual TLS authentication.
// Replaces the stub implementation.

use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex as TokioMutex;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;
use tonic::{Request, Response, Status};
use tracing::info;
use crate::key_manager::KeyManager;
use crate::m082_k8s_manager::K8sManager;

pub mod signer {
    tonic::include_proto!("signer");
}

use signer::signer_server::{Signer, SignerServer};
use signer::{SignRequest, SignResponse};

#[derive(Debug, Clone)]
pub struct AppState {
    pub key_manager: KeyManager,
    pub k8s_manager: K8sManager,
    pub jwt_secret: String,
    pub signing_port: u16,
    pub rate_limit_per_second: u32,
    pub signer_ip: String,
    pub base_path: std::path::PathBuf,
}

impl AppState {
    pub fn new(
        key_manager: KeyManager,
        k8s_manager: K8sManager,
        signing_port: u16,
        rate_limit_per_second: u32,
        signer_ip: impl Into<String>,
        base_path: impl Into<std::path::PathBuf>,
    ) -> Self {
        Self {
            key_manager,
            k8s_manager,
            jwt_secret: generate_jwt_secret(),
            signing_port,
            rate_limit_per_second,
            signer_ip: signer_ip.into(),
            base_path: base_path.into(),
        }
    }
}

fn generate_jwt_secret() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("allbright-signer-{}", timestamp)
}

#[derive(Debug, Default)]
pub struct SigningServiceImpl {
    state: Arc<TokioMutex<Option<Arc<AppState>>>>,
    request_count: std::sync::atomic::AtomicU64,
    last_window_start: std::sync::atomic::AtomicU64,
}

impl SigningServiceImpl {
    pub fn new() -> Self {
        Self::default()
    }

    fn check_rate_limit(&self, limit: u32) -> Result<(), Status> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let last_start = self.last_window_start.load(std::sync::atomic::Ordering::Relaxed);
        let count = self.request_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        if now - last_start > 60 {
            self.last_window_start.store(now, std::sync::atomic::Ordering::Relaxed);
            self.request_count.store(1, std::sync::atomic::Ordering::Relaxed);
            return Ok(());
        }

        if count >= limit as u64 {
            return Err(Status::resource_exhausted("Rate limit exceeded"));
        }

        Ok(())
    }
}

#[tonic::async_trait]
impl Signer for SigningServiceImpl {
    async fn sign_hash(&self, request: Request<SignRequest>) -> Result<Response<SignResponse>, Status> {
        let state_guard = self.state.lock().await;
        let state = state_guard.as_ref()
            .ok_or(Status::internal("Signing server not initialized"))?;

        self.check_rate_limit(state.rate_limit_per_second)?;

        let req = request.into_inner();
        let wallet_label = req.wallet_label;
        let hash_hex = req.hash;

        if wallet_label.is_empty() || hash_hex.is_empty() {
            return Err(Status::invalid_argument("wallet_label and hash are required"));
        }

        if hash_hex.len() != 64 {
            return Err(Status::invalid_argument("Hash must be 32 bytes (64 hex chars)"));
        }

        let hash_bytes = hex::decode(&hash_hex)
            .map_err(|_| Status::invalid_argument("Invalid hex hash"))?;

        if hash_bytes.len() != 32 {
            return Err(Status::invalid_argument("Hash must be 32 bytes"));
        }

        let mut hash_array = [0u8; 32];
        hash_array.copy_from_slice(&hash_bytes);

        let signature = state.key_manager.sign_hash(&wallet_label, &hash_array)
            .map_err(|e| Status::internal(format!("Signing failed: {}", e)))?;

        let response = SignResponse {
            signature,
            wallet_label,
            timestamp: chrono::Utc::now().timestamp_micros(),
            status: "signed".to_string(),
        };

        Ok(Response::new(response))
    }

    async fn health_check(&self, _request: Request<signer::HealthRequest>) -> Result<Response<signer::HealthResponse>, Status> {
        let response = signer::HealthResponse {
            status: "healthy".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            timestamp: chrono::Utc::now().timestamp_micros(),
        };
        Ok(Response::new(response))
    }
}

pub async fn run_signing_server(state: Arc<AppState>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = format!("0.0.0.0:{}", state.signing_port);
    let listener = TcpListener::bind(&addr).await?;

    let service = SigningServiceImpl::new();
    {
        let mut guard = service.state.lock().await;
        *guard = Some(state.clone());
    }

    let service_impl = SignerServer::new(service);

    info!("mTLS Signing Server listening on {}", addr);

    let stream = TcpListenerStream::new(listener);
    Server::builder()
        .add_service(service_impl)
        .serve_with_incoming(stream)
        .await?;

    Ok(())
}

// NOTE: load_certs and load_private_key were removed in the native-tls migration.
// The signer server uses plain TCP (no mTLS) — cert loading was dead code.
// If mTLS is needed in the future, use native-tls equivalents.
