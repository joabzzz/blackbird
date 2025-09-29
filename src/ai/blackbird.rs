use super::{ChatError, ChatResult, LLMBackend, StreamHandle};
use crate::types::ChatMessage;
use async_trait::async_trait;
use futures_util::StreamExt;
use reqwest::Client;
use serde::Deserialize;

const DEFAULT_TIER: &str = "ultra";
const DEFAULT_MODEL: &str = "gpt-oss-120b";

pub struct BlackbirdBackend {
    client: Client,
    endpoint: String,
    tier: String,
    model: String,
    api_key: Option<String>,
}

impl BlackbirdBackend {
    pub fn from_env(endpoint: String) -> Self {
        let tier = std::env::var("BLACKBIRD_TIER").unwrap_or_else(|_| DEFAULT_TIER.to_string());
        let model = std::env::var("BLACKBIRD_MODEL").unwrap_or_else(|_| DEFAULT_MODEL.to_string());
        let api_key = std::env::var("BLACKBIRD_API_KEY").ok();
        Self {
            client: Client::new(),
            endpoint,
            tier,
            model,
            api_key,
        }
    }
}

#[derive(serde::Serialize)]
struct BlackbirdRequest<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    tier: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<&'a str>,
    messages: &'a [ChatMessage],
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

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
            if let Some(delta) = first.delta {
                if let Some(piece) = delta.content {
                    return Some((piece, false));
                }
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

#[async_trait]
impl LLMBackend for BlackbirdBackend {
    async fn complete(&self, messages: &[ChatMessage]) -> ChatResult<String> {
        let mut request = self.client.post(&self.endpoint).json(&BlackbirdRequest {
            tier: Some(&self.tier),
            model: Some(&self.model),
            messages,
            stream: None,
        });
        if let Some(key) = &self.api_key {
            request = request.bearer_auth(key);
        }

        let response = request.send().await?;
        let status = response.status();
        let body = response.text().await?;
        if status.is_success() {
            if let Ok(parsed) = serde_json::from_str::<BBResponseOpenAIShape>(&body) {
                if let Some(choice) = parsed.choices.into_iter().next() {
                    if let Some(msg) = choice.message {
                        return Ok(msg.content);
                    }
                }
            }
            if let Ok(parsed) = serde_json::from_str::<BBResponseContentOnly>(&body) {
                return Ok(parsed.content);
            }
            Ok(body)
        } else {
            Err(ChatError::new(format!("Blackbird error {status}: {body}")))
        }
    }

    fn supports_streaming(&self) -> bool {
        true
    }

    async fn stream(&self, messages: &[ChatMessage], handle: StreamHandle) -> ChatResult<()> {
        let mut request = self
            .client
            .post(&self.endpoint)
            .header("accept", "text/event-stream")
            .json(&BlackbirdRequest {
                tier: Some(&self.tier),
                model: Some(&self.model),
                messages,
                stream: Some(true),
            });
        if let Some(key) = &self.api_key {
            request = request.bearer_auth(key);
        }

        let response = request.send().await?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(ChatError::new(format!("Blackbird error {status}: {body}")));
        }

        let mut buffer = String::new();
        let mut data_acc: Option<String> = None;
        let mut stream = response.bytes_stream();
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
                            if let Some(data) = data_acc.take() {
                                if let Some((piece, done)) = parse_blackbird_sse_data(&data) {
                                    if !piece.is_empty() {
                                        handle.append(&piece);
                                    }
                                    if done {
                                        return Ok(());
                                    }
                                }
                            }
                            continue;
                        }

                        if let Some(rest) = line.strip_prefix("data:") {
                            let fragment = rest.trim_start();
                            match &mut data_acc {
                                Some(existing) => existing.push_str(fragment),
                                None => data_acc = Some(fragment.to_string()),
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
    use super::parse_blackbird_sse_data;

    #[test]
    fn parses_blackbird_data() {
        assert!(parse_blackbird_sse_data("").is_none());
        assert_eq!(
            parse_blackbird_sse_data("[DONE]"),
            Some((String::new(), true))
        );
        assert_eq!(
            parse_blackbird_sse_data(r#"{"choices":[{"delta":{"content":"hello"}}]}"#),
            Some(("hello".to_string(), false))
        );
        assert_eq!(
            parse_blackbird_sse_data(r#"{"content":"hi"}"#),
            Some(("hi".to_string(), false))
        );
    }
}
