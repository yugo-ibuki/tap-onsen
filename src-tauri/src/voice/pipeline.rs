use crate::voice::format::{self, MONO_CHANNELS, WHISPER_SAMPLE_RATE};
use crate::voice::{SpeechRecognizer, TranscriptionResult, VoiceError};

/// チャンク分割のデフォルトサイズ（サンプル数）
/// 16kHz × 5秒 = 80,000 サンプル
const DEFAULT_CHUNK_SAMPLES: usize = 80_000;

/// 音声データをチャンクに分割して逐次文字起こしするパイプライン
pub struct TranscriptionPipeline<R: SpeechRecognizer> {
    recognizer: R,
    chunk_samples: usize,
    language: String,
}

impl<R: SpeechRecognizer> TranscriptionPipeline<R> {
    pub fn new(recognizer: R, language: &str) -> Self {
        Self {
            recognizer,
            chunk_samples: DEFAULT_CHUNK_SAMPLES,
            language: language.to_string(),
        }
    }

    /// チャンクサイズをカスタムで設定する（テスト用途など）
    pub fn with_chunk_samples(mut self, samples: usize) -> Self {
        self.chunk_samples = samples;
        self
    }

    /// PCM f32 音声データ全体を一括で文字起こしする
    ///
    /// 短い音声（数秒〜十数秒）向け。チャンク分割せず全体を送信する。
    pub async fn transcribe_all(
        &self,
        pcm_f32: &[f32],
    ) -> Result<TranscriptionResult, VoiceError> {
        let wav_data = format::pcm_f32_to_wav(pcm_f32, WHISPER_SAMPLE_RATE, MONO_CHANNELS)?;
        self.recognizer
            .transcribe(&wav_data, &self.language)
            .await
    }

    /// PCM f32 音声データをチャンクに分割して逐次文字起こしする
    ///
    /// 長い音声データに対して使用。各チャンクの結果を結合して返す。
    /// コールバックで各チャンクの interim result を受け取れる。
    pub async fn transcribe_chunked(
        &self,
        pcm_f32: &[f32],
        on_interim: Option<&dyn Fn(&TranscriptionResult)>,
    ) -> Result<TranscriptionResult, VoiceError> {
        let chunks: Vec<&[f32]> = pcm_f32.chunks(self.chunk_samples).collect();
        let total_chunks = chunks.len();
        let mut full_text = String::new();
        let mut last_timestamp = 0u64;

        for (i, chunk) in chunks.iter().enumerate() {
            let is_last = i == total_chunks - 1;
            let wav_data =
                format::pcm_f32_to_wav(chunk, WHISPER_SAMPLE_RATE, MONO_CHANNELS)?;

            let mut result = self
                .recognizer
                .transcribe(&wav_data, &self.language)
                .await?;

            result.is_final = is_last;
            last_timestamp = result.timestamp;

            if !full_text.is_empty() && !result.text.is_empty() {
                full_text.push(' ');
            }
            full_text.push_str(&result.text);

            if let Some(callback) = on_interim {
                callback(&result);
            }
        }

        Ok(TranscriptionResult {
            text: full_text,
            confidence: 1.0,
            is_final: true,
            timestamp: last_timestamp,
        })
    }

    /// 生バイト列（PCM i16 LE）を直接文字起こしする
    ///
    /// フロントエンドから受け取った `Vec<u8>` を処理するヘルパー。
    pub async fn transcribe_raw_bytes(
        &self,
        raw_bytes: &[u8],
    ) -> Result<TranscriptionResult, VoiceError> {
        let wav_data =
            format::pcm_bytes_to_wav(raw_bytes, WHISPER_SAMPLE_RATE, MONO_CHANNELS)?;
        self.recognizer
            .transcribe(&wav_data, &self.language)
            .await
    }
}
