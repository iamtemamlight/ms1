# Secret Rotation Integration Guide

The `secret_rotation.rs` module provides automated secret rotation for production security. This guide explains how to integrate it into the AllBright system.

## Manual Integration Steps

### Step 1: Add Module Declaration

Add this line to the module declarations section in `main.rs` (around line 274):

```rust
mod secret_rotation;
```

### Step 2: Add Import

Add this import after the existing imports (around line 275):

```rust
use secret_rotation::SecretRotationManager;
```

### Step 3: Initialize Secret Rotation Manager

In the `run_server` function, initialize the secret rotation manager after environment validation:

```rust
// Initialize secret rotation manager (30-day default interval)
let secret_manager = Arc::new(SecretRotationManager::new(720)); // 720 hours = 30 days

// Add secrets to rotation manager
if let Ok(openai_key) = std::env::var("OPENAI_API_KEY") {
    secret_manager.add_secret("OPENAI_API_KEY".to_string(), openai_key, Some(720)).await;
}

if let Ok(groq_key) = std::env::var("GROQ_API_KEY") {
    secret_manager.add_secret("GROQ_API_KEY".to_string(), groq_key, Some(720)).await;
}

if let Ok(openrouter_key) = std::env::var("OPENROUTER_API_KEY") {
    secret_manager.add_secret("OPENROUTER_API_KEY".to_string(), openrouter_key, Some(720)).await;
}

// Start automatic rotation background task (check every 24 hours)
secret_manager.start_rotation_task(24).await;
```

### Step 4: Add Secret Rotation API Endpoints

Add these routes to the HTTP router (around line 2614):

```rust
.route("/api/secrets/rotation/status", get(get_secret_rotation_status))
.route("/api/secrets/rotation/rotate/:name", post(rotate_secret))
.route("/api/secrets/rotation/rotate-all", post(rotate_all_secrets))
```

### Step 5: Implement API Handlers

Add these handler functions to main.rs:

```rust
async fn get_secret_rotation_status(
    State(secret_manager): State<Arc<SecretRotationManager>>
) -> Json<HashMap<String, SecretMetadata>> {
    Json(secret_manager.get_rotation_status().await)
}

async fn rotate_secret(
    Path(name): Path<String>,
    State(secret_manager): State<Arc<SecretRotationManager>>
) -> Json<RotationResult> {
    match secret_manager.rotate_secret(&name).await {
        Ok(result) => Json(result),
        Err(e) => Json(RotationResult {
            secret_name: name,
            rotated_at: chrono::Utc::now().to_rfc3339(),
            previous_value_hash: "error".to_string(),
            status: "error".to_string(),
            message: e,
        }),
    }
}

async fn rotate_all_secrets(
    State(secret_manager): State<Arc<SecretRotationManager>>
) -> Json<Vec<RotationResult>> {
    Json(secret_manager.check_and_rotate_all().await)
}
```

## Environment Configuration

Add these variables to your `.env` file:

```bash
# Enable automatic secret rotation
SECRET_ROTATION_ENABLED=true

# Rotation interval in hours (default: 720 = 30 days)
SECRET_ROTATION_INTERVAL_HOURS=720

# Check interval for background task (default: 24 hours)
SECRET_ROTATION_CHECK_INTERVAL_HOURS=24
```

## API Endpoints

### GET /api/secrets/rotation/status

Get rotation status for all managed secrets.

**Response:**
```json
{
  "OPENAI_API_KEY": {
    "name": "OPENAI_API_KEY",
    "value": "sk-...",
    "created_at": "2026-07-15T00:00:00Z",
    "last_rotated": "2026-07-15T00:00:00Z",
    "rotation_interval_hours": 720,
    "next_rotation": "2026-08-14T00:00:00Z"
  }
}
```

### POST /api/secrets/rotation/rotate/:name

Manually trigger rotation for a specific secret.

**Response:**
```json
{
  "secret_name": "OPENAI_API_KEY",
  "rotated_at": "2026-07-15T12:00:00Z",
  "previous_value_hash": "a1b2c3...",
  "status": "success",
  "message": "Secret rotated successfully"
}
```

### POST /api/secrets/rotation/rotate-all

Manually trigger rotation for all secrets that need it.

**Response:**
```json
[
  {
    "secret_name": "OPENAI_API_KEY",
    "rotated_at": "2026-07-15T12:00:00Z",
    "previous_value_hash": "a1b2c3...",
    "status": "success",
    "message": "Secret rotated successfully"
  }
]
```

## Production Integration

For production deployment, integrate with your secrets manager:

```rust
use secret_rotation::SecretRotationManager;

// Custom rotation function that calls AWS Secrets Manager
async fn rotate_aws_secret(secret_name: &str) -> Result<String, String> {
    // Call AWS Secrets Manager API to rotate the secret
    // Return the new secret value
    Ok("new_secret_value".to_string())
}

// Override the rotate_secret method to use your secrets manager
```

## Best Practices

1. **30-Day Rotation Interval** - Standard security practice for API keys
2. **Different Intervals** - Use shorter intervals for high-risk secrets
3. **Monitor Rotation** - Log all rotation attempts and failures
4. **Rollback Plan** - Keep previous secret versions for rollback
5. **Test Rotation** - Test rotation in staging before production

## Security Notes

- Secret values are hashed before logging (SHA-256)
- Previous values are not stored in plaintext
- Rotation is disabled by default (enable with SECRET_ROTATION_ENABLED)
- Background task checks at configurable intervals
- Manual rotation available via API

## Dependencies

The secret_rotation module requires these dependencies (already in Cargo.toml):
- `tokio` for async operations
- `chrono` for date/time handling
- `serde` for JSON serialization
- `sha2` for value hashing
- `tracing` for logging

## Testing

Test the secret rotation:

```bash
# Enable rotation
export SECRET_ROTATION_ENABLED=true

# Run the server
cargo run

# Check rotation status
curl http://localhost:3000/api/secrets/rotation/status

# Manually rotate a secret
curl -X POST http://localhost:3000/api/secrets/rotation/rotate/OPENAI_API_KEY

# Rotate all secrets
curl -X POST http://localhost:3000/api/secrets/rotation/rotate-all
```

## Monitoring

Monitor these metrics:
- Rotation success rate
- Rotation failure rate
- Time since last rotation
- Secrets approaching rotation deadline
- Rotation API call volume
