# IP Whitelisting Integration Guide

The `ip_whitelist.rs` module provides IP-based access control for production security. This guide explains how to integrate it into the AllBright system.

## Manual Integration Steps

### Step 1: Add Module Declaration

Add this line to the module declarations section in `main.rs` (around line 274):

```rust
mod ip_whitelist;
```

### Step 2: Add Import

Add this import after the existing imports (around line 275):

```rust
use ip_whitelist::IpWhitelist;
```

### Step 3: Initialize IP Whitelist

In the `run_server` function, initialize the IP whitelist after environment validation:

```rust
// Initialize IP whitelist
let ip_whitelist = Arc::new(IpWhitelist::new());
```

### Step 4: Add IP Whitelist Middleware

Create an Axum middleware to check IP addresses:

```rust
async fn ip_whitelist_middleware(
    State(ip_whitelist): State<Arc<IpWhitelist>>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract client IP from request
    let client_ip = req
        .headers()
        .get("X-Forwarded-For")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .unwrap_or("unknown")
        .trim();

    // Check if IP is allowed
    let check_result = ip_whitelist.is_allowed(client_ip).await;
    
    if !check_result.allowed {
        warn!("IP blocked: {} - {}", client_ip, check_result.reason);
        return Err(StatusCode::FORBIDDEN);
    }
    
    info!("IP allowed: {}", client_ip);
    Ok(next.run(req).await)
}
```

### Step 5: Apply Middleware to Router

Apply the middleware to the HTTP router (around line 2616):

```rust
let http_router = Router::new()
    .route("/api/ai/ask", post(handle_ai_ask))
    // ... other routes ...
    .layer(cors)
    .layer(axum::middleware::from_fn_with_state(
        ip_whitelist.clone(),
        ip_whitelist_middleware
    ));
```

### Step 6: Add IP Whitelist API Endpoints

Add these routes to the HTTP router (around line 2614):

```rust
.route("/api/ip-whitelist/config", get(get_ip_whitelist_config))
.route("/api/ip-whitelist/add-ip", post(add_ip_to_whitelist))
.route("/api/ip-whitelist/remove-ip", post(remove_ip_from_whitelist))
.route("/api/ip-whitelist/add-cidr", post(add_cidr_to_whitelist))
.route("/api/ip-whitelist/remove-cidr", post(remove_cidr_from_whitelist))
.route("/api/ip-whitelist/enable", post(enable_ip_whitelist))
.route("/api/ip-whitelist/disable", post(disable_ip_whitelist))
.route("/api/ip-whitelist/reload", post(reload_ip_whitelist))
.route("/api/ip-whitelist/check/:ip", get(check_ip_allowed))
```

### Step 7: Implement API Handlers

Add these handler functions to main.rs:

```rust
async fn get_ip_whitelist_config(
    State(ip_whitelist): State<Arc<IpWhitelist>>
) -> Json<IpWhitelistConfig> {
    Json(ip_whitelist.get_config().await)
}

async fn add_ip_to_whitelist(
    State(ip_whitelist): State<Arc<IpWhitelist>>,
    Json(req): Json<AddIpRequest>
) -> Json<serde_json::Value> {
    ip_whitelist.add_ip(req.ip).await;
    Json(json!({"success": true, "message": "IP added to whitelist"}))
}

async fn remove_ip_from_whitelist(
    State(ip_whitelist): State<Arc<IpWhitelist>>,
    Json(req): Json<RemoveIpRequest>
) -> Json<serde_json::Value> {
    ip_whitelist.remove_ip(&req.ip).await;
    Json(json!({"success": true, "message": "IP removed from whitelist"}))
}

async fn add_cidr_to_whitelist(
    State(ip_whitelist): State<Arc<IpWhitelist>>,
    Json(req): Json<AddCidrRequest>
) -> Json<serde_json::Value> {
    ip_whitelist.add_cidr(req.cidr).await;
    Json(json!({"success": true, "message": "CIDR added to whitelist"}))
}

async fn remove_cidr_from_whitelist(
    State(ip_whitelist): State<Arc<IpWhitelist>>,
    Json(req): Json<RemoveCidrRequest>
) -> Json<serde_json::Value> {
    ip_whitelist.remove_cidr(&req.cidr).await;
    Json(json!({"success": true, "message": "CIDR removed from whitelist"}))
}

async fn enable_ip_whitelist(
    State(ip_whitelist): State<Arc<IpWhitelist>>
) -> Json<serde_json::Value> {
    ip_whitelist.set_enabled(true).await;
    Json(json!({"success": true, "message": "IP whitelist enabled"}))
}

async fn disable_ip_whitelist(
    State(ip_whitelist): State<Arc<IpWhitelist>>
) -> Json<serde_json::Value> {
    ip_whitelist.set_enabled(false).await;
    Json(json!({"success": true, "message": "IP whitelist disabled"}))
}

async fn reload_ip_whitelist(
    State(ip_whitelist): State<Arc<IpWhitelist>>
) -> Json<serde_json::Value> {
    ip_whitelist.reload_config().await;
    Json(json!({"success": true, "message": "IP whitelist configuration reloaded"}))
}

async fn check_ip_allowed(
    State(ip_whitelist): State<Arc<IpWhitelist>>,
    Path(ip): Path<String>
) -> Json<IpCheckResult> {
    Json(ip_whitelist.is_allowed(&ip).await)
}
```

### Step 8: Add Request Types

Add these types to your request/response module:

```rust
#[derive(Deserialize)]
struct AddIpRequest {
    ip: String,
}

#[derive(Deserialize)]
struct RemoveIpRequest {
    ip: String,
}

#[derive(Deserialize)]
struct AddCidrRequest {
    cidr: String,
}

#[derive(Deserialize)]
struct RemoveCidrRequest {
    cidr: String,
}
```

## Environment Configuration

Add these variables to your `.env` file:

```bash
# Enable IP whitelisting
IP_WHITELIST_ENABLED=true

# Allowed IP addresses (comma-separated)
ALLOWED_IPS=192.168.1.100,10.0.0.50,203.0.113.45

# Allowed CIDR ranges (comma-separated)
ALLOWED_CIDRS=192.168.1.0/24,10.0.0.0/8,203.0.113.0/24
```

## API Endpoints

### GET /api/ip-whitelist/config

Get current IP whitelist configuration.

**Response:**
```json
{
  "enabled": true,
  "allowed_ips": ["192.168.1.100", "10.0.0.50"],
  "allowed_cidrs": ["192.168.1.0/24", "10.0.0.0/8"],
  "default_action": "Deny"
}
```

### POST /api/ip-whitelist/add-ip

Add an IP to the whitelist.

**Request:**
```json
{
  "ip": "192.168.1.100"
}
```

**Response:**
```json
{
  "success": true,
  "message": "IP added to whitelist"
}
```

### POST /api/ip-whitelist/remove-ip

Remove an IP from the whitelist.

**Request:**
```json
{
  "ip": "192.168.1.100"
}
```

**Response:**
```json
{
  "success": true,
  "message": "IP removed from whitelist"
}
```

### POST /api/ip-whitelist/add-cidr

Add a CIDR range to the whitelist.

**Request:**
```json
{
  "cidr": "192.168.1.0/24"
}
```

**Response:**
```json
{
  "success": true,
  "message": "CIDR added to whitelist"
}
```

### POST /api/ip-whitelist/remove-cidr

Remove a CIDR range from the whitelist.

**Request:**
```json
{
  "cidr": "192.168.1.0/24"
}
```

**Response:**
```json
{
  "success": true,
  "message": "CIDR removed from whitelist"
}
```

### POST /api/ip-whitelist/enable

Enable the IP whitelist.

**Response:**
```json
{
  "success": true,
  "message": "IP whitelist enabled"
}
```

### POST /api/ip-whitelist/disable

Disable the IP whitelist.

**Response:**
```json
{
  "success": true,
  "message": "IP whitelist disabled"
}
```

### POST /api/ip-whitelist/reload

Reload configuration from environment variables.

**Response:**
```json
{
  "success": true,
  "message": "IP whitelist configuration reloaded"
}
```

### GET /api/ip-whitelist/check/:ip

Check if an IP is allowed.

**Response:**
```json
{
  "allowed": true,
  "ip": "192.168.1.100",
  "reason": "IP is in whitelist"
}
```

## Production Integration

For production deployment behind a reverse proxy:

```nginx
# Nginx configuration to pass client IP
location / {
    proxy_pass http://backend:3000;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    proxy_set_header X-Real-IP $remote_addr;
}
```

For Kubernetes with ingress:

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: allbright-ingress
  annotations:
    nginx.ingress.kubernetes.io/whitelist-source-range: "10.0.0.0/8,192.168.1.0/24"
spec:
  rules:
    - host: api.allbright.io
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: allbright-backend
                port:
                  number: 3000
```

## Best Practices

1. **Enable in production** - Always enable IP whitelisting in production
2. **Use CIDR ranges** - Prefer CIDR ranges over individual IPs for flexibility
3. **Include office IPs** - Whitelist office network ranges
4. **Include VPN IPs** - Whitelist VPN exit points
5. **Monitor blocked IPs** - Log and monitor blocked IP attempts
6. **Regular reviews** - Review and update whitelist regularly
7. **Emergency access** - Have a process for emergency IP additions

## Security Notes

- Localhost (127.0.0.1, ::1) is always allowed
- IPv4 and IPv6 are both supported
- CIDR notation is supported (e.g., 192.168.1.0/24)
- X-Forwarded-For header is used for proxy environments
- Configuration can be reloaded without restart
- All IP checks are logged

## Dependencies

The ip_whitelist module requires these dependencies (already in Cargo.toml):
- `std::net` for IP address parsing
- `tokio` for async operations
- `serde` for JSON serialization
- `tracing` for logging

## Testing

Test the IP whitelist:

```bash
# Enable whitelist
export IP_WHITELIST_ENABLED=true
export ALLOWED_IPS=192.168.1.100

# Check configuration
curl http://localhost:3000/api/ip-whitelist/config

# Add IP
curl -X POST http://localhost:3000/api/ip-whitelist/add-ip \
  -H "Content-Type: application/json" \
  -d '{"ip": "10.0.0.50"}'

# Add CIDR
curl -X POST http://localhost:3000/api/ip-whitelist/add-cidr \
  -H "Content-Type: application/json" \
  -d '{"cidr": "10.0.0.0/8"}'

# Check IP
curl http://localhost:3000/api/ip-whitelist/check/192.168.1.100

# Disable whitelist
curl -X POST http://localhost:3000/api/ip-whitelist/disable
```

## Monitoring

Monitor these metrics:
- IP whitelist enable/disable events
- IP add/remove operations
- Blocked IP attempts
- Allowed IP requests
- CIDR range modifications
- Configuration reloads
