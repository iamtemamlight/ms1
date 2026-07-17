# Audit Logging Integration Guide

The `audit_logging.rs` module provides comprehensive audit logging for production security and compliance. This guide explains how to integrate it into the AllBright system.

## Manual Integration Steps

### Step 1: Add Module Declaration

Add this line to the module declarations section in `main.rs` (around line 274):

```rust
mod audit_logging;
```

### Step 2: Add Import

Add this import after the existing imports (around line 275):

```rust
use audit_logging::{AuditLogger, AuditEvent, AuditResult};
```

### Step 3: Initialize Audit Logger

In the `run_server` function, initialize the audit logger after environment validation:

```rust
// Initialize audit logger
let audit_logger = Arc::new(AuditLogger::new());

// Start cleanup task for old events
let audit_cleanup = audit_logger.clone();
tokio::spawn(async move {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(86400)); // Daily
    loop {
        interval.tick().await;
        audit_cleanup.cleanup_old_events().await;
    }
});
```

### Step 4: Add Audit Logging API Endpoints

Add these routes to the HTTP router (around line 2614):

```rust
.route("/api/audit/events", get(get_audit_events))
.route("/api/audit/events/user/:user_id", get_get_audit_events_by_user)
.route("/api/audit/events/action/:action", get_get_audit_events_by_action)
.route("/api/audit/statistics", get(get_audit_statistics))
.route("/api/audit/cleanup", post(cleanup_audit_events))
```

### Step 5: Implement API Handlers

Add these handler functions to main.rs:

```rust
async fn get_audit_events(
    State(audit_logger): State<Arc<AuditLogger>>,
    Query(params): Query<AuditQueryParams>
) -> Json<Vec<AuditEvent>> {
    let limit = params.limit.unwrap_or(100);
    Json(audit_logger.get_events(limit).await)
}

async fn get_get_audit_events_by_user(
    State(audit_logger): State<Arc<AuditLogger>>,
.Path(user_id): Path<String>,
    Query(params): Query<AuditQueryParams>
) -> Json<Vec<AuditEvent>> {
    let limit = params.limit.unwrap_or(100);
    Json(audit_logger.get_events_by_user(&user_id, limit).await)
}

async fn get_get_audit_events_by_action(
    State(audit_logger): State<Arc<AuditLogger>>,
    Path(action): Path<String>,
    Query(params): Query<AuditQueryParams>
) -> Json<Vec<AuditEvent>> {
    let limit = params.limit.unwrap_or(100);
    Json(audit_logger.get_events_by_action(&action, limit).await)
}

async fn get_audit_statistics(
    State(audit_logger): State<Arc<AuditLogger>>
) -> Json<AuditStatistics> {
    Json(audit_logger.get_statistics().await)
}

async fn cleanup_audit_events(
    State(audit_logger): State<Arc<AuditLogger>>
) -> Json<serde_json::Value> {
    audit_logger.cleanup_old_events().await;
    Json(json!({"success": true, "message": "Audit events cleaned up"}))
}
```

### Step 6: Add Query Parameters

Add these types to your request/response module:

```rust
#[derive(Deserialize)]
struct AuditQueryParams {
    limit: Option<usize>,
}
```

### Step 7: Add Audit Logging to Critical Operations

Add audit logging to critical operations throughout the codebase:

```rust
// Example: Mode execution
async fn execute_mode(
    State(audit_logger): State<Arc<AuditLogger>>,
    Json(req): Json<ExecuteModeRequest>
) -> Json<serde_json::Value> {
    let event = AuditLogger::create_event(
        "MODE_EXECUTE".to_string(),
        Some(req.mode.clone()),
        AuditResult::Success,
        Some(req.user_id.clone()),
        Some(req.ip_address.clone()),
        Some(req.user_agent.clone()),
        serde_json::json!({"mode": req.mode, "params": req.params}),
    );
    
    audit_logger.log(event).await;
    
    // Continue with mode execution...
}

// Example: 2FA verification
async fn verify_2fa_challenge(
    State(audit_logger): State<Arc<AuditLogger>>,
    Json(req): Json<VerifyChallengeRequest>
) -> Json<TwoFactorVerification> {
    let result = auth.verify_challenge(req.challenge_id.clone(), req.code.clone()).await;
    
    let audit_result = match &result {
        Ok(v) if v.success => AuditResult::Success,
        _ => AuditResult::Failure,
    };
    
    let event = AuditLogger::create_event(
        "2FA_VERIFY".to_string(),
        Some(req.challenge_id.clone()),
        audit_result,
        None,
        None,
        None,
        serde_json::json!({"challenge_id": req.challenge_id}),
    );
    
    audit_logger.log(event).await;
    
    // Return result...
}
```

## Environment Configuration

Add these variables to your `.env` file:

