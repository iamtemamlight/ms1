//! AI Module - Rig Rust AI Framework Integration
//! 
//! Provides AI-powered analysis for flash loan arbitrage opportunities.
//! Supports multiple providers: Groq and OpenRouter.

pub mod groq;
pub mod openrouter;
pub mod manager;
pub mod provider_registry;

pub fn build_opportunity_prompt(
    token_in: &str,
    token_out: &str,
    amount: f64,
    chain: &str,
    pool_data: &str,
) -> (String, String) {
    let system_prompt = r#"You are a DeFi trading assistant specialized in flash loan arbitrage analysis.
Your role is strictly limited to:
1. Opportunity analysis - identify potential arbitrage paths
2. Risk analysis - assess market risks, slippage, gas costs
3. Trade summaries - explain trade logic in human-readable terms
4. Logging explanations - provide context for decisions

NEVER:
- Sign transactions
- Hold private keys
- Execute swaps directly
- Control wallets

All transaction execution remains inside the Rust trading engine."#.to_string();

    let user_prompt = format!(
        r#"Analyze this potential flash loan arbitrage opportunity:

Input Token: {}
Output Token: {}
Amount: {} (in wei)
Chain: {}

Pool Data:
{}

Please provide:
1. Is this a viable opportunity? (brief yes/no)
2. Estimated profit after gas (if positive)
3. Risk factors to consider
4. Recommended action
"#,
        token_in, token_out, amount, chain, pool_data
    );

    (system_prompt, user_prompt)
}

/// Risk analysis prompt template - kept for future integration
#[allow(dead_code)]
pub fn build_risk_prompt(
    trade_size: f64,
    source_pool: &str,
    dest_pool: &str,
    chain: &str,
) -> (String, String) {
    let system_prompt = r#"You are a DeFi risk analysis assistant.
Provide risk assessments without executing or recommending transactions."#.to_string();

    let user_prompt = format!(
        r#"Analyze risk for this trade:

Trade Size: {} wei
Source Pool: {}
Destination Pool: {}
Chain: {}

Provide risk level (LOW/MEDIUM/HIGH) and explanation."#,
        trade_size, source_pool, dest_pool, chain
    );

    (system_prompt, user_prompt)
}
