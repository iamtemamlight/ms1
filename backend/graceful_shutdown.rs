// Graceful shutdown handler for clean service termination
use std::sync::Arc;
use tokio::signal;
use tokio::sync::broadcast;
use tracing::{info, warn, error};

#[derive(Clone)]
pub struct ShutdownSignal {
    tx: broadcast::Sender<()>,
}

impl ShutdownSignal {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1);
        Self { tx }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<()> {
        self.tx.subscribe()
    }

    pub fn shutdown(&self) {
        let _ = self.tx.send(());
    }
}

/// Wait for shutdown signals (SIGTERM, SIGINT, CTRL+C)
pub async fn wait_for_shutdown_signal(shutdown: ShutdownSignal) {
    #[cfg(unix)]
    let mut sigterm = match signal::unix::signal(signal::unix::SignalKind::terminate()) {
        Ok(s) => Some(s),
        Err(_) => None,
    };

    #[cfg(unix)]
    let mut sigint = match signal::unix::signal(signal::unix::SignalKind::interrupt()) {
        Ok(s) => Some(s),
        Err(_) => None,
    };

    #[cfg(windows)]
    let mut ctrl_c = match signal::windows::ctrl_c() {
        Ok(s) => Some(s),
        Err(_) => None,
    };

    #[cfg(unix)]
    tokio::select! {
        _ = async {
            if let Some(ref mut s) = sigterm {
                let _ = s.recv().await;
            } else {
                std::future::pending().await
            }
        } => {
            info!("Received SIGTERM signal, initiating graceful shutdown...");
        }
        _ = async {
            if let Some(ref mut s) = sigint {
                let _ = s.recv().await;
            } else {
                std::future::pending().await
            }
        } => {
            info!("Received SIGINT signal, initiating graceful shutdown...");
        }
    }

    #[cfg(windows)]
    tokio::select! {
        _ = async {
            if let Some(ref mut s) = ctrl_c {
                let _ = s.recv().await;
            } else {
                std::future::pending().await
            }
        } => {
            info!("Received CTRL+C, initiating graceful shutdown...");
        }
    }

    shutdown.shutdown();
}

/// Graceful shutdown handler with timeout
pub struct GracefulShutdown {
    shutdown: ShutdownSignal,
    timeout_seconds: u64,
}

impl GracefulShutdown {
    pub fn new(shutdown: ShutdownSignal, timeout_seconds: u64) -> Self {
        Self {
            shutdown,
            timeout_seconds,
        }
    }

    /// Wait for shutdown signal and perform cleanup
    pub async fn wait_and_cleanup<F, Fut>(self, cleanup: F)
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<(), Box<dyn std::error::Error>>>,
    {
        let mut rx = self.shutdown.subscribe();

        tokio::select! {
            _ = rx.recv() => {
                info!("Shutdown signal received, starting graceful shutdown...");

                let cleanup_result = tokio::time::timeout(
                    tokio::time::Duration::from_secs(self.timeout_seconds),
                    cleanup()
                ).await;

                match cleanup_result {
                    Ok(Ok(())) => {
                        info!("Cleanup completed successfully");
                    }
                    Ok(Err(e)) => {
                        error!("Cleanup failed: {}", e);
                    }
                    Err(_) => {
                        warn!("Cleanup timed out after {} seconds", self.timeout_seconds);
                    }
                }

                info!("Graceful shutdown complete");
            }
        }
    }

    /// Subscribe to shutdown signal for custom handlers
    pub fn subscribe(&self) -> broadcast::Receiver<()> {
        self.shutdown.subscribe()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_shutdown_signal() {
        let shutdown = ShutdownSignal::new();
        let mut rx = shutdown.subscribe();

        let shutdown_clone = shutdown.clone();
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            shutdown_clone.shutdown();
        });

        let result = tokio::time::timeout(
            tokio::time::Duration::from_secs(1),
            rx.recv()
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_graceful_shutdown() {
        let shutdown = ShutdownSignal::new();
        let graceful = GracefulShutdown::new(shutdown, 5);

        let cleanup_called = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let cleanup_clone = cleanup_called.clone();

        let graceful_clone = graceful.clone();
        tokio::spawn(async move {
            graceful_clone.wait_and_cleanup(|| async {
                cleanup_clone.store(true, std::sync::atomic::Ordering::SeqCst);
                Ok(())
            }).await;
        });

        shutdown.shutdown();

        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        assert!(cleanup_called.load(std::sync::atomic::Ordering::SeqCst));
    }
}
