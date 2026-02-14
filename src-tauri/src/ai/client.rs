use async_trait::async_trait;
use reqwest::Client;
use std::time::Duration;
use tokio::sync::mpsc;

use super::streaming::{parse_anthropic_stream, parse_openai_stream};
use super::{AIError, AIProvider, AIResponse, ProviderType, StreamChunk, TokenUsage};

const DEFAULT_TIMEOUT_SECS: u64 = 30;
const OPENAI_API_URL: &str = "https://api.openai.com/v1/chat/completions";
const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";

/// OpenAI APIクライアント
pub struct OpenAIClient {
    client: Client,
    api_key: String,
    model: String,
}

impl OpenAIClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
                .build()
                .unwrap(),
            api_key,
            model: "gpt-4o-mini".to_string(),
        }
    }

    fn build_request_body(&self, prompt: &str, stream: bool) -> serde_json::Value {
        serde_json::json!({
            "model": self.model,
            "messages": [
                { "role": "user", "content": prompt }
            ],
            "stream": stream,
        })
    }
}

#[async_trait]
impl AIProvider for OpenAIClient {
    async fn process(&self, prompt: &str) -> Result<AIResponse, AIError> {
        let body = self.build_request_body(prompt, false);

        let response = self
            .client
            .post(OPENAI_API_URL)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    AIError::Timeout
                } else {
                    AIError::RequestFailed(e.to_string())
                }
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AIError::RequestFailed(format!(
                "HTTP {}: {}",
                status, text
            )));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AIError::ParseError(e.to_string()))?;

        let text = json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let usage = json.get("usage").map(|u| TokenUsage {
            prompt_tokens: u["prompt_tokens"].as_u64().unwrap_or(0) as u32,
            completion_tokens: u["completion_tokens"].as_u64().unwrap_or(0) as u32,
            total_tokens: u["total_tokens"].as_u64().unwrap_or(0) as u32,
        });

        Ok(AIResponse {
            text,
            model: self.model.clone(),
            usage,
        })
    }

    async fn process_stream(
        &self,
        prompt: &str,
        sender: mpsc::Sender<StreamChunk>,
    ) -> Result<(), AIError> {
        let body = self.build_request_body(prompt, true);

        let response = self
            .client
            .post(OPENAI_API_URL)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    AIError::Timeout
                } else {
                    AIError::RequestFailed(e.to_string())
                }
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AIError::RequestFailed(format!(
                "HTTP {}: {}",
                status, text
            )));
        }

        parse_openai_stream(response, sender).await?;
        Ok(())
    }
}

/// Anthropic APIクライアント
pub struct AnthropicClient {
    client: Client,
    api_key: String,
    model: String,
}

impl AnthropicClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
                .build()
                .unwrap(),
            api_key,
            model: "claude-haiku-4-5-20251001".to_string(),
        }
    }

    fn build_request_body(&self, prompt: &str, stream: bool) -> serde_json::Value {
        serde_json::json!({
            "model": self.model,
            "max_tokens": 1024,
            "messages": [
                { "role": "user", "content": prompt }
            ],
            "stream": stream,
        })
    }
}

#[async_trait]
impl AIProvider for AnthropicClient {
    async fn process(&self, prompt: &str) -> Result<AIResponse, AIError> {
        let body = self.build_request_body(prompt, false);

        let response = self
            .client
            .post(ANTHROPIC_API_URL)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    AIError::Timeout
                } else {
                    AIError::RequestFailed(e.to_string())
                }
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AIError::RequestFailed(format!(
                "HTTP {}: {}",
                status, text
            )));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AIError::ParseError(e.to_string()))?;

        let text = json["content"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let usage = json.get("usage").map(|u| TokenUsage {
            prompt_tokens: u["input_tokens"].as_u64().unwrap_or(0) as u32,
            completion_tokens: u["output_tokens"].as_u64().unwrap_or(0) as u32,
            total_tokens: (u["input_tokens"].as_u64().unwrap_or(0)
                + u["output_tokens"].as_u64().unwrap_or(0)) as u32,
        });

        Ok(AIResponse {
            text,
            model: self.model.clone(),
            usage,
        })
    }

    async fn process_stream(
        &self,
        prompt: &str,
        sender: mpsc::Sender<StreamChunk>,
    ) -> Result<(), AIError> {
        let body = self.build_request_body(prompt, true);

        let response = self
            .client
            .post(ANTHROPIC_API_URL)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    AIError::Timeout
                } else {
                    AIError::RequestFailed(e.to_string())
                }
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AIError::RequestFailed(format!(
                "HTTP {}: {}",
                status, text
            )));
        }

        parse_anthropic_stream(response, sender).await?;
        Ok(())
    }
}

