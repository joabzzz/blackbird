use crate::types::ChatMessage;
use anyhow::Result;
use once_cell::sync::Lazy;
use rig::client::CompletionClient;
use rig::completion::{Chat, Prompt};
use std::collections::HashMap;
use std::env;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use super::providers::ProviderClient;

// ============================================
// Error Types
// ============================================

#[derive(Debug, Clone)]
pub struct ChatError(String);

impl ChatError {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl std::fmt::Display for ChatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ChatError {}

impl From<anyhow::Error> for ChatError {
    fn from(err: anyhow::Error) -> Self {
        ChatError::new(err.to_string())
    }
}

pub type ChatResult<T> = Result<T, ChatError>;

// ============================================
// Streaming State Management
// ============================================

static STREAM_STORE: Lazy<StreamStore> = Lazy::new(StreamStore::default);

struct StreamStore {
    counter: AtomicU64,
    entries: Mutex<HashMap<u64, StreamEntry>>,
}

impl Default for StreamStore {
    fn default() -> Self {
        Self {
            counter: AtomicU64::new(1),
            entries: Mutex::new(HashMap::new()),
        }
    }
}

#[derive(Default)]
struct StreamEntry {
    buffer: String,
    done: bool,
}

impl StreamStore {
    fn create_handle(&self) -> StreamHandle {
        let id = self.counter.fetch_add(1, Ordering::Relaxed);
        let mut entries = self.entries.lock().expect("stream store poisoned");
        entries.insert(id, StreamEntry::default());
        StreamHandle { id }
    }

    fn append(&self, id: u64, chunk: &str) {
        let mut entries = self.entries.lock().expect("stream store poisoned");
        if let Some(entry) = entries.get_mut(&id) {
            entry.buffer.push_str(chunk);
        }
    }

    fn finish(&self, id: u64) {
        let mut entries = self.entries.lock().expect("stream store poisoned");
        if let Some(entry) = entries.get_mut(&id) {
            entry.done = true;
        }
    }

    fn fail(&self, id: u64, message: String) {
        let mut entries = self.entries.lock().expect("stream store poisoned");
        if let Some(entry) = entries.get_mut(&id) {
            entry.buffer = message;
            entry.done = true;
        }
    }

    fn snapshot(&self, id: u64) -> ChatResult<(String, bool)> {
        let entries = self.entries.lock().expect("stream store poisoned");
        if let Some(entry) = entries.get(&id) {
            Ok((entry.buffer.clone(), entry.done))
        } else {
            Err(ChatError::new("invalid stream id"))
        }
    }
}

#[derive(Clone)]
pub struct StreamHandle {
    id: u64,
}

impl StreamHandle {
    pub fn append(&self, piece: &str) {
        STREAM_STORE.append(self.id, piece);
    }

    pub fn finish(&self) {
        STREAM_STORE.finish(self.id);
    }

    pub fn fail(&self, err: &str) {
        STREAM_STORE.fail(self.id, err.to_string());
    }
}

/// Unified AI client wrapper for Blackbird
/// Handles provider auto-detection and agent configuration
pub struct BlackbirdAI {
    client: ProviderClient,
}

impl BlackbirdAI {
    /// Create AI client from environment configuration
    pub fn from_env() -> Result<Self> {
        let client = ProviderClient::from_env()?;
        Ok(Self { client })
    }

    /// Get the system prompt for Blackbird
    fn system_prompt() -> String {
        r#"You are Blackbird, an AI writing partner designed to help users think, write, and organize their ideas.

When responding:
- Be concise and helpful
- Format with markdown when appropriate
- At the end of each response, suggest relevant document tags on a new line in the format: [[doc_tags: tag1, tag2, tag3]]

You have access to tools that allow you to:
- Perform calculations
- Search through the user's saved documents
- List saved documents with filtering
- Get current application settings"#
            .to_string()
    }

