// Environment variable validation for production deployment
use std::collections::HashSet;
use std::env;
use tracing::{info, warn, error};

#[derive(Debug, Clone)]
pub struct EnvVarRequirement {
    pub name: String,
    pub required: bool,
    pub description: String,
    pub validator: Option<fn(&str) -> Result<(), String>>,
}

#[derive(Debug)]
pub struct ValidationResult {
    pub valid: bool,
    pub missing: Vec<String>,
    pub invalid: Vec<(String, String)>,
    pub warnings: Vec<String>,
}

pub struct EnvValidator {
    requirements: Vec<EnvVarRequirement>,
}

impl EnvValidator {
    pub fn new() -> Self {
        Self {
            requirements: Self::default_requirements(),
        }
    }

    pub fn with_custom_requirements(requirements: Vec<EnvVarRequirement>) -> Self {
        Self { requirements }
    }

    fn default_requirements() -> Vec<EnvVarRequirement> {
        vec![
            // Database
            EnvVarRequirement {
                name: "DATABASE_URL".to_string(),
                required: true,
                description: "PostgreSQL connection string".to_string(),
                validator: Some(|v| {
                    if v.starts_with("postgresql://") || v.starts_with("postgres://") {
                        Ok(())
                    } else {
                        Err("Must start with postgresql:// or postgres://".to_string())
                    }
                }),
            },
            EnvVarRequirement {
                name: "REDIS_URL".to_string(),
                required: true,
                description: "Redis connection string".to_string(),
                validator: Some(|v| {
                    if v.starts_with("redis://") {
                        Ok(())
                    } else {
                        Err("Must start with redis://".to_string())
                    }
                }),
            },
            
            // AI/ML API Keys
            EnvVarRequirement {
                name: "OPENAI_API_KEY".to_string(),
                required: true,
                description: "OpenAI API key for AI features".to_string(),
                validator: Some(|v| {
                    if v.starts_with("sk-") && v.len() >= 20 {
                        Ok(())
                    } else {
                        Err("Must be a valid OpenAI API key (starts with sk-)".to_string())
                    }
                }),
            },
            EnvVarRequirement {
                name: "GROQ_API_KEY".to_string(),
                required: false,
                description: "Groq API key for fast inference".to_string(),
                validator: Some(|v| {
                    if v.starts_with("gsk_") && v.len() >= 20 {
                        Ok(())
                    } else {
                        Err("Must be a valid Groq API key (starts with gsk_)".to_string())
                    }
                }),
            },
            
            // Blockchain Configuration
            EnvVarRequirement {
                name: "PRIVATE_KEY".to_string(),
                required: true,
                description: "Wallet private key (use hardware wallet in production)".to_string(),
                validator: Some(|v| {
                    if v.starts_with("0x") && v.len() == 66 {
                        Ok(())
                    } else {
                        Err("Must be a 32-byte hex string with 0x prefix".to_string())
                    }
                }),
            },
            EnvVarRequirement {
                name: "WALLET_ADDRESS".to_string(),
                required: true,
                description: "Wallet address".to_string(),
                validator: Some(|v| {
                    if v.starts_with("0x") && v.len() == 42 {
                        Ok(())
                    } else {
                        Err("Must be a 20-byte hex address with 0x prefix".to_string())
                    }
                }),
            },
            EnvVarRequirement {
                name: "CHAIN_ID".to_string(),
                required: true,
                description: "Blockchain chain ID (1=Ethereum, 8453=Base, 137=Polygon)".to_string(),
                validator: Some(|v| {
                    let chain_id: u64 = v.parse().map_err(|_| "Must be a valid number".to_string())?;
                    match chain_id {
                        1 | 8453 | 137 | 56 | 42161 | 10 | 43114 => Ok(()),
                        _ => Err("Must be a supported chain ID".to_string()),
                    }
                }),
            },
            
            // RPC Endpoints
            EnvVarRequirement {
                name: "RPC_ENDPOINT".to_string(),
                required: true,
                description: "Primary RPC endpoint URL".to_string(),
                validator: Some(|v| {
                    if v.starts_with("http://") || v.starts_with("https://") {
                        Ok(())
                    } else {
                        Err("Must be a valid HTTP/HTTPS URL".to_string())
                    }
                }),
            },
            
            // Engine Configuration
            EnvVarRequirement {
                name: "VITE_ENGINE_MODE".to_string(),
                required: true,
                description: "Engine mode: simulation, shadow-fork, or production".to_string(),
                validator: Some(|v| {
                    match v.to_lowercase().as_str() {
                        "simulation" | "shadow-fork" | "production" => Ok(()),
                        _ => Err("Must be simulation, shadow-fork, or production".to_string()),
                    }
                }),
            },
            EnvVarRequirement {
                name: "PAPER_TRADING_MODE".to_string(),
                required: true,
                description: "Paper trading mode (true/false)".to_string(),
                validator: Some(|v| {
                    match v.to_lowercase().as_str() {
                        "true" | "false" => Ok(()),
                        _ => Err("Must be true or false".to_string()),
                    }
                }),
            },
            EnvVarRequirement {
                name: "VITE_DEMO_MODE".to_string(),
                required: true,
                description: "Demo mode (true/false)".to_string(),
                validator: Some(|v| {
                    match v.to_lowercase().as_str() {
                        "true" | "false" => Ok(()),
                        _ => Err("Must be true or false".to_string()),
                    }
                }),
            },
            
            // Security
            EnvVarRequirement {
                name: "SESSION_SECRET".to_string(),
                required: true,
                description: "Session secret (min 64 chars)".to_string(),
                validator: Some(|v| {
                    if v.len() >= 64 {
                        Ok(())
                    } else {
                        Err("Must be at least 64 characters".to_string())
                    }
                }),
            },
            
            // Server Configuration
            EnvVarRequirement {
                name: "HTTP_BIND_ADDR".to_string(),
                required: false,
                description: "HTTP bind address (default: 0.0.0.0:3000)".to_string(),
                validator: None,
            },
            EnvVarRequirement {
                name: "C2_BIND_ADDR".to_string(),
                required: false,
                description: "gRPC bind address (default: 0.0.0.0:50051)".to_string(),
                validator: None,
            },
        ]
    }

