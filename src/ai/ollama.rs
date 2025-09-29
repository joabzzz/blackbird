use super::{ChatError, ChatResult, LLMBackend, StreamHandle};
use crate::types::ChatMessage;
use async_trait::async_trait;
use futures_util::StreamExt;
use reqwest::Client;
use serde::Deserialize;

const DEFAULT_MODEL: &str = "gpt-oss:20b";
const DEFAULT_ENDPOINT: &str = "http://127.0.0.1:11434/api/chat";

pub struct OllamaBackend {
    client: Client,
    model: String,
    endpoint: String,
}

impl OllamaBackend {
    pub fn from_env() -> Self {
        let model = std::env::var("LLM_MODEL").unwrap_or_else(|_| DEFAULT_MODEL.to_string());
        Self {
            client: Client::new(),
            model,
            endpoint: DEFAULT_ENDPOINT.to_string(),
        }
    }
}

#[derive(serde::Serialize)]
struct OllamaChatRequest<'a> {
    model: &'a str,
    messages: &'a [ChatMessage],
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

#[async_trait]
impl LLMBackend for OllamaBackend {
    async fn complete(&self, messages: &[ChatMessage]) -> ChatResult<String> {
        let response = self
            .client
            .post(&self.endpoint)
            .json(&OllamaChatRequest {
                model: &self.model,
                messages,
                stream: false,
            })
            .send()
            .await?;
        let status = response.status();
        let body = response.text().await?;

        if status.is_success() {
            match serde_json::from_str::<OllamaChatResponse>(&body) {
                Ok(parsed) => Ok(parsed.message.map(|msg| msg.content).unwrap_or(body)),
                Err(_) => Ok(body),
            }
        } else {
            Err(ChatError::new(format!("Ollama error {status}: {body}")))
        }
    }

    fn supports_streaming(&self) -> bool {
        true
    }

    async fn stream(&self, messages: &[ChatMessage], handle: StreamHandle) -> ChatResult<()> {
        let response = self
            .client
            .post(&self.endpoint)
            .json(&OllamaChatRequest {
                model: &self.model,
                messages,
                stream: true,
            })
            .send()
            .await?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(ChatError::new(format!("Ollama error {status}: {body}")));
        }

        let mut buffer = String::new();
        let mut stream = response.bytes_stream();
        while let Some(item) = stream.next().await {
            match item {
                Ok(bytes) => {
                    let chunk = String::from_utf8_lossy(&bytes);
                    buffer.push_str(&chunk);
                    while let Some(pos) = buffer.find('\n') {
                        let line = buffer[..pos].to_string();
                        buffer = buffer[pos + 1..].to_string();
                        if let Some((piece, done)) = parse_ollama_stream_line(&line) {
                            if !piece.is_empty() {
                                handle.append(&piece);
                            }
                            if done {
                                return Ok(());
                            }
                        }
                    }
                }
                Err(err) => return Err(ChatError::from(err)),
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::parse_ollama_stream_line;

    #[test]
    fn parses_stream_lines() {
        let mut acc = String::new();
        let mut done = false;
        for line in [
            r#"{"message":{"content":"Hello"},"done":false}"#,
            r#"{"message":{"content":" world"},"done":false}"#,
            r#"{"done":true}"#,
        ] {
            if let Some((piece, finished)) = parse_ollama_stream_line(line) {
                acc.push_str(&piece);
                done = finished;
            }
        }
        assert_eq!(acc, "Hello world");
        assert!(done);
    }
}
