pub mod format;
#[cfg(target_os = "macos")]
pub mod macos_speech;
pub mod pipeline;
pub mod whisper_api;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;

/// 音声認識エンジンが返す文字起こし結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionResult {
    pub text: String,
    pub confidence: f64,
    pub is_final: bool,
    pub timestamp: u64,
}

/// 音声認識で発生しうるエラー
#[derive(Debug)]
pub enum VoiceError {
    /// 音声フォーマット変換エラー
    FormatError(String),
    /// API 通信エラー
    ApiError(String),
    /// API キーが未設定
    MissingApiKey,
    /// パイプラインエラー
    PipelineError(String),
    /// macOS Speech Framework エラー
    NativeError(String),
    /// 音声認識の権限が未承認
    PermissionDenied,
}

impl fmt::Display for VoiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VoiceError::FormatError(msg) => write!(f, "Format error: {}", msg),
            VoiceError::ApiError(msg) => write!(f, "API error: {}", msg),
            VoiceError::MissingApiKey => write!(f, "OPENAI_API_KEY environment variable is not set"),
            VoiceError::PipelineError(msg) => write!(f, "Pipeline error: {}", msg),
            VoiceError::NativeError(msg) => write!(f, "Native speech error: {}", msg),
            VoiceError::PermissionDenied => write!(f, "Speech recognition permission denied"),
        }
    }
}

impl std::error::Error for VoiceError {}

impl From<VoiceError> for String {
    fn from(e: VoiceError) -> Self {
        e.to_string()
    }
}

/// 音声認識エンジンの共通インターフェース
///
/// Whisper API、whisper.cpp、macOS native など複数のバックエンドを
/// 統一的に扱うための trait。
#[async_trait]
pub trait SpeechRecognizer: Send + Sync {
    /// 音声データを文字起こしする
    ///
    /// # Arguments
    /// * `audio_data` - PCM 形式の音声データ (f32 サンプル、16kHz、モノラル)
    /// * `language` - 言語コード (例: "ja", "en")
    async fn transcribe(
        &self,
        audio_data: &[u8],
        language: &str,
    ) -> Result<TranscriptionResult, VoiceError>;
}
