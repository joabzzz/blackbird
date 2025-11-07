pub mod blackbird;

use anyhow::Result;
use rig::providers;
use std::env;

pub use blackbird::BlackbirdClient;

/// Enum to hold different provider clients
pub enum ProviderClient {
    OpenAI(providers::openai::Client),
    Anthropic(providers::anthropic::Client),
    Ollama(providers::ollama::Client),
    Blackbird(BlackbirdClient),
}

impl ProviderClient {
    /// Auto-detect and configure provider from environment variables
    pub fn from_env() -> Result<Self> {
        // Priority order:
        // 1. BLACKBIRD_ENDPOINT → Blackbird API
        // 2. OPENAI_API_KEY → OpenAI
        // 3. ANTHROPIC_API_KEY → Claude
        // 4. LLM_USE_OLLAMA=true → Ollama

        // Check for Blackbird endpoint first
        if let Ok(endpoint) = env::var("BLACKBIRD_ENDPOINT") {
            let tier = env::var("BLACKBIRD_TIER").unwrap_or_else(|_| "ultra".to_string());
            let model = env::var("BLACKBIRD_MODEL").unwrap_or_else(|_| "gpt-oss-120b".to_string());
            let api_key = env::var("BLACKBIRD_API_KEY").ok();

            return Ok(Self::Blackbird(BlackbirdClient::new(
                endpoint, tier, model, api_key,
            )));
        }

        if let Ok(key) = env::var("OPENAI_API_KEY") {
            return Ok(Self::OpenAI(providers::openai::Client::new(&key)));
        }

        if let Ok(key) = env::var("ANTHROPIC_API_KEY") {
            return Ok(Self::Anthropic(providers::anthropic::Client::new(&key)));
        }

        let use_ollama = env::var("LLM_USE_OLLAMA")
            .unwrap_or_else(|_| "false".into())
            .to_ascii_lowercase();

        if matches!(use_ollama.as_str(), "1" | "true" | "yes" | "on") {
            // Ollama endpoint is configured via OLLAMA_HOST environment variable
            // The Rig client reads this automatically (defaults to http://localhost:11434)
            return Ok(Self::Ollama(providers::ollama::Client::new()));
        }

        Err(anyhow::anyhow!(
            "No AI provider configured. Set BLACKBIRD_ENDPOINT, OPENAI_API_KEY, ANTHROPIC_API_KEY, or LLM_USE_OLLAMA=true"
        ))
    }
}
