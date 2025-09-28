use crate::types::ChatMessage;
use futures_util::StreamExt;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt,
    sync::{Mutex, atomic::AtomicU64},
};

#[derive(Debug, Clone)]
pub struct ChatError(String);

impl ChatError {
    fn new(msg: impl Into<String>) -> Self {
        Self(msg.into())
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

type ChatResult<T> = Result<T, ChatError>;

// ---------------
// Non-streaming endpoint (custom URL, local Ollama, or Blackbird via env)
// ---------------

pub async fn chat_reply(messages: Vec<ChatMessage>) -> ChatResult<String> {
    use std::env;

    // Try runtime env var first; if not set, choose other backends
    let endpoint = env::var("LLM_ENDPOINT").ok();
    let bb_endpoint = env::var("BLACKBIRD_ENDPOINT").ok();
    let use_ollama = matches!(
        env::var("LLM_USE_OLLAMA")
            .unwrap_or_else(|_| "false".into())
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "yes" | "on"
    );

    if let Some(url) = endpoint {
        // Minimal JSON format { messages: [{role, content}, ...] }
        #[derive(Serialize)]
        struct ChatRequest<'a> {
            messages: &'a [ChatMessage],
        }

        #[derive(Deserialize)]
        struct ChatResponse {
            content: String,
        }

        let client = reqwest::Client::new();
        let res = client
            .post(url)
            .json(&ChatRequest {
                messages: &messages,
            })
            .send()
            .await
            .map_err(ChatError::from)?;

        let status = res.status();
        let body_text = res.text().await.map_err(ChatError::from)?;

        if status.is_success() {
            match serde_json::from_str::<ChatResponse>(&body_text) {
                Ok(data) => Ok(data.content),
                Err(_) => Ok(body_text),
            }
        } else {
            Err(ChatError::new(format!(
                "LLM endpoint error {status}: {body_text}"
            )))
        }
    } else if use_ollama {
        // Optional local: Ollama chat API, non-streaming (opt-in)
        #[derive(Serialize)]
        struct OllamaChatRequest<'a> {
            model: &'a str,
            messages: &'a [ChatMessage],
            #[serde(default)]
            stream: bool,
        }

        #[derive(Deserialize)]
        struct OllamaMessage {
            content: String,
        }

        #[derive(Deserialize)]
        struct OllamaChatResponse {
            message: Option<OllamaMessage>,
        }

        let model = env::var("LLM_MODEL").unwrap_or_else(|_| "gpt-oss:20b".to_string());
        let client = reqwest::Client::new();
        let url = "http://127.0.0.1:11434/api/chat";
        let res = client
            .post(url)
            .json(&OllamaChatRequest {
                model: &model,
                messages: &messages,
                stream: false,
            })
            .send()
            .await
            .map_err(ChatError::from)?;

        let status = res.status();
        let body_text = res.text().await.map_err(ChatError::from)?;
        if status.is_success() {
            match serde_json::from_str::<OllamaChatResponse>(&body_text) {
                Ok(data) => {
                    if let Some(msg) = data.message {
                        Ok(msg.content)
                    } else {
                        Ok(body_text)
                    }
                }
                Err(_) => Ok(body_text),
            }
        } else {
            Err(ChatError::new(format!(
                "Ollama error {status}: {body_text}"
            )))
        }
    } else if let Some(bb_url) = bb_endpoint {
        // Hosted: Blackbird chat completions (endpoint provided via env)
        #[derive(Serialize)]
        struct BlackbirdRequest<'a> {
            tier: &'a str,
            model: &'a str,
            messages: &'a [ChatMessage],
        }

        // Try to deserialize OpenAI-like responses first; otherwise accept { content } or raw text
        #[derive(Deserialize)]
        struct BBMessage {
            content: String,
        }
        #[derive(Deserialize)]
        struct BBChoice {
            message: BBMessage,
        }
        #[derive(Deserialize)]
        struct BBResponseOpenAIShape {
            choices: Vec<BBChoice>,
        }
        #[derive(Deserialize)]
        struct BBResponseContentOnly {
            content: String,
        }

