use async_trait::async_trait;
use reqwest::multipart;

use crate::voice::{SpeechRecognizer, TranscriptionResult, VoiceError};

const WHISPER_API_URL: &str = "https://api.openai.com/v1/audio/transcriptions";
const WHISPER_MODEL: &str = "whisper-1";

/// OpenAI Whisper API のレスポンス
#[derive(Debug, serde::Deserialize)]
struct WhisperResponse {
    text: String,
}

/// OpenAI Whisper API を使った音声認識クライアント
pub struct WhisperApiClient {
    client: reqwest::Client,
    api_key: String,
}

impl WhisperApiClient {
    /// 環境変数 `OPENAI_API_KEY` から API キーを取得して初期化する
    pub fn from_env() -> Result<Self, VoiceError> {
        let api_key =
            std::env::var("OPENAI_API_KEY").map_err(|_| VoiceError::MissingApiKey)?;
        Ok(Self {
            client: reqwest::Client::new(),
            api_key,
        })
    }

    /// 指定の API キーで初期化する
    pub fn new(api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
        }
    }
}

#[async_trait]
impl SpeechRecognizer for WhisperApiClient {
    /// WAV 形式の音声データを Whisper API に送信して文字起こしする
    ///
    /// `audio_data` は WAV ファイルのバイト列。
    /// format モジュールで PCM → WAV 変換した後にこのメソッドを呼ぶ。
    async fn transcribe(
        &self,
        audio_data: &[u8],
        language: &str,
    ) -> Result<TranscriptionResult, VoiceError> {
        let file_part = multipart::Part::bytes(audio_data.to_vec())
            .file_name("audio.wav")
            .mime_str("audio/wav")
            .map_err(|e| VoiceError::ApiError(format!("Failed to create multipart: {}", e)))?;

        let form = multipart::Form::new()
            .part("file", file_part)
            .text("model", WHISPER_MODEL.to_string())
            .text("language", language.to_string());

        let response = self
            .client
            .post(WHISPER_API_URL)
            .bearer_auth(&self.api_key)
            .multipart(form)
            .send()
            .await
            .map_err(|e| VoiceError::ApiError(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(VoiceError::ApiError(format!(
                "Whisper API returned {}: {}",
                status, body
            )));
        }

        let whisper_response: WhisperResponse = response
            .json()
            .await
            .map_err(|e| VoiceError::ApiError(format!("Failed to parse response: {}", e)))?;

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Ok(TranscriptionResult {
            text: whisper_response.text,
            confidence: 1.0, // Whisper API は信頼度スコアを返さないためデフォルト値
            is_final: true,
            timestamp,
        })
    }
}
