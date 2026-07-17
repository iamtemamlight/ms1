# Two-Factor Authentication Integration Guide

The `two_factor_auth.rs` module provides 2FA enforcement for LIVE mode operations. This guide explains how to integrate it into the AllBright system.

## Manual Integration Steps

### Step 1: Add Module Declaration

Add this line to the module declarations section in `main.rs` (around line 274):

```rust
mod two_factor_auth;
```

### Step 2: Add Import

Add this import after the existing imports (around line 275):

```rust
use two_factor_auth::TwoFactorAuth;
```

### Step 3: Initialize 2FA System

In the `run_server` function, initialize the 2FA system after environment validation:

```rust
// Initialize 2FA system
let two_factor_auth = Arc::new(TwoFactorAuth::new());

// Start cleanup task for expired challenges
let auth_cleanup = two_factor_auth.clone();
tokio::spawn(async move {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300)); // Every 5 minutes
    loop {
        interval.tick().await;
        auth_cleanup.cleanup_expired_challenges().await;
    }
});
```

### Step 4: Add 2FA API Endpoints

Add these routes to the HTTP router (around line 2614):

```rust
.route("/api/2fa/enable", post(enable_2fa))
.route("/api/2fa/disable", post(disable_2fa))
.route("/api/2fa/verify-setup", post(verify_totp_setup))
.route("/api/2fa/challenge", post(create_2fa_challenge))
.route("/api/2fa/verify", post(verify_2fa_challenge))
.route("/api/2fa/backup", post(verify_backup_code))
.route("/api/2fa/status", get(get_2fa_status))
```

### Step 5: Implement API Handlers

Add these handler functions to main.rs:

```rust
async fn enable_2fa(
    State(auth): State<Arc<TwoFactorAuth>>,
    Json(req): Json<Enable2faRequest>
) -> Json<serde_json::Value> {
    match auth.enable_2fa(req.user_id, req.totp_secret).await {
        Ok(_) => Json(json!({"success": true, "message": "2FA enabled"})),
        Err(e) => Json(json!({"success": false, "message": e})),
    }
}

async fn disable_2fa(
    State(auth): State<Arc<TwoFactorAuth>>,
    Json(req): Json<Disable2faRequest>
) -> Json<serde_json::Value> {
    match auth.disable_2fa(req.user_id).await {
        Ok(_) => Json(json!({"success": true, "message": "2FA disabled"})),
        Err(e) => Json(json!({"success": false, "message": e})),
    }
}

async fn verify_totp_setup(
    State(auth): State<Arc<TwoFactorAuth>>,
    Json(req): Json<VerifyTotpRequest>
) -> Json<serde_json::Value> {
    match auth.verify_totp_setup(req.user_id, req.code).await {
        Ok(_) => Json(json!({"success": true, "message": "TOTP verified"})),
        Err(e) => Json(json!({"success": false, "message": e})),
    }
}

async fn create_2fa_challenge(
    State(auth): State<Arc<TwoFactorAuth>>,
    Json(req): Json<CreateChallengeRequest>
) -> Json<serde_json::Value> {
    match auth.create_challenge(req.user_id).await {
        Ok(code) => Json(json!({
            "success": true,
            "code": code,
            "message": "Challenge created (send via SMS/email in production)"
        })),
        Err(e) => Json(json!({"success": false, "message": e})),
    }
}

async fn verify_2fa_challenge(
    State(auth): State<Arc<TwoFactorAuth>>,
    Json(req): Json<VerifyChallengeRequest>
) -> Json<TwoFactorVerification> {
    match auth.verify_challenge(req.challenge_id, req.code).await {
        Ok(result) => Json(result),
        Err(e) => Json(TwoFactorVerification {
            success: false,
            message: e,
            challenge_id: req.challenge_id,
        }),
    }
}

async fn verify_backup_code(
    State(auth): State<Arc<TwoFactorAuth>>,
    Json(req): Json<BackupCodeRequest>
) -> Json<TwoFactorVerification> {
    match auth.verify_backup_code(req.user_id, req.code).await {
        Ok(result) => Json(result),
        Err(e) => Json(TwoFactorVerification {
            success: false,
            message: e,
            challenge_id: String::new(),
        }),
    }
}

async fn get_2fa_status(
    State(auth): State<Arc<TwoFactorAuth>>,
    Path(user_id): Path<String>
) -> Json<Option<TwoFactorConfig>> {
    Json(auth.get_status(&user_id).await)
}
```