        let tier = env::var("BLACKBIRD_TIER").unwrap_or_else(|_| "ultra".to_string());
        let model = env::var("BLACKBIRD_MODEL").unwrap_or_else(|_| "gpt-oss-120b".to_string());
        let api_key = env::var("BLACKBIRD_API_KEY").ok();
        let client = reqwest::Client::new();
        let mut req = client.post(bb_url).json(&BlackbirdRequest {
            tier: &tier,
            model: &model,
            messages: &messages,
        });
        if let Some(key) = api_key {
            req = req.bearer_auth(key);
        }
        let res = req.send().await.map_err(ChatError::from)?;

        let status = res.status();
        let body_text = res.text().await.map_err(ChatError::from)?;
        if status.is_success() {
            // Attempt several shapes
            if let Ok(data) = serde_json::from_str::<BBResponseOpenAIShape>(&body_text)
                && let Some(first) = data.choices.into_iter().next()
            {
                return Ok(first.message.content);
            }
            if let Ok(data) = serde_json::from_str::<BBResponseContentOnly>(&body_text) {
                return Ok(data.content);
            }
            Ok(body_text)
        } else {
            Err(ChatError::new(format!(
                "Blackbird error {status}: {body_text}"
            )))
        }
    } else {
        Err(ChatError::new(
            "No LLM configured. Set LLM_ENDPOINT for a custom backend, BLACKBIRD_ENDPOINT for Blackbird, or LLM_USE_OLLAMA=true for local Ollama.",
        ))
    }
}

// ---------------
// Streaming support (Blackbird SSE or local Ollama)
// ---------------

#[allow(dead_code)]
static STREAMS: Lazy<Mutex<HashMap<u64, (String, bool)>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
#[allow(dead_code)]
static STREAM_COUNTER: AtomicU64 = AtomicU64::new(1);

pub async fn chat_reply_stream_start(messages: Vec<ChatMessage>) -> ChatResult<u64> {
    use std::env;
    let id = STREAM_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    {
        let mut map = STREAMS.lock().unwrap();
        map.insert(id, (String::new(), false));
    }

    // Select provider by env. Priority: custom endpoint (non-streaming) -> Ollama -> Blackbird.
    let use_ollama = matches!(
        env::var("LLM_USE_OLLAMA")
            .unwrap_or_else(|_| "false".into())
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "yes" | "on"
    );
    let has_custom = env::var("LLM_ENDPOINT").is_ok();
    let has_blackbird = env::var("BLACKBIRD_ENDPOINT").is_ok();

    if has_custom {
        let id_copy = id;
        tokio::spawn(async move {
            let full = chat_reply(messages)
                .await
                .unwrap_or_else(|e| format!("error: {}", e));
            let mut map = STREAMS.lock().unwrap();
            if let Some(entry) = map.get_mut(&id_copy) {
                entry.0 = full;
                entry.1 = true;
            }
        });
        return Ok(id);
    }

    if use_ollama {
        tokio::spawn(async move {
            if let Err(e) = stream_from_ollama(id, messages).await {
                eprintln!("ollama stream error: {}", e);
                let mut map = STREAMS.lock().unwrap();
                if let Some(entry) = map.get_mut(&id) {
                    entry.1 = true;
                }
            }
        });
    } else if has_blackbird {
        tokio::spawn(async move {
            if let Err(e) = stream_from_blackbird(id, messages).await {
                eprintln!("blackbird stream error: {}", e);
                let mut map = STREAMS.lock().unwrap();
                if let Some(entry) = map.get_mut(&id) {
                    entry.1 = true;
                }
            }
        });
    } else {
        // No provider configured: immediately complete with a helpful message
        let mut map = STREAMS.lock().unwrap();
        if let Some(entry) = map.get_mut(&id) {
            entry.0 = "No LLM configured. Set LLM_ENDPOINT for a custom backend, BLACKBIRD_ENDPOINT for Blackbird, or LLM_USE_OLLAMA=true for local Ollama.".to_string();
            entry.1 = true;
        }
    }

    Ok(id)
}

