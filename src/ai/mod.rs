use crate::types::ChatMessage;
use async_trait::async_trait;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::env;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

mod blackbird;
mod custom;
mod ollama;

pub use blackbird::parse_blackbird_sse_data;
pub use ollama::parse_ollama_stream_line;

#[derive(Debug, Clone)]
pub struct ChatError(String);

impl ChatError {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for ChatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ChatError {}

impl From<reqwest::Error> for ChatError {
    fn from(err: reqwest::Error) -> Self {
        ChatError::new(err.to_string())
    }
}

impl From<serde_json::Error> for ChatError {
    fn from(err: serde_json::Error) -> Self {
        ChatError::new(err.to_string())
    }
}

pub type ChatResult<T> = Result<T, ChatError>;

#[async_trait]
pub trait LLMBackend: Send + Sync {
    async fn complete(&self, messages: &[ChatMessage]) -> ChatResult<String>;

    fn supports_streaming(&self) -> bool {
        false
    }

    async fn stream(&self, messages: &[ChatMessage], handle: StreamHandle) -> ChatResult<()> {
        let response = self.complete(messages).await?;
        handle.replace(response);
        Ok(())
    }
}

pub async fn chat_reply(messages: Vec<ChatMessage>) -> ChatResult<String> {
    let backend = select_backend()?;
    backend.complete(&messages).await
}

pub async fn chat_reply_stream_start(messages: Vec<ChatMessage>) -> ChatResult<u64> {
    let backend = select_backend()?;
    let handle = STREAM_STORE.create_handle();
    let id = handle.id();
    let task_backend = backend.clone();
    let task_messages = Arc::new(messages);
    let task_handle = handle.clone();

    tokio::spawn(async move {
        let result = task_backend
            .stream(task_messages.as_ref(), task_handle.clone())
            .await;
        match result {
            Ok(_) => {
                task_handle.finish();
            }
            Err(err) => {
                eprintln!("llm stream error: {}", err);
                task_handle.fail(err);
            }
        }
    });

    Ok(id)
}

pub async fn chat_reply_stream_poll(id: u64) -> ChatResult<(String, bool)> {
    STREAM_STORE.snapshot(id)
}

fn select_backend() -> ChatResult<Arc<dyn LLMBackend>> {
    if let Ok(endpoint) = env::var("LLM_ENDPOINT") {
        return Ok(Arc::new(custom::CustomBackend::new(endpoint)));
    }

    let use_ollama = env::var("LLM_USE_OLLAMA")
        .unwrap_or_else(|_| "false".into())
        .to_ascii_lowercase();
    if matches!(use_ollama.as_str(), "1" | "true" | "yes" | "on") {
        return Ok(Arc::new(ollama::OllamaBackend::from_env()));
    }

    if let Ok(endpoint) = env::var("BLACKBIRD_ENDPOINT") {
        return Ok(Arc::new(blackbird::BlackbirdBackend::from_env(endpoint)));
    }

    Err(ChatError::new(
        "No LLM configured. Set LLM_ENDPOINT for a custom backend, BLACKBIRD_ENDPOINT for Blackbird, or LLM_USE_OLLAMA=true for local Ollama.",
    ))
}

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

static STREAM_STORE: Lazy<StreamStore> = Lazy::new(StreamStore::default);

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

    fn replace(&self, id: u64, content: String) {
        let mut entries = self.entries.lock().expect("stream store poisoned");
        if let Some(entry) = entries.get_mut(&id) {
            entry.buffer = content;
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
    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn append(&self, piece: &str) {
        STREAM_STORE.append(self.id, piece);
    }

    pub fn replace(&self, content: String) {
        STREAM_STORE.replace(self.id, content);
    }

    pub fn finish(&self) {
        STREAM_STORE.finish(self.id);
    }

    pub fn fail(&self, err: ChatError) {
        STREAM_STORE.fail(self.id, format!("error: {}", err));
    }
}
