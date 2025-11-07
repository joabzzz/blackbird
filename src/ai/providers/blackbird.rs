use crate::types::ChatMessage;
use anyhow::Result;
use serde::Deserialize;

/// Custom client for Blackbird API endpoint
pub struct BlackbirdClient {
    client: reqwest::Client,
    endpoint: String,
    tier: String,
    model: String,
    api_key: Option<String>,
}

// Blackbird API response types
#[derive(Deserialize)]
struct BBMessage {
    content: String,
}

#[derive(Deserialize)]
struct BBChoice {
    message: Option<BBMessage>,
}

#[derive(Deserialize)]
struct BBResponseOpenAIShape {
    choices: Vec<BBChoice>,
}

#[derive(Deserialize)]
struct BBResponseContentOnly {
    content: String,
}

#[derive(serde::Serialize)]
struct BlackbirdRequest<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    tier: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<&'a str>,
    messages: &'a [ChatMessage],
}

impl BlackbirdClient {
    pub fn new(endpoint: String, tier: String, model: String, api_key: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            endpoint,
            tier,
            model,
            api_key,
        }
    }

    pub async fn complete(&self, messages: &[ChatMessage]) -> Result<String> {
        let mut request = self.client.post(&self.endpoint).json(&BlackbirdRequest {
            tier: Some(&self.tier),
            model: Some(&self.model),
            messages,
        });

        if let Some(key) = &self.api_key {
            request = request.bearer_auth(key);
        }

        let response = request.send().await?;
        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            return Err(anyhow::anyhow!("Blackbird API error {}: {}", status, body));
        }

        // Try OpenAI-shaped response first
        if let Ok(parsed) = serde_json::from_str::<BBResponseOpenAIShape>(&body)
            && let Some(choice) = parsed.choices.into_iter().next()
            && let Some(msg) = choice.message
        {
            return Ok(msg.content);
        }

        // Try content-only response
        if let Ok(parsed) = serde_json::from_str::<BBResponseContentOnly>(&body) {
            return Ok(parsed.content);
        }

        // Fallback to raw body
        Ok(body)
    }
}