pub async fn chat_reply_stream_poll(id: u64) -> ChatResult<(String, bool)> {
    let map = STREAMS.lock().unwrap();
    if let Some((content, done)) = map.get(&id) {
        Ok((content.clone(), *done))
    } else {
        Err(ChatError::new("invalid stream id"))
    }
}

#[allow(dead_code)]
async fn stream_from_ollama(id: u64, messages: Vec<ChatMessage>) -> ChatResult<()> {
    use std::env;

    #[derive(Serialize)]
    struct OllamaChatRequest<'a> {
        model: &'a str,
        messages: &'a [ChatMessage],
        stream: bool,
    }

    let model = env::var("LLM_MODEL").unwrap_or_else(|_| "gpt-oss:20b".to_string());
    let client = reqwest::Client::new();
    let res = client
        .post("http://127.0.0.1:11434/api/chat")
        .json(&OllamaChatRequest {
            model: &model,
            messages: &messages,
            stream: true,
        })
        .send()
        .await
        .map_err(ChatError::from)?;

    let status = res.status();
    if !status.is_success() {
        let body_text = res.text().await.unwrap_or_default();
        return Err(ChatError::new(format!(
            "Ollama error {status}: {body_text}"
        )));
    }

    let mut buffer = String::new();
    let mut stream = res.bytes_stream();
    while let Some(item) = stream.next().await {
        match item {
            Ok(bytes) => {
                let chunk = String::from_utf8_lossy(&bytes);
                buffer.push_str(&chunk);
                while let Some(pos) = buffer.find('\n') {
                    let line_owned = buffer[..pos].to_string();
                    buffer = buffer[pos + 1..].to_string();
                    if let Some((piece, done)) = parse_ollama_stream_line(&line_owned) {
                        if !piece.is_empty() {
                            append_stream(id, &piece);
                        }
                        if done {
                            mark_stream_done(id);
                            return Ok(());
                        }
                    }
                }
            }
            Err(e) => return Err(ChatError::from(e)),
        }
    }

    mark_stream_done(id);
    Ok(())
}

// Types + minimal parser for Ollama JSONL stream lines
#[derive(Deserialize, Debug)]
pub struct StreamChunkMessage {
    pub content: String,
}

#[derive(Deserialize, Debug)]
pub struct StreamChunk {
    pub message: Option<StreamChunkMessage>,
    pub done: Option<bool>,
}

pub fn parse_ollama_stream_line(line_with_ws: &str) -> Option<(String, bool)> {
    let line = line_with_ws.trim();
    if line.is_empty() {
        return None;
    }
    if let Ok(parsed) = serde_json::from_str::<StreamChunk>(line) {
        let mut piece = String::new();
        if let Some(msg) = parsed.message {
            piece.push_str(&msg.content);
        }
        let done = parsed.done.unwrap_or(false);
        return Some((piece, done));
    }
    None
}

#[allow(dead_code)]
fn append_stream(id: u64, piece: &str) {
    let mut map = STREAMS.lock().unwrap();
    if let Some(entry) = map.get_mut(&id) {
        entry.0.push_str(piece);
    }
}

#[allow(dead_code)]
fn mark_stream_done(id: u64) {
    let mut map = STREAMS.lock().unwrap();
    if let Some(entry) = map.get_mut(&id) {
        entry.1 = true;
    }
}