/// Vertex AI Gemini Flash クライアント
///
/// `gcloud auth print-access-token` で OAuth2 トークンを取得し、
/// Vertex AI の generateContent エンドポイントを呼び出す。
pub struct VertexAIClient {
    client: Client,
    project: String,
    location: String,
    model: String,
}

impl VertexAIClient {
    pub fn new(project: String, location: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
                .build()
                .unwrap(),
            project,
            location,
            model: "gemini-2.0-flash".to_string(),
        }
    }

    fn endpoint(&self) -> String {
        format!(
            "https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models/{}:generateContent",
            self.location, self.project, self.location, self.model
        )
    }

    async fn get_access_token() -> Result<String, AIError> {
        let output = tokio::process::Command::new("gcloud")
            .args(["auth", "print-access-token"])
            .output()
            .await
            .map_err(|e| AIError::RequestFailed(format!("Failed to run gcloud: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AIError::ApiKeyMissing(format!(
                "gcloud auth failed: {}",
                stderr.trim()
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}

#[async_trait]
impl AIProvider for VertexAIClient {
    async fn process(&self, prompt: &str) -> Result<AIResponse, AIError> {
        let token = Self::get_access_token().await?;

        let body = serde_json::json!({
            "contents": [
                { "role": "user", "parts": [{ "text": prompt }] }
            ]
        });

        let response = self
            .client
            .post(&self.endpoint())
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    AIError::Timeout
                } else {
                    AIError::RequestFailed(e.to_string())
                }
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AIError::RequestFailed(format!("HTTP {}: {}", status, text)));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AIError::ParseError(e.to_string()))?;

        let text = json["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let usage = json.get("usageMetadata").map(|u| TokenUsage {
            prompt_tokens: u["promptTokenCount"].as_u64().unwrap_or(0) as u32,
            completion_tokens: u["candidatesTokenCount"].as_u64().unwrap_or(0) as u32,
            total_tokens: u["totalTokenCount"].as_u64().unwrap_or(0) as u32,
        });

        Ok(AIResponse {
            text,
            model: self.model.clone(),
            usage,
        })
    }

    async fn process_stream(
        &self,
        prompt: &str,
        sender: mpsc::Sender<StreamChunk>,
    ) -> Result<(), AIError> {
        let response = self.process(prompt).await?;
        let _ = sender
            .send(StreamChunk {
                content: response.text,
                done: true,
            })
            .await;
        Ok(())
    }
}

/// プロバイダーに応じたクライアントを生成する
pub fn create_provider(provider_type: &ProviderType) -> Result<Box<dyn AIProvider>, AIError> {
    match provider_type {
        ProviderType::VertexAI => {
            let project = std::env::var("GOOGLE_CLOUD_PROJECT")
                .map_err(|_| AIError::ApiKeyMissing("GOOGLE_CLOUD_PROJECT".to_string()))?;
            let location =
                std::env::var("GOOGLE_CLOUD_LOCATION").unwrap_or_else(|_| "us-central1".into());
            Ok(Box::new(VertexAIClient::new(project, location)))
        }
        ProviderType::OpenAI => {
            let api_key = std::env::var("OPENAI_API_KEY")
                .map_err(|_| AIError::ApiKeyMissing("OPENAI_API_KEY".to_string()))?;
            Ok(Box::new(OpenAIClient::new(api_key)))
        }
        ProviderType::Anthropic => {
            let api_key = std::env::var("ANTHROPIC_API_KEY")
                .map_err(|_| AIError::ApiKeyMissing("ANTHROPIC_API_KEY".to_string()))?;
            Ok(Box::new(AnthropicClient::new(api_key)))
        }
    }
}
