//! macOS ネイティブ音声認識 (SFSpeechRecognizer) 実装
//!
//! Apple Speech Framework を使用してオフライン/オンラインの音声認識を行う。
//! WAV バイト列を一時ファイルに書き出し、SFSpeechURLRecognitionRequest で認識する。

use async_trait::async_trait;
use block2::RcBlock;
use objc2::AnyThread;
use objc2_foundation::{NSLocale, NSString, NSURL};
use objc2_speech::{
    SFSpeechRecognitionResult, SFSpeechRecognizer as NativeSpeechRecognizer,
    SFSpeechRecognizerAuthorizationStatus, SFSpeechURLRecognitionRequest,
};

use crate::voice::{SpeechRecognizer, TranscriptionResult, VoiceError};

/// macOS Speech Framework による音声認識エンジン
pub struct MacOSSpeechRecognizer {
    _language: String,
}

impl MacOSSpeechRecognizer {
    /// 指定された言語で認識エンジンを作成する
    ///
    /// # Arguments
    /// * `language` - BCP 47 言語コード (例: "ja-JP", "en-US")
    pub fn new(language: &str) -> Result<Self, VoiceError> {
        Ok(Self {
            _language: language.to_string(),
        })
    }

    /// 音声認識の権限を確認・リクエストする
    fn ensure_authorized() -> Result<(), VoiceError> {
        unsafe {
            let status = NativeSpeechRecognizer::authorizationStatus();

            if status == SFSpeechRecognizerAuthorizationStatus::Authorized {
                return Ok(());
            }
            if status == SFSpeechRecognizerAuthorizationStatus::Denied
                || status == SFSpeechRecognizerAuthorizationStatus::Restricted
            {
                return Err(VoiceError::PermissionDenied);
            }

            // NotDetermined: 権限リクエストを実行
            let (tx, rx) = std::sync::mpsc::channel();
            let block =
                RcBlock::new(move |s: SFSpeechRecognizerAuthorizationStatus| {
                    let _ = tx.send(s);
                });
            NativeSpeechRecognizer::requestAuthorization(&block);

            let result_status = rx
                .recv_timeout(std::time::Duration::from_secs(60))
                .map_err(|_| {
                    VoiceError::NativeError("Authorization request timed out".into())
                })?;

            if result_status == SFSpeechRecognizerAuthorizationStatus::Authorized {
                Ok(())
            } else {
                Err(VoiceError::PermissionDenied)
            }
        }
    }
}

#[async_trait]
impl SpeechRecognizer for MacOSSpeechRecognizer {
    async fn transcribe(
        &self,
        audio_data: &[u8],
        language: &str,
    ) -> Result<TranscriptionResult, VoiceError> {
        // WAV データを一時ファイルに書き出す
        let temp_path = std::env::temp_dir().join(format!(
            "tap_onsen_{}.wav",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        std::fs::write(&temp_path, audio_data)
            .map_err(|e| VoiceError::NativeError(format!("Failed to write temp file: {}", e)))?;

        let (tx, rx) = tokio::sync::oneshot::channel::<Result<String, VoiceError>>();
        let temp_path_clone = temp_path.clone();
        let language = language.to_string();

        // ObjC API はバックグラウンドスレッドで実行
        // コールバックは recognizer の queue（デフォルトでメインキュー）で呼ばれる
        std::thread::spawn(move || {
            if let Err(e) = MacOSSpeechRecognizer::ensure_authorized() {
                let _ = tx.send(Err(e));
                return;
            }

            unsafe {
                let locale_str = NSString::from_str(&language);
                let locale =
                    NSLocale::initWithLocaleIdentifier(NSLocale::alloc(), &locale_str);

                let recognizer = match NativeSpeechRecognizer::initWithLocale(
                    NativeSpeechRecognizer::alloc(),
                    &locale,
                ) {
                    Some(r) => r,
                    None => {
                        let _ = tx.send(Err(VoiceError::NativeError(format!(
                            "Failed to create recognizer for locale: {}",
                            language
                        ))));
                        return;
                    }
                };

                if !recognizer.isAvailable() {
                    let _ = tx.send(Err(VoiceError::NativeError(
                        "Speech recognizer is not available".into(),
                    )));
                    return;
                }

                // ファイル URL を作成
                let path_str =
                    NSString::from_str(temp_path_clone.to_str().unwrap_or(""));
                let url = NSURL::fileURLWithPath(&path_str);

                // 認識リクエストを作成
                let request = SFSpeechURLRecognitionRequest::initWithURL(
                    SFSpeechURLRecognitionRequest::alloc(),
                    &url,
                );

                // オンデバイス認識を優先（オフラインモデルが利用可能な場合）
                if recognizer.supportsOnDeviceRecognition() {
                    request.setRequiresOnDeviceRecognition(true);
                }

                // 結果受信用の std チャンネル（ブロック内から送信）
                let (result_tx, result_rx) =
                    std::sync::mpsc::channel::<Result<String, String>>();

                // ObjC コールバックブロック
                // 部分結果が複数回呼ばれ、isFinal で最終結果を取得する
                let handler = RcBlock::new(
                    move |result_ptr: *mut SFSpeechRecognitionResult,
                          error_ptr: *mut objc2_foundation::NSError| {
                        if !error_ptr.is_null() {
                            let _ = result_tx
                                .send(Err("Speech recognition error".to_string()));
                            return;
                        }
                        if let Some(result) = result_ptr.as_ref() {
                            if result.isFinal() {
                                let transcription = result.bestTranscription();
                                let text = transcription.formattedString().to_string();
                                let _ = result_tx.send(Ok(text));
                            }
                        }
                    },
                );

                // 認識タスクを開始（_task を保持してタスクが生存するようにする）
                let _task = recognizer
                    .recognitionTaskWithRequest_resultHandler(&request, &handler);

                // 最終結果を待機（60秒タイムアウト: SFSpeechRecognitionTask の制限に合わせる）
                match result_rx.recv_timeout(std::time::Duration::from_secs(60)) {
                    Ok(Ok(text)) => {
                        let _ = tx.send(Ok(text));
                    }
                    Ok(Err(e)) => {
                        let _ = tx.send(Err(VoiceError::NativeError(e)));
                    }
                    Err(_) => {
                        let _ = tx.send(Err(VoiceError::NativeError(
                            "Speech recognition timed out".into(),
                        )));
                    }
                }
            }
        });

        // tokio の非同期コンテキストで結果を待機
        let text = rx
            .await
            .map_err(|_| VoiceError::NativeError("Recognition channel closed".into()))??;

        // 一時ファイルを削除
        let _ = std::fs::remove_file(&temp_path);

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Ok(TranscriptionResult {
            text,
            confidence: 1.0,
            is_final: true,
            timestamp,
        })
    }
}