#[allow(dead_code)]
async fn stream_from_blackbird(id: u64, messages: Vec<ChatMessage>) -> ChatResult<()> {
    use std::env;

    #[derive(Serialize)]
    struct BlackbirdRequest<'a> {
        #[serde(skip_serializing_if = "Option::is_none")]
        tier: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        model: Option<&'a str>,
        messages: &'a [ChatMessage],
        stream: bool,
    }
    // Parsing helpers are below as pub fns for tests

    let tier_val = env::var("BLACKBIRD_TIER").unwrap_or_else(|_| "ultra".to_string());
    let model_val = env::var("BLACKBIRD_MODEL").unwrap_or_else(|_| "gpt-oss-120b".to_string());
    let api_key = env::var("BLACKBIRD_API_KEY").ok();

    let endpoint =
        env::var("BLACKBIRD_ENDPOINT").map_err(|_| ChatError::new("BLACKBIRD_ENDPOINT not set"))?;
    let client = reqwest::Client::new();
    let mut req = client
        .post(endpoint)
        .header("accept", "text/event-stream")
        .json(&BlackbirdRequest {
            tier: Some(&tier_val),
            model: Some(&model_val),
            messages: &messages,
            stream: true,
        });
    if let Some(key) = api_key {
        req = req.bearer_auth(key);
    }
    let res = req.send().await.map_err(ChatError::from)?;

    let status = res.status();
    if !status.is_success() {
        let body_text = res.text().await.unwrap_or_default();
        return Err(ChatError::new(format!(
            "Blackbird error {status}: {body_text}"
        )));
    }

    // Parse SSE by lines. Collect consecutive data: lines (if any) until a blank line, then process.
    let mut buffer = String::new();
    let mut data_acc: Option<String> = None;
    let mut stream = res.bytes_stream();
    while let Some(item) = stream.next().await {
        match item {
            Ok(bytes) => {
                let chunk = String::from_utf8_lossy(&bytes);
                buffer.push_str(&chunk);
                while let Some(pos) = buffer.find('\n') {
                    let mut line = buffer[..pos].to_string();
                    if line.ends_with('\r') {
                        line.pop();
                    }
                    buffer = buffer[pos + 1..].to_string();

                    if line.is_empty() {
                        // End of event
                        if let Some(data) = data_acc.take()
                            && let Some((piece, done)) = parse_blackbird_sse_data(&data)
                        {
                            if !piece.is_empty() {
                                append_stream(id, &piece);
                            }
                            if done {
                                mark_stream_done(id);
                                return Ok(());
                            }
                        }
                        continue;
                    }

                    if let Some(rest) = line.strip_prefix("data:") {
                        let s = rest.trim_start();
                        match &mut data_acc {
                            Some(acc) => {
                                acc.push_str(s);
                            }
                            None => {
                                data_acc = Some(s.to_string());
                            }
                        }
                    }
                }
            }
            Err(e) => return Err(ChatError::from(e)),
        }
    }

    mark_stream_done(id);
    Ok(())
}

// -----------------
// Blackbird SSE parsing helpers (exported for tests)
// -----------------

#[derive(Deserialize)]
pub struct BBMessage {
    pub content: String,
}
#[derive(Deserialize)]
pub struct BBChoice {
    pub message: Option<BBMessage>,
    #[serde(default)]
    pub delta: Option<OAIDelta>,
}
#[derive(Deserialize)]
pub struct BBResponseOpenAIShape {
    pub choices: Vec<BBChoice>,
}
#[derive(Deserialize)]
pub struct BBResponseContentOnly {
    pub content: String,
}
#[derive(Deserialize)]
pub struct OAIDelta {
    pub content: Option<String>,
}

#[allow(dead_code)]
pub fn parse_blackbird_sse_data(data: &str) -> Option<(String, bool)> {
    let trimmed = data.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed == "[DONE]" {
        return Some((String::new(), true));
    }

    if let Ok(parsed) = serde_json::from_str::<BBResponseOpenAIShape>(trimmed) {
        if let Some(first) = parsed.choices.into_iter().next() {
            if let Some(delta) = first.delta
                && let Some(piece) = delta.content
            {
                return Some((piece, false));
            }
            if let Some(msg) = first.message {
                return Some((msg.content, false));
            }
        }
        return Some((String::new(), false));
    }
    if let Ok(parsed) = serde_json::from_str::<BBResponseContentOnly>(trimmed) {
        return Some((parsed.content, false));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_ollama_stream_lines() {
        let lines = vec![
            r#"{"message":{"content":"Hello"},"done":false}"#,
            r#"{"message":{"content":" world"},"done":false}"#,
            r#"{"done":true}"#,
        ];
        let mut acc = String::new();
        let mut finished = false;
        for l in lines {
            if let Some((piece, done)) = parse_ollama_stream_line(l) {
                acc.push_str(&piece);
                finished = done;
            }
        }
        assert_eq!(acc, "Hello world");
        assert!(finished);
    }
}
