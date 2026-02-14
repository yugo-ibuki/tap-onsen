pub mod client;
pub mod context;
pub mod prompt;
pub mod streaming;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

/// AI処理のエラー型
#[derive(Debug)]
pub enum AIError {
    RequestFailed(String),
    ApiKeyMissing(String),
    ParseError(String),
    Timeout,
    StreamError(String),
}

impl std::fmt::Display for AIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AIError::RequestFailed(msg) => write!(f, "API request failed: {}", msg),
            AIError::ApiKeyMissing(key) => write!(f, "API key not found: {}", key),
            AIError::ParseError(msg) => write!(f, "Failed to parse response: {}", msg),
            AIError::Timeout => write!(f, "Request timed out"),
            AIError::StreamError(msg) => write!(f, "Stream error: {}", msg),
        }
    }
}

impl std::error::Error for AIError {}

/// ストリーミングチャンク
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    pub content: String,
    pub done: bool,
}

/// AI処理結果
#[derive(Debug, Serialize, Deserialize)]
pub struct AIResponse {
    pub text: String,
    pub model: String,
    pub usage: Option<TokenUsage>,
}

/// トークン使用量
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// AIプロバイダーの抽象trait
#[async_trait]
pub trait AIProvider: Send + Sync {
    /// テキストを処理して結果を返す（非ストリーミング）
    async fn process(&self, prompt: &str) -> Result<AIResponse, AIError>;

    /// テキストをストリーミング処理する
    async fn process_stream(
        &self,
        prompt: &str,
        sender: mpsc::Sender<StreamChunk>,
    ) -> Result<(), AIError>;
}

/// サポートするAIプロバイダーの種別
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ProviderType {
    VertexAI,
    OpenAI,
    Anthropic,
}

impl Default for ProviderType {
    fn default() -> Self {
        ProviderType::VertexAI
    }
}
