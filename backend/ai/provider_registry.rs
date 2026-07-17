//! AI Provider Registry
//! 
//! Manages custom AI providers that can be registered at runtime via the API.
//! Custom providers take precedence over built-in .env providers.

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tracing::{info};
use rig::client::CompletionClient;
use rig::completion::Prompt;
use rig::providers::openai;
use std::sync::LazyLock;

/// A custom AI provider registered at runtime
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomProvider {
    pub name: String,
    pub api_key: String,
    pub base_url: String,
    pub model_id: String,
}

impl CustomProvider {
    /// Call the custom provider's API
    pub async fn call(&self, system_prompt: &str, user_prompt: &str) -> anyhow::Result<String> {
        let client = openai::Client::builder()
            .api_key(&self.api_key)
            .base_url(&self.base_url)
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to create client for {}: {}", self.name, e))?;

        let agent = client
            .agent(&self.model_id)
            .preamble(system_prompt)
            .build();

        let response = agent
            .prompt(user_prompt)
            .await
            .map_err(|e| anyhow::anyhow!("{} prompt failed: {}", self.name, e))?;

        Ok(response)
    }
}

/// Global registry of custom providers
/// Key: provider name (lowercase)
/// Value: CustomProvider config
pub static CUSTOM_PROVIDERS: LazyLock<DashMap<String, CustomProvider>> = LazyLock::new(|| DashMap::new());

/// Register a custom provider
pub fn register_provider(provider: CustomProvider) {
    let name = provider.name.to_lowercase();
    info!("Registering custom AI provider: {}", name);
    CUSTOM_PROVIDERS.insert(name.clone(), provider);
}

/// Remove a custom provider
pub fn unregister_provider(name: &str) -> bool {
    let name = name.to_lowercase();
    CUSTOM_PROVIDERS.remove(&name).is_some()
}

/// List all registered custom providers
pub fn list_providers() -> Vec<CustomProvider> {
    CUSTOM_PROVIDERS.iter().map(|entry| entry.clone()).collect()
}

/// Check if a provider exists (custom or built-in)
pub fn provider_exists(name: &str) -> bool {
    let name = name.to_lowercase();
    if CUSTOM_PROVIDERS.contains_key(&name) {
        return true;
    }
    matches!(name.as_str(), "groq" | "openrouter")
}