    /// Simple prompt (non-streaming, single-turn)
    pub async fn prompt(&self, message: &str) -> Result<String> {
        match &self.client {
            ProviderClient::OpenAI(client) => {
                let model = env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o".to_string());

                let agent = client
                    .agent(&model)
                    .preamble(&Self::system_prompt())
                    .max_tokens(4096)
                    .temperature(0.7)
                    .build();

                Ok(agent.prompt(message).await?)
            }
            ProviderClient::Anthropic(client) => {
                let model = env::var("ANTHROPIC_MODEL")
                    .unwrap_or_else(|_| "claude-3-5-sonnet-20241022".to_string());

                let agent = client
                    .agent(&model)
                    .preamble(&Self::system_prompt())
                    .max_tokens(4096)
                    .temperature(0.7)
                    .build();

                Ok(agent.prompt(message).await?)
            }
            ProviderClient::Ollama(client) => {
                let model = env::var("LLM_MODEL").unwrap_or_else(|_| "llama3.1:latest".to_string());

                let agent = client
                    .agent(&model)
                    .preamble(&Self::system_prompt())
                    .build();

                Ok(agent.prompt(message).await?)
            }
            ProviderClient::Blackbird(client) => {
                // For Blackbird, we send messages directly
                let messages = vec![
                    ChatMessage {
                        role: crate::types::Role::User,
                        content: Self::system_prompt(),
                        created_at: None,
                        tags: vec![],
                    },
                    ChatMessage {
                        role: crate::types::Role::User,
                        content: message.to_string(),
                        created_at: None,
                        tags: vec![],
                    },
                ];

                Ok(client.complete(&messages).await?)
            }
        }
    }

    /// Chat with conversation history (non-streaming, multi-turn)
    pub async fn chat(&self, message: &str, history: Vec<ChatMessage>) -> Result<String> {
        match &self.client {
            ProviderClient::OpenAI(client) => {
                let rig_messages = self.convert_to_rig_messages(history);
                let model = env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o".to_string());

                let agent = client
                    .agent(&model)
                    .preamble(&Self::system_prompt())
                    .max_tokens(4096)
                    .temperature(0.7)
                    .build();

                Ok(agent.chat(message, rig_messages).await?)
            }
            ProviderClient::Anthropic(client) => {
                let rig_messages = self.convert_to_rig_messages(history);
                let model = env::var("ANTHROPIC_MODEL")
                    .unwrap_or_else(|_| "claude-3-5-sonnet-20241022".to_string());

                let agent = client
                    .agent(&model)
                    .preamble(&Self::system_prompt())
                    .max_tokens(4096)
                    .temperature(0.7)
                    .build();

                Ok(agent.chat(message, rig_messages).await?)
            }
            ProviderClient::Ollama(client) => {
                let rig_messages = self.convert_to_rig_messages(history);
                let model = env::var("LLM_MODEL").unwrap_or_else(|_| "llama3.1:latest".to_string());

                let agent = client
                    .agent(&model)
                    .preamble(&Self::system_prompt())
                    .build();

                Ok(agent.chat(message, rig_messages).await?)
            }
            ProviderClient::Blackbird(client) => {
                // For Blackbird, we build the full message array
                let mut messages = vec![ChatMessage {
                    role: crate::types::Role::User,
                    content: Self::system_prompt(),
                    created_at: None,
                    tags: vec![],
                }];

                // Add history
                messages.extend(history);

                // Add new user message
                messages.push(ChatMessage {
                    role: crate::types::Role::User,
                    content: message.to_string(),
                    created_at: None,
                    tags: vec![],
                });

                Ok(client.complete(&messages).await?)
            }
        }
    }

    /// Convert Blackbird ChatMessage to Rig Message format
    fn convert_to_rig_messages(&self, messages: Vec<ChatMessage>) -> Vec<rig::message::Message> {
        messages
            .into_iter()
            .map(|msg| match msg.role {
                crate::types::Role::User => rig::message::Message::user(&msg.content),
                crate::types::Role::Assistant => rig::message::Message::assistant(&msg.content),
            })
            .collect()
    }
}

// ============================================
// Public API Functions
// ============================================

/// Simple chat reply (blocking)
pub async fn chat_reply(messages: Vec<ChatMessage>) -> ChatResult<String> {
    let ai = BlackbirdAI::from_env()
        .map_err(|e| ChatError::new(format!("Failed to initialize AI: {}", e)))?;

    if messages.is_empty() {
        return Err(ChatError::new("No messages provided"));
    }

    let last_message = &messages[messages.len() - 1];
    let history = messages[..messages.len() - 1].to_vec();

    ai.chat(&last_message.content, history)
        .await
        .map_err(|e| ChatError::new(format!("Chat error: {}", e)))
}

/// Start streaming chat response
pub async fn chat_reply_stream_start(messages: Vec<ChatMessage>) -> ChatResult<u64> {
    let handle = STREAM_STORE.create_handle();
    let id = handle.id;

    tokio::spawn(async move {
        match chat_reply(messages).await {
            Ok(response) => {
                handle.append(&response);
                handle.finish();
            }
            Err(err) => {
                handle.fail(&err.to_string());
            }
        }
    });

    Ok(id)
}

/// Poll streaming chat status
pub async fn chat_reply_stream_poll(id: u64) -> ChatResult<(String, bool)> {
    STREAM_STORE.snapshot(id)
}
