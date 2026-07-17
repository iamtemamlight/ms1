# Graceful Shutdown Integration Guide

The `graceful_shutdown.rs` module has been created with comprehensive graceful shutdown functionality. This guide explains how to integrate it into `main.rs`.

## Manual Integration Steps

### Step 1: Add Module Declaration

Add this line to the module declarations section in `main.rs` (around line 274):

```rust
mod graceful_shutdown;
```

### Step 2: Add Import

Add this import after the existing imports (around line 275):

```rust
use graceful_shutdown::{GracefulShutdown, ShutdownSignal, wait_for_shutdown_signal};
```

### Step 3: Initialize Shutdown Signal

In the `run_server` function, initialize the shutdown signal before starting servers:

```rust
let shutdown_signal = ShutdownSignal::new();
```

### Step 4: Modify Server Startup

Replace the existing server startup code (around line 2659) with graceful shutdown support:

```rust
// Run both gRPC and HTTP servers concurrently with graceful shutdown
let grpc_addr: std::net::SocketAddr = addr.parse::<std::net::SocketAddr>()
    .map_err(|e| AppError::GrpcBind(e.to_string()))?;
let grpc_server = builder
    .add_service(FleetCommandServer::new(c2_server))
    .serve_with_graceful_shutdown(grpc_addr, async {
        let mut rx = shutdown_signal.subscribe();
        let _ = rx.recv().await;
        info!("gRPC server graceful shutdown initiated");
    });

let http_server = axum::serve::serve_with_graceful_shutdown(
    listener,
    http_router.into_make_service(),
    async {
        let mut rx = shutdown_signal.subscribe();
        let _ = rx.recv().await;
        info!("HTTP server graceful shutdown initiated");
    }
);

// Wait for shutdown signal
tokio::select! {
    result = http_server => {
        if let Err(e) = result {
            warn!("HTTP server error: {}", e);
        }
    }
    result = grpc_server => {
        if let Err(e) = result {
            warn!("gRPC server error: {}", e);
        }
    }
}

Ok(())
```

### Step 5: Add Shutdown Signal Handler

Add this before the server startup code to handle shutdown signals:

```rust
// Spawn shutdown signal handler
let shutdown_signal_clone = shutdown_signal.clone();
tokio::spawn(async move {
    wait_for_shutdown_signal(shutdown_signal_clone).await;
});
```

## Cleanup Handlers

You can add custom cleanup logic using the GracefulShutdown struct:

```rust
let graceful = GracefulShutdown::new(shutdown_signal.clone(), 30); // 30 second timeout

tokio::spawn(async move {
    graceful.wait_and_cleanup(|| async {
        // Custom cleanup logic here
        info!("Performing cleanup...");
        
        // Close database connections
        // Flush logs
        // Save state
        // Notify other services
        
        Ok(())
    }).await;
});
```

## Signal Handling

The graceful shutdown module handles:

- **SIGTERM** (Unix) - Standard termination signal
- **SIGINT** (Unix) - Interrupt signal (Ctrl+C)
- **CTRL+C** (Windows) - Windows-specific handling

## Timeout Configuration

Configure the shutdown timeout based on your needs:

```rust
// 30 second timeout for cleanup
let graceful = GracefulShutdown::new(shutdown_signal, 30);

// 60 second timeout for long-running cleanup
let graceful = GracefulShutdown::new(shutdown_signal, 60);
```

## Testing

Test graceful shutdown by sending signals:

```bash
# Send SIGTERM (Unix)
kill -TERM <pid>

# Send SIGINT (Unix)
kill -INT <pid>

# Press CTRL+C in terminal
```

## Kubernetes Integration

Add termination grace period to your Kubernetes deployment:

```yaml
spec:
  terminationGracePeriodSeconds: 30
  containers:
    - name: backend
      lifecycle:
        preStop:
          exec:
            command: ["/bin/sh", "-c", "curl -X POST http://localhost:3000/shutdown"]
```

## Docker Compose Integration

Add stop grace period to docker-compose.yml:

```yaml
backend:
  stop_grace_period: 30s
```

## Best Practices

1. **Set appropriate timeouts** - Balance between cleanup time and fast shutdown
2. **Log shutdown progress** - Monitor shutdown process in production
3. **Test shutdown regularly** - Ensure cleanup works correctly
4. **Handle cleanup failures** - Log errors but allow shutdown to complete
5. **Save critical state** - Ensure data persistence before shutdown

## Dependencies

The graceful_shutdown module requires these dependencies (already in Cargo.toml):
- `tokio` for async operations and signal handling
- `tracing` for logging
- `tokio::sync::broadcast` for signal propagation

## Notes

- Shutdown signals are broadcast to all subscribers
- Cleanup operations have configurable timeout
- Failed cleanup is logged but doesn't block shutdown
- Windows and Unix signal handling are both supported
- Multiple cleanup handlers can be spawned concurrently