```bash
# Enable audit logging
AUDIT_LOGGING_ENABLED=true

# Log to file
AUDIT_LOG_TO_FILE=false

# Log to database
AUDIT_LOG_TO_DATABASE=true

# Log to external service (Datadog, Splunk, etc.)
AUDIT_LOG_TO_EXTERNAL=false

# Retention period in days (default: 90)
AUDIT_LOG_RETENTION_DAYS=90
```

## API Endpoints

### GET /api/audit/events?limit=100

Get recent audit events.

**Response:**
```json
[
  {
    "timestamp": "2026-07-15T12:00:00Z",
    "event_id": "uuid-here",
    "user_id": "user123",
    "action": "MODE_EXECUTE",
    "resource": "LIVE",
    "result": "Success",
    "ip_address": "192.168.1.100",
    "user_agent": "Mozilla/5.0...",
    "metadata": {"mode": "LIVE"}
  }
]
```

### GET /api/audit/events/user/:user_id?limit=100

Get audit events for a specific user.

### GET /api/audit/events/action/:action?limit=100

Get audit events for a specific action.

### GET /api/audit/statistics

Get audit statistics.

**Response:**
```json
{
  "total_events": 1000,
  "success_events": 850,
  "failure_events": 120,
  "warning_events": 30,
  "unique_users": 50,
  "unique_actions": 25
}
```

### POST /api/audit/cleanup

Manually trigger cleanup of old events.

**Response:**
```json
{
  "success": true,
  "message": "Audit events cleaned up"
}
```

## Critical Actions to Log

Log these critical actions:

- **MODE_EXECUTE** - Engine mode execution
- **2FA_VERIFY** - 2FA verification attempts
- **SECRET_ROTATE** - Secret rotation events
- **IP_WHITELIST_MODIFY** - IP whitelist modifications
- **WALLET_ACCESS** - Wallet access attempts
- **LIVE_TRADING** - Live trading operations
- **CONFIG_CHANGE** - Configuration changes
- **USER_LOGIN** - User authentication
- **API_ACCESS** - API access attempts

## Production Integration

For production deployment, integrate with external logging services:

```rust
use audit_logging::AuditLogger;

// Send to Datadog
async fn log_to_datadog(event: &AuditEvent) {
    let client = reqwest::Client::new();
    client.post("https://http-intake.logs.datadoghq.com/v1/input")
        .header("DD-API-KEY", std::env::var("DATADOG_API_KEY").unwrap())
        .json(event)
        .send()
        .await
        .ok();
}

// Send to Splunk
async fn log_to_splunk(event: &AuditEvent) {
    let client = reqwest::Client::new();
    client.post("https://splunk-hec.example.com:8088/services/collector/event")
        .header("Authorization", format!("Splunk {}", std::env::var("SPLUNK_TOKEN").unwrap()))
        .json(event)
        .send()
        .await
        .ok();
}
```

## Database Integration

For persistent audit logging, integrate with your database:

```rust
use sqlx::PgPool;

async fn log_to_database(pool: &PgPool, event: &AuditEvent) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO audit_logs (
            event_id, user_id, action, resource, result,
            ip_address, user_agent, metadata, timestamp
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
        event.event_id,
        event.user_id,
        event.action,
        event.resource,
        event.result as AuditResult,
        event.ip_address,
        event.user_agent,
        event.metadata,
        event.timestamp
    )
    .execute(pool)
    .await?;
    
    Ok(())
}
```

## Best Practices

1. **Log all critical operations** - Mode execution, 2FA, wallet access
2. **Include context** - User ID, IP address, user agent
3. **Use structured metadata** - JSON metadata for additional context
4. **Regular cleanup** - Daily cleanup based on retention policy
3. **Monitor failure rates** - Alert on high failure rates
4. **Compliance requirements** - Meet SOC 2, ISO 27001 audit requirements
5. **Secure storage** - Encrypt audit logs at rest
6. **Immutable logs** - Prevent modification of audit logs

## Security Notes

- Audit logs are enabled by default
- Events include unique IDs for traceability
- Failed operations are logged with error context
- IP addresses and user agents are captured
- Retention policy prevents unlimited growth
- Logs can be exported for compliance audits

## Dependencies

The audit_logging module requires these dependencies (already in Cargo.toml):
- `tokio` for async operations
- `chrono` for date/time handling
- `serde` for JSON serialization
- `uuid` for unique event IDs
- `tracing` for logging

## Testing

Test the audit logging:

```bash
# Enable audit logging
export AUDIT_LOGGING_ENABLED=true

# Get events
curl http://localhost:3000/api/audit/events

# Get events by user
curl http://localhost:3000/api/audit/events/user/user123

# Get events by action
curl http://localhost:3000/api/audit/events/action/MODE_EXECUTE

# Get statistics
curl http://localhost:3000/api/audit/statistics

# Trigger cleanup
curl -X POST http://localhost:3000/api/audit/cleanup
```

## Monitoring

Monitor these metrics:
- Total audit events per hour
- Success/failure ratio
- Unique user activity
- Action frequency
- Failed authentication attempts
- IP whitelist modifications
- Secret rotation events
