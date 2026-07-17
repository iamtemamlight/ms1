# Health Check Integration Guide

The `health_checks.rs` module has been created with comprehensive health check functionality. This guide explains how to integrate it into `main.rs`.

## Manual Integration Steps

### Step 1: Add Module Declaration

Add this line to the module declarations section in `main.rs` (around line 274):

```rust
mod health_checks;
```

### Step 2: Add Import

Add this import after the existing imports (around line 275):

```rust
use health_checks::HealthChecker;
```

### Step 3: Initialize Health Checker

In the `run_server` function, initialize the health checker with database pool and RPC URL:

```rust
// After database pool initialization
let health_checker = HealthChecker::new(
    Some(db_pool.clone()),
    std::env::var("RPC_ENDPOINT").ok()
);
```

### Step 4: Add Health Check Routes

Add these routes to the HTTP router (around line 2614, after existing routes):

```rust
.route("/health", get(|| async {
    axum::Json(health_checker.comprehensive_health().await)
}))
.route("/health/live", get(|| async {
    axum::Json(health_checker.liveness().await)
}))
.route("/health/ready", get(|| async {
    axum::Json(health_checker.readiness().await)
}))
.route("/health/database", get(|| async {
    axum::Json(health_checker.check_database().await)
}))
.route("/health/rpc", get(|| async {
    axum::Json(health_checker.check_rpc().await)
}))
```

## Health Check Endpoints

Once integrated, the following endpoints will be available:

### GET /health
Comprehensive health check including all subsystems.

**Response:**
```json
{
  "status": "healthy",
  "timestamp": "2026-07-15T14:30:00Z",
  "checks": {
    "database": {
      "status": "pass",
      "message": "Database connection successful",
      "duration_ms": 15
    },
    "rpc": {
      "status": "pass",
      "message": "RPC endpoint responsive",
      "duration_ms": 23
    },
    "memory": {
      "status": "pass",
      "message": "Memory usage: 45.2%",
      "duration_ms": 1
    },
    "ai_agents": {
      "status": "pass",
      "message": "AI agents operational",
      "duration_ms": 2
    }
  }
}
```

### GET /health/live
Liveness check - is the service running?

**Response:**
```json
{
  "status": "healthy",
  "timestamp": "2026-07-15T14:30:00Z",
  "checks": {
    "server": {
      "status": "pass",
      "message": "Server is running",
      "duration_ms": 0
    }
  }
}
```

### GET /health/ready
Readiness check - is the service ready to accept traffic?

**Response:**
```json
{
  "status": "ready",
  "timestamp": "2026-07-15T14:30:00Z",
  "checks": {
    "database": {
      "status": "pass",
      "message": "Database connection successful",
      "duration_ms": 15
    },
    "rpc": {
      "status": "pass",
      "message": "RPC endpoint responsive",
      "duration_ms": 23
    }
  }
}
```

### GET /health/database
Database-specific health check.

### GET /health/rpc
RPC endpoint-specific health check.

## Kubernetes Integration

Add these probes to your Kubernetes deployment:

```yaml
livenessProbe:
  httpGet:
    path: /health/live
    port: 3000
  initialDelaySeconds: 30
  periodSeconds: 10
  timeoutSeconds: 5
  failureThreshold: 3

readinessProbe:
  httpGet:
    path: /health/ready
    port: 3000
  initialDelaySeconds: 10
  periodSeconds: 5
  timeoutSeconds: 3
  failureThreshold: 3
```

## Docker Compose Integration

Add health check to docker-compose.yml:

```yaml
backend:
  healthcheck:
    test: ["CMD", "curl", "-f", "http://localhost:3000/health/live"]
    interval: 30s
    timeout: 10s
    retries: 3
    start_period: 40s
```

## Testing

Test the endpoints after integration:

```bash
# Test comprehensive health
curl http://localhost:3000/health

# Test liveness
curl http://localhost:3000/health/live

# Test readiness
curl http://localhost:3000/health/ready

# Test database
curl http://localhost:3000/health/database

# Test RPC
curl http://localhost:3000/health/rpc
```

## Dependencies

The health_checks module requires these dependencies (already in Cargo.toml):
- `sqlx` for database checks
- `reqwest` for RPC checks
- `chrono` for timestamps
- `serde` for JSON serialization
- `tokio` for async operations

## Notes

- Database checks timeout after 5 seconds
- RPC checks timeout after 5 seconds
- Memory usage check is Linux-specific (returns 0.0 on other platforms)
- All checks return duration in milliseconds for performance monitoring
- Status values: "pass", "fail", "skip", "warn"
