# Environment Validation Integration Guide

The `env_validation.rs` module has been created with comprehensive environment variable validation. This guide explains how to integrate it into `main.rs`.

## Manual Integration Steps

### Step 1: Add Module Declaration

Add this line to the module declarations section in `main.rs` (around line 274):

```rust
mod env_validation;
```

### Step 2: Add Import

Add this import after the existing imports (around line 275):

```rust
use env_validation::EnvValidator;
```

### Step 3: Validate Environment on Startup

Add environment validation at the beginning of the `run_server` function, before any other initialization:

```rust
pub async fn run_server(addr: String, http_addr: String) -> Result<(), AppError> {
    // Validate environment variables first
    info!("Validating environment configuration...");
    let validator = EnvValidator::new();
    validator.validate_or_panic();
    
    // Continue with rest of initialization...
}
```

### Step 4: Optional: Custom Validation

For custom validation requirements, create a custom validator:

```rust
let custom_requirements = vec![
    EnvVarRequirement {
        name: "CUSTOM_VAR".to_string(),
        required: true,
        description: "Custom variable".to_string(),
        validator: Some(|v| {
            // Custom validation logic
            if v.len() > 10 {
                Ok(())
            } else {
                Err("Must be longer than 10 characters".to_string())
            }
        }),
    },
];

let validator = EnvValidator::with_custom_requirements(custom_requirements);
validator.validate_or_panic();
```

## Validated Environment Variables

The validator checks the following variables:

### Required Variables
- `DATABASE_URL` - PostgreSQL connection string (must start with postgresql://)
- `REDIS_URL` - Redis connection string (must start with redis://)
- `OPENAI_API_KEY` - OpenAI API key (must start with sk-, min 20 chars)
- `PRIVATE_KEY` - Wallet private key (0x prefix, 66 chars)
- `WALLET_ADDRESS` - Wallet address (0x prefix, 42 chars)
- `CHAIN_ID` - Blockchain chain ID (supported: 1, 8453, 137, 56, 42161, 10, 43114)
- `RPC_ENDPOINT` - Primary RPC endpoint (HTTP/HTTPS URL)
- `VITE_ENGINE_MODE` - Engine mode (simulation, shadow-fork, production)
- `PAPER_TRADING_MODE` - Paper trading mode (true/false)
- `VITE_DEMO_MODE` - Demo mode (true/false)
- `SESSION_SECRET` - Session secret (min 64 chars)

### Optional Variables
- `GROQ_API_KEY` - Groq API key for fast inference
- `HTTP_BIND_ADDR` - HTTP bind address (default: 0.0.0.0:3000)
- `C2_BIND_ADDR` - gRPC bind address (default: 0.0.0.0:50051)

## Validation Rules

### Database URL
- Must start with `postgresql://` or `postgres://`
- Example: `postgresql://user:pass@localhost:5432/dbname`

### Redis URL
- Must start with `redis://`
- Example: `redis://localhost:6379`

### API Keys
- OpenAI: Must start with `sk-`, minimum 20 characters
- Groq: Must start with `gsk_`, minimum 20 characters

### Blockchain Configuration
- Private Key: 0x prefix, exactly 66 characters (32 bytes hex)
- Wallet Address: 0x prefix, exactly 42 characters (20 bytes hex)
- Chain ID: Must be a supported chain (1=Ethereum, 8453=Base, 137=Polygon, etc.)

### Engine Mode
- Must be one of: `simulation`, `shadow-fork`, `production`

### Boolean Values
- Must be exactly `true` or `false` (case-insensitive)

## Security Warnings

The validator automatically generates warnings for:

1. **Plaintext Private Keys**
   - Warns if PRIVATE_KEY is not using VAULT_MANAGED or hardware wallet
   - Recommendation: Use hardware wallet or secrets manager in production

2. **Demo Mode in Production**
   - Warns if VITE_ENGINE_MODE=production but VITE_DEMO_MODE=true
   - Demo mode will block live trading

## Validation Output

### Success Example
```
INFO ✓ DATABASE_URL validated successfully
INFO ✓ REDIS_URL validated successfully
INFO ✓ OPENAI_API_KEY validated successfully
INFO ✓ PRIVATE_KEY validated successfully
INFO ✓ WALLET_ADDRESS validated successfully
INFO ✓ CHAIN_ID validated successfully
INFO ✓ RPC_ENDPOINT validated successfully
INFO ✓ VITE_ENGINE_MODE validated successfully
INFO ✓ PAPER_TRADING_MODE validated successfully
INFO ✓ VITE_DEMO_MODE validated successfully
INFO ✓ SESSION_SECRET validated successfully
INFO Environment validation passed ✓
```

### Failure Example
```
ERROR ✗ DATABASE_URL is required but not set
ERROR ✗ OPENAI_API_KEY is required but not set
ERROR Environment validation failed!
ERROR Missing variables: ["DATABASE_URL", "OPENAI_API_KEY"]
```

## Testing

Test the validation logic:

```bash
# Test with valid environment
export DATABASE_URL="postgresql://user:pass@localhost/db"
export REDIS_URL="redis://localhost"
export OPENAI_API_KEY="sk-test12345678901234567890"
# ... set other required variables
cargo run

# Test with missing variables
unset DATABASE_URL
cargo run  # Should panic with validation error
```

## Integration with Docker

Add environment validation to Dockerfile:

```dockerfile
# Add validation step before starting server
RUN cargo build --release
CMD ["sh", "-c", "./allbright-c2-backend --validate-env && ./allbright-c2-backend"]
```

## Integration with Kubernetes

Add environment validation as init container:

```yaml
spec:
  initContainers:
    - name: env-validation
      image: allbright-c2:latest
      command: ["./allbright-c2-backend", "--validate-env"]
      envFrom:
        - secretRef:
            name: allbright-secrets
        - configMapRef:
            name: allbright-config
```

## Best Practices

1. **Run validation early** - Validate before any resource initialization
2. **Fail fast** - Use `validate_or_panic()` to prevent startup with invalid config
3. **Custom validators** - Add custom validation for application-specific variables
4. **Security warnings** - Pay attention to warnings about plaintext secrets
5. **Environment separation** - Use different validation rules for dev/staging/prod

## Dependencies

The env_validation module requires these dependencies (already in Cargo.toml):
- `std::env` for environment variable access
- `tracing` for logging
- `std::collections` for data structures

## Notes

- Required variables must be set or the application will panic
- Optional variables generate warnings but don't prevent startup
- Custom validators can be added for application-specific validation
- Validation runs once at startup
- Security warnings are logged but don't prevent startup
