 //! Groq AI Provider Implementation
//! 
//! Provides Groq API integration for flash loan analysis using Rig framework.

use anyhow::Result;
use rig::client::CompletionClient;
use rig::completion::Prompt;
use rig::providers::groq;
use std::env;

/// Groq model constants
const MODEL_PRIMARY: &str = "llama-3.3-70b-versatile";

/// Send a prompt to Groq and get the response text.
pub async fn groq_prompt(
    system_prompt: &str,
    user_prompt: &str,
) -> Result<String> {
    let api_key = env::var("GROQ_API_KEY")
        .map_err(|_| anyhow::anyhow!(
            "GROQ_API_KEY not found in environment. Please set it in .env file."
        ))?;

    let groq_client = groq::Client::new(&api_key)
        .map_err(|e| anyhow::anyhow!("Failed to create Groq client: {}", e))?;

    let agent = groq_client
        .agent(MODEL_PRIMARY)
        .preamble(system_prompt)
        .build();

    let response = agent
        .prompt(user_prompt)
        .await
        .map_err(|e| anyhow::anyhow!("Groq prompt failed: {}", e))?;

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_groq_prompt_missing_key() {
        // Ensure GROQ_API_KEY is not set for this test
        std::env::remove_var("GROQ_API_KEY");
        let system = "You are a helpful assistant.";
        let user = "What is 2 + 2?";
        
        let result = groq_prompt(system, user).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_groq_prompt_empty_prompts() {
        std::env::remove_var("GROQ_API_KEY");
        let result = groq_prompt("", "").await;
        assert!(result.is_err());
    }
}