### Step 6: Add Request/Response Types

Add these types to your request/response module:

```rust
#[derive(Deserialize)]
struct Enable2faRequest {
    user_id: String,
    totp_secret: Option<String>,
}

#[derive(Deserialize)]
struct Disable2faRequest {
    user_id: String,
}

#[derive(Deserialize)]
struct VerifyTotpRequest {
    user_id: String,
    code: String,
}

#[derive(Deserialize)]
struct CreateChallengeRequest {
    user_id: String,
}

#[derive(Deserialize)]
struct VerifyChallengeRequest {
    challenge_id: String,
    code: String,
}

#[derive(Deserialize)]
struct BackupCodeRequest {
    user_id: String,
    code: String,
}
```

### Step 7: Enforce 2FA for LIVE Mode

Modify the mode execution handler to require 2FA for LIVE mode:

```rust
async fn execute_mode(
    State(auth): State<Arc<TwoFactorAuth>>,
    Json(req): Json<ExecuteModeRequest>
) -> Json<serde_json::Value> {
    // Check if 2FA is required for this mode
    if auth.is_2fa_required(&req.user_id, &req.mode).await {
        if !req.two_factor_verified {
            return Json(json!({
                "success": false,
                "message": "2FA verification required for LIVE mode",
                "requires_2fa": true
            }));
        }
        
        // Verify the challenge
        match auth.verify_challenge(req.challenge_id.clone(), req.two_factor_code.clone()).await {
            Ok(verification) if verification.success => {
                // Proceed with mode execution
            },
            Ok(_) | Err(_) => {
                return Json(json!({
                    "success": false,
                    "message": "2FA verification failed"
                }));
            }
        }
    }
    
    // Continue with mode execution...
}
```

## Environment Configuration

Add these variables to your `.env` file:

```bash
# Enable 2FA enforcement
TWO_FACTOR_AUTH_ENABLED=true

# 2FA code length (default: 6)
TWO_FACTOR_CODE_LENGTH=6

# 2FA code expiry in seconds (default: 300 = 5 minutes)
TWO_FACTOR_CODE_EXPIRY_SECONDS=300
```

## API Endpoints

### POST /api/2fa/enable

Enable 2FA for a user.

**Request:**
```json
{
  "user_id": "user123",
  "totp_secret": "JBSWY3DPEHPK3PXP" // Optional, generated if not provided
}
```

**Response:**
```json
{
  "success": true,
  "message": "2FA enabled"
}
```

### POST /api/2fa/disable

Disable 2FA for a user.

**Request:**
```json
{
  "user_id": "user123"
}
```

**Response:**
```json
{
  "success": true,
  "message": "2FA disabled"
}
```

### POST /api/2fa/verify-setup

Verify TOTP setup (first-time verification).

**Request:**
```json
{
  "user_id": "user123",
  "code": "123456"
}
```

**Response:**
```json
{
  "success": true,
  "message": "TOTP verified"
}
```

### POST /api/2fa/challenge

Create a 2FA challenge for LIVE mode execution.

**Request:**
```json
{
  "user_id": "user123"
}
```

**Response:**
```json
{
  "success": true,
  "code": "123456",
  "message": "Challenge created (send via SMS/email in production)"
}
```

### POST /api/2fa/verify

Verify a 2FA challenge.

