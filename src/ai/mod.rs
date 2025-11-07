/// AI module for Blackbird
///
/// This module provides a unified interface for LLM interactions using the Rig framework.
/// It supports multiple providers (Blackbird, OpenAI, Anthropic, Ollama) with automatic
/// detection based on environment variables.
///
/// # Architecture
///
/// - `client` - Main BlackbirdAI client with streaming support
/// - `providers` - Provider-specific implementations (Blackbird custom, Rig-based)
///
/// # Usage
///
/// ```rust,no_run
/// use blackbird::ai::BlackbirdAI;
///
/// # async fn example() -> anyhow::Result<()> {
/// let ai = BlackbirdAI::from_env()?;
/// let response = ai.prompt("Hello!").await?;
/// # Ok(())
/// # }
/// ```
mod client;
mod providers;

// Re-export main types
pub use client::{
    BlackbirdAI, ChatError, ChatResult, StreamHandle, chat_reply, chat_reply_stream_poll,
    chat_reply_stream_start,
};
