use super::{ChatError, ChatResult, LLMBackend, StreamHandle};
use crate::types::ChatMessage;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct CustomBackend {
    endpoint: String,
    client: Client,
}

impl CustomBackend {
    pub fn new(endpoint: String) -> Self {
        Self {
            endpoint,
            client: Client::new(),
        }
    }
}

#[derive(Serialize)]
struct ChatRequest<'a> {
    messages: &'a [ChatMessage],
}

#[derive(Deserialize)]
struct ChatResponse {
    content: String,
}

#[async_trait]
impl LLMBackend for CustomBackend {
    async fn complete(&self, messages: &[ChatMessage]) -> ChatResult<String> {
        let response = self
            .client
            .post(&self.endpoint)
            .json(&ChatRequest { messages })
            .send()
            .await?;
        let status = response.status();
        let body = response.text().await?;
        if status.is_success() {
            match serde_json::from_str::<ChatResponse>(&body) {
                Ok(data) => Ok(data.content),
                Err(_) => Ok(body),
            }
        } else {
            Err(ChatError::new(format!(
                "LLM endpoint error {status}: {body}"
            )))
        }
    }

    async fn stream(&self, messages: &[ChatMessage], handle: StreamHandle) -> ChatResult<()> {
        let content = self.complete(messages).await?;
        handle.replace(content);
        Ok(())
    }
}