**Request:**
```json
{
  "challenge_id": "uuid-here",
  "code": "123456"
}
```

**Response:**
```json
{
  "success": true,
  "message": "2FA verification successful",
  "challenge_id": "uuid-here"
}
```

### POST /api/2fa/backup

Verify using a backup code.

**Request:**
```json
{
  "user_id": "user123",
  "code": "12345678"
}
```

**Response:**
```json
{
  "success": true,
  "message": "Backup code verified",
  "challenge_id": "uuid-here"
}
```

### GET /api/2fa/status/:user_id

Get 2FA status for a user.

**Response:**
```json
{
  "enabled": true,
  "totp_secret": "JBSWY3DPEHPK3PXP",
  "backup_codes": ["12345678", "87654321"],
  "verified": true
}
```

## Production Integration

For production deployment, integrate with SMS/email services:

```rust
use two_factor_auth::TwoFactorAuth;

// Send code via SMS (Twilio, AWS SNS, etc.)
async fn send_2fa_code_sms(user_phone: &str, code: &str) -> Result<(), String> {
    // Call SMS API to send the code
    Ok(())
}

// Send code via email (SendGrid, AWS SES, etc.)
async fn send_2fa_code_email(user_email: &str, code: &str) -> Result<(), String> {
    // Call email API to send the code
    Ok(())
}
```

## TOTP Integration

For authenticator app integration (Google Authenticator, Authy):

```rust
use totp_lite::{Sha1, TOTP};

// Generate TOTP code
fn generate_totp(secret: &str) -> String {
    let totp = TOTP::<Sha1>::new(secret.as_bytes()).unwrap();
    totp.generate(std::time::SystemTime::now())
}

// Verify TOTP code
fn verify_totp(secret: &str, code: &str) -> bool {
    let totp = TOTP::<Sha1>::new(secret.as_bytes()).unwrap();
    totp.check(code, std::time::SystemTime::now())
}
```

## Best Practices

1. **Require 2FA for LIVE mode** - All live trading operations must have 2FA
2. **Generate backup codes** - Provide 10 backup codes for recovery
3. **Send codes securely** - Use SMS/email in production, never return in API
4. **Short expiry** - 5-minute code expiry for security
5. **Rate limiting** - Limit challenge creation attempts
6. **Audit logging** - Log all 2FA attempts (success/failure)

## Security Notes

- Codes are 6 digits by default
- Codes expire after 5 minutes
- Challenges can only be used once
- Backup codes are single-use
- TOTP secrets are never exposed in API responses
- Expired challenges are automatically cleaned up

## Dependencies

The two_factor_auth module requires these dependencies (add to Cargo.toml):

```toml
[dependencies]
uuid = { version = "1.0", features = ["v4", "serde"] }
rand = "0.8"
base64 = "0.21"
chrono = "0.4"
```

## Testing

Test the 2FA system:

```bash
# Enable 2FA
curl -X POST http://localhost:3000/api/2fa/enable \
  -H "Content-Type: application/json" \
  -d '{"user_id": "user123"}'

# Verify TOTP setup
curl -X POST http://localhost:3000/api/2fa/verify-setup \
  -H "Content-Type: application/json" \
  -d '{"user_id": "user123", "code": "123456"}'

# Create challenge
curl -X POST http://localhost:3000/api/2fa/challenge \
  -H "Content-Type: application/json" \
  -d '{"user_id": "user123"}'

# Verify challenge
curl -X POST http://localhost:3000/api/2fa/verify \
  -H "Content-Type: application/json" \
  -d '{"challenge_id": "uuid-here", "code": "123456"}'

# Check status
curl http://localhost:3000/api/2fa/status/user123
```

## Monitoring

Monitor these metrics:
- 2FA enablement rate
- Challenge creation rate
- Verification success rate
- Verification failure rate
- Backup code usage
- Challenge expiry rate
