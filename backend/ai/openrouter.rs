//! OpenRouter AI Provider Implementation
//! 
//! Provides OpenRouter API integration using Rig's OpenAI-compatible interface.

use anyhow::Result;
use rig::client::CompletionClient;
use rig::completion::Prompt;
use rig::providers::openai;
use std::env;

/// OpenRouter base URL constant
const OPENROUTER_BASE_URL: &str = "https://openrouter.ai/api/v1";

/// OpenRouter model constants
const MODEL_PRIMARY: &str = "deepseek/deepseek-chat-v3";

/// Send a prompt to OpenRouter and get the response text.
pub async fn openrouter_prompt(
    system_prompt: &str,
    user_prompt: &str,
) -> Result<String> {
    let api_key = env::var("OPENROUTER_API_KEY")
        .map_err(|_| anyhow::anyhow!(
            "OPENROUTER_API_KEY not found in environment. Please set it in .env file."
        ))?;

    let client = openai::Client::builder()
        .api_key(&api_key)
        .base_url(OPENROUTER_BASE_URL)
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to create OpenRouter client: {}", e))?;

    let agent = client
        .agent(MODEL_PRIMARY)
        .preamble(system_prompt)
        .build();

    let response = agent
        .prompt(user_prompt)
        .await
        .map_err(|e| anyhow::anyhow!("OpenRouter prompt failed: {}", e))?;

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_openrouter_prompt_missing_key() {
        std::env::remove_var("OPENROUTER_API_KEY");
        let system = "You are a helpful assistant.";
        let user = "What is 2 + 2?";
        
        let result = openrouter_prompt(system, user).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_openrouter_prompt_empty_prompts() {
        std::env::remove_var("OPENROUTER_API_KEY");
        let result = openrouter_prompt("", "").await;
        assert!(result.is_err());
    }
}
