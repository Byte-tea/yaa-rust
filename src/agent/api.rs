use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("API request failed: {0}")]
    RequestFailed(String),
    #[error("API response parsing failed: {0}")]
    ParseFailed(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[async_trait]
pub trait OpenAIApi {
    async fn chat_completion(
        &self,
        messages: Vec<Message>,
        model: &str,
        temperature: f32,
    ) -> Result<String, ApiError>;
}

pub struct OpenAIClient {
    api_key: String,
    base_url: String,
    client: reqwest::Client,
}

impl OpenAIClient {
    pub fn new(api_key: String, base_url: Option<String>) -> Self {
        Self {
            api_key,
            base_url: base_url.unwrap_or_else(|| "https://api.deepseek.com".to_string()),
            client: reqwest::Client::new(),
        }
    }
}

impl Default for OpenAIClient {
    fn default() -> Self {
        let api_key = std::env::var("OPENAI_API_KEY")
            .unwrap_or_else(|_| "".to_string());
        let base_url = std::env::var("OPENAI_API_BASE")
            .ok()
            .unwrap_or_else(|| "https://api.deepseek.com".to_string());
        
        Self::new(api_key, Some(base_url))
    }
}

#[async_trait]
impl OpenAIApi for OpenAIClient {
    async fn chat_completion(
        &self,
        messages: Vec<Message>,
        model: &str,
        temperature: f32,
    ) -> Result<String, ApiError> {
        (&self).chat_completion(messages, model, temperature).await
    }
}

#[async_trait]
impl OpenAIApi for &OpenAIClient {
    async fn chat_completion(
        &self,
        messages: Vec<Message>,
        model: &str,
        temperature: f32,
    ) -> Result<String, ApiError> {
        let response = self.client
            .post(format!("{}/v1/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "model": model,
                "messages": messages,
                "temperature": temperature,
            }))
            .send()
            .await
            .map_err(|e| ApiError::RequestFailed(e.to_string()))?;

        let response_text = response.text()
            .await
            .map_err(|e| ApiError::ParseFailed(e.to_string()))?;

        let json: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| ApiError::ParseFailed(format!("{} - Response: {}", e, response_text)))?;

        json["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| ApiError::ParseFailed(format!("Invalid response format - Response: {}", response_text)))
    }
}
