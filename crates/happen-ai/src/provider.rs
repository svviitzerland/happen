use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AiProviderConfig {
    pub provider: String,
    pub model: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
}

impl Default for AiProviderConfig {
    fn default() -> Self {
        Self {
            provider: "anthropic".to_string(),
            model: "claude-sonnet-4-6".to_string(),
            api_key: None,
            base_url: None,
        }
    }
}

#[async_trait::async_trait]
pub trait AiProvider: Send + Sync {
    async fn generate(&self, prompt: &str, system_prompt: &str) -> Result<String, AiError>;
}

#[derive(Debug)]
pub enum AiError {
    Network(String),
    Parse(String),
    ApiError(String),
    NoApiKey,
}

impl std::fmt::Display for AiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AiError::Network(e) => write!(f, "Network error: {}", e),
            AiError::Parse(e) => write!(f, "Parse error: {}", e),
            AiError::ApiError(e) => write!(f, "API error: {}", e),
            AiError::NoApiKey => write!(f, "No API key configured"),
        }
    }
}

impl std::error::Error for AiError {}

pub struct AnthropicProvider {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl AnthropicProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            api_key,
            model,
            client: reqwest::Client::new(),
        }
    }

    pub fn from_config(config: &AiProviderConfig) -> Result<Self, AiError> {
        let api_key = config
            .api_key
            .clone()
            .or_else(|| std::env::var("ANTHROPIC_API_KEY").ok())
            .ok_or(AiError::NoApiKey)?;

        Ok(Self::new(api_key, config.model.clone()))
    }
}

#[async_trait::async_trait]
impl AiProvider for AnthropicProvider {
    async fn generate(&self, prompt: &str, system_prompt: &str) -> Result<String, AiError> {
        let body = serde_json::json!({
            "model": self.model,
            "max_tokens": 4096,
            "system": system_prompt,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ]
        });

        let base_url = "https://api.anthropic.com/v1/messages";

        let response = self
            .client
            .post(base_url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| AiError::Network(e.to_string()))?;

        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(|e| AiError::Network(e.to_string()))?;

        if !status.is_success() {
            return Err(AiError::ApiError(format!("HTTP {}: {}", status, text)));
        }

        let json: serde_json::Value =
            serde_json::from_str(&text).map_err(|e| AiError::Parse(e.to_string()))?;

        json["content"][0]["text"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| AiError::Parse("No text in response".to_string()))
    }
}

pub struct OpenRouterProvider {
    api_key: String,
    model: String,
    client: reqwest::Client,
    base_url: String,
}

impl OpenRouterProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            api_key,
            model,
            client: reqwest::Client::new(),
            base_url: "https://openrouter.ai/api/v1/chat/completions".to_string(),
        }
    }

    pub fn with_base_url(mut self, url: String) -> Self {
        self.base_url = url;
        self
    }

    pub fn from_config(config: &AiProviderConfig) -> Result<Self, AiError> {
        let api_key = config
            .api_key
            .clone()
            .or_else(|| std::env::var("OPENROUTER_API_KEY").ok())
            .ok_or(AiError::NoApiKey)?;

        let mut provider = Self::new(api_key, config.model.clone());
        if let Some(ref url) = config.base_url {
            provider.base_url = url.clone();
        }
        Ok(provider)
    }
}

#[async_trait::async_trait]
impl AiProvider for OpenRouterProvider {
    async fn generate(&self, prompt: &str, system_prompt: &str) -> Result<String, AiError> {
        let body = serde_json::json!({
            "model": self.model,
            "max_tokens": 4096,
            "messages": [
                {
                    "role": "system",
                    "content": system_prompt
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ]
        });

        let response = self
            .client
            .post(&self.base_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("HTTP-Referer", "https://github.com/happen-engine")
            .header("X-Title", "Happen Engine")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| AiError::Network(e.to_string()))?;

        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(|e| AiError::Network(e.to_string()))?;

        if !status.is_success() {
            return Err(AiError::ApiError(format!("HTTP {}: {}", status, text)));
        }

        let json: serde_json::Value =
            serde_json::from_str(&text).map_err(|e| AiError::Parse(e.to_string()))?;

        json["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| AiError::Parse("No content in response".to_string()))
    }
}
