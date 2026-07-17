//! AI Manager - Provider Dispatcher
//! 
//! Manages multiple AI providers and dispatches requests to the appropriate one.
//! Supports both built-in providers (.env) and custom providers (registered via API).

use crate::error::AppError;
use crate::ai::groq;
use crate::ai::openrouter;
use crate::ai::provider_registry::CUSTOM_PROVIDERS;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

/// Available AI providers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiProvider {
    /// Groq provider (fast inference)
    Groq,
    /// OpenRouter provider (OpenAI-compatible)
    OpenRouter,
}

impl std::fmt::Display for AiProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AiProvider::Groq => write!(f, "Groq"),
            AiProvider::OpenRouter => write!(f, "OpenRouter"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AiAskRequest {
    pub provider: String,
    pub system_prompt: String,
    pub user_prompt: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AiAskResponse {
    pub response: String,
    pub provider_used: String,
}

/// Dispatch an AI prompt to the specified provider.
///
/// # Arguments
/// * `provider` - Which AI provider to use
/// * `system_prompt` - System instructions
/// * `user_prompt` - User query
///
/// # Returns
/// * `Ok(String)` - Provider's response
/// * `Err(anyhow::Error)` - Any error during dispatch
pub async fn ask_ai(
    provider: &str,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<String> {
    // First check custom providers
    if let Some(custom) = CUSTOM_PROVIDERS.get(provider) {
        info!("Dispatching to custom provider: {}", provider);
        return custom.call(system_prompt, user_prompt).await;
    }

    // Fall back to built-in providers
    let builtin = match provider.to_lowercase().as_str() {
        "groq" => AiProvider::Groq,
        "openrouter" => AiProvider::OpenRouter,
        _ => return Err(anyhow::anyhow!("Unknown provider: {}", provider)),
    };

    ask_builtin(builtin, system_prompt, user_prompt).await
}

/// Dispatch to a built-in provider
async fn ask_builtin(
    provider: AiProvider,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<String> {
    match provider {
        AiProvider::Groq => {
            info!("Dispatching to Groq provider");
            groq::groq_prompt(system_prompt, user_prompt).await
        }
        AiProvider::OpenRouter => {
            info!("Dispatching to OpenRouter provider");
            openrouter::openrouter_prompt(system_prompt, user_prompt).await
        }
    }
}

/// Get the default provider based on environment availability.
///
/// Checks for API keys and returns the first available provider.
/// If neither is available, returns an error.
#[allow(dead_code)]
pub fn get_default_provider() -> Result<AiProvider> {
    // Check Groq first
    if std::env::var("GROQ_API_KEY").is_ok() {
        info!("Using Groq as default provider");
        return Ok(AiProvider::Groq);
    }
    
    // Check OpenRouter second
    if std::env::var("OPENROUTER_API_KEY").is_ok() {
        info!("Using OpenRouter as default provider");
        return Ok(AiProvider::OpenRouter);
    }
    
    // Neither available
    warn!("No AI API key found in environment");
    Err(anyhow::anyhow!(
        "No AI provider API key found. Set either GROQ_API_KEY or OPENROUTER_API_KEY in .env"
    ))
}

/// Auto-select best available provider with fallback chain.
///
/// Tries providers in order and returns first successful response.
pub async fn ask_ai_auto(
    system_prompt: &str,
    user_prompt: &str,
) -> Result<(String, AiProvider)> {
    // Try Groq first
    if std::env::var("GROQ_API_KEY").is_ok() {
        match groq::groq_prompt(system_prompt, user_prompt).await {
            Ok(response) => return Ok((response, AiProvider::Groq)),
            Err(e) => warn!("Groq failed: {}", e),
        }
    }
    
    // Try OpenRouter second
    if std::env::var("OPENROUTER_API_KEY").is_ok() {
        match openrouter::openrouter_prompt(system_prompt, user_prompt).await {
            Ok(response) => return Ok((response, AiProvider::OpenRouter)),
            Err(e) => warn!("OpenRouter failed: {}", e),
        }
    }
    
    // Neither worked
    Err(anyhow::anyhow!(
        "All AI providers failed. Check API keys and try again."
    ))
}

/// Handle incoming AI ask request, dispatching to appropriate provider.
pub async fn ask_ai_endpoint(req: AiAskRequest) -> Result<AiAskResponse, AppError> {
    let response = ask_ai(&req.provider, &req.system_prompt, &req.user_prompt)
        .await
        .map_err(|e| AppError::AiProvider(e.to_string()))?;
    
    Ok(AiAskResponse {
        response,
        provider_used: req.provider.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_display() {
        assert_eq!(AiProvider::Groq.to_string(), "Groq");
        assert_eq!(AiProvider::OpenRouter.to_string(), "OpenRouter");
    }

    #[tokio::test]
    async fn test_ask_ai_unknown_provider() {
        let req = AiAskRequest {
            provider: "unknown".to_string(),
            system_prompt: "test".to_string(),
            user_prompt: "test".to_string(),
        };
        let result = ask_ai_endpoint(req).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_ask_ai_missing_keys() {
        std::env::remove_var("GROQ_API_KEY");
        std::env::remove_var("OPENROUTER_API_KEY");
        let req = AiAskRequest {
            provider: "groq".to_string(),
            system_prompt: "test".to_string(),
            user_prompt: "test".to_string(),
        };
        let result = ask_ai_endpoint(req).await;
        assert!(result.is_err());
    }
}