    pub fn validate(&self) -> ValidationResult {
        let mut missing = Vec::new();
        let mut invalid = Vec::new();
        let mut warnings = Vec::new();

        for req in &self.requirements {
            match env::var(&req.name) {
                Ok(value) => {
                    if let Some(validator) = &req.validator {
                        match validator(&value) {
                            Ok(_) => {
                                info!("✓ {} validated successfully", req.name);
                            }
                            Err(e) => {
                                error!("✗ {} validation failed: {}", req.name, e);
                                invalid.push((req.name.clone(), e));
                            }
                        }
                    }
                }
                Err(_) => {
                    if req.required {
                        error!("✗ {} is required but not set", req.name);
                        missing.push(req.name.clone());
                    } else {
                        warn!("⚠ {} is not set (optional)", req.name);
                        warnings.push(format!("{} not set (optional): {}", req.name, req.description));
                    }
                }
            }
        }

        // Security warnings
        if let Ok(private_key) = env::var("PRIVATE_KEY") {
            if !private_key.starts_with("VAULT_MANAGED") && private_key != "0xYourPrivateKeyHere" {
                warnings.push("PRIVATE_KEY is in plaintext - consider using hardware wallet or secrets manager".to_string());
            }
        }

        if let Ok(vite_engine_mode) = env::var("VITE_ENGINE_MODE") {
            if vite_engine_mode == "production" {
                if env::var("VITE_DEMO_MODE").unwrap_or_default() == "true" {
                    warnings.push("VITE_ENGINE_MODE=production but VITE_DEMO_MODE=true - demo mode will block live trading".to_string());
                }
            }
        }

        let valid = missing.is_empty() && invalid.is_empty();

        ValidationResult {
            valid,
            missing,
            invalid,
            warnings,
        }
    }

    pub fn validate_or_panic(&self) {
        let result = self.validate();
        
        if !result.valid {
            error!("Environment validation failed!");
            error!("Missing variables: {:?}", result.missing);
            error!("Invalid variables: {:?}", result.invalid);
            
            if !result.warnings.is_empty() {
                warn!("Warnings: {:?}", result.warnings);
            }
            
            panic!("Environment validation failed. Please set required environment variables.");
        }
        
        info!("Environment validation passed ✓");
        
        if !result.warnings.is_empty() {
            for warning in &result.warnings {
                warn!("⚠ {}", warning);
            }
        }
    }
}

impl Default for EnvValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        env::set_var("DATABASE_URL", "postgresql://user:pass@localhost/db");
        env::set_var("REDIS_URL", "redis://localhost");
        env::set_var("OPENAI_API_KEY", "sk-test12345678901234567890");
        env::set_var("PRIVATE_KEY", "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef");
        env::set_var("WALLET_ADDRESS", "0x1234567890abcdef1234567890abcdef12345678");
        env::set_var("CHAIN_ID", "1");
        env::set_var("RPC_ENDPOINT", "https://eth.llamarpc.com");
        env::set_var("VITE_ENGINE_MODE", "simulation");
        env::set_var("PAPER_TRADING_MODE", "true");
        env::set_var("VITE_DEMO_MODE", "true");
        env::set_var("SESSION_SECRET", "a".repeat(64));

        let validator = EnvValidator::new();
        let result = validator.validate();

        assert!(result.valid);
        assert!(result.missing.is_empty());
        assert!(result.invalid.is_empty());
    }

    #[test]
    fn test_validation_missing_required() {
        env::remove_var("DATABASE_URL");
        
        let validator = EnvValidator::new();
        let result = validator.validate();

        assert!(!result.valid);
        assert!(result.missing.contains(&"DATABASE_URL".to_string()));
    }

    #[test]
    fn test_validation_invalid_format() {
        env::set_var("DATABASE_URL", "invalid-url");
        
        let validator = EnvValidator::new();
        let result = validator.validate();

        assert!(!result.valid);
        assert!(!result.invalid.is_empty());
    }
}
