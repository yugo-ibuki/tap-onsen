use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::error::AppError;
use crate::voice::format::pcm_bytes_to_wav;
use crate::voice::whisper_api::WhisperApiClient;
use crate::voice::SpeechRecognizer;

#[derive(Debug, Serialize, Deserialize)]
pub struct TranscriptionResult {
    pub text: String,
    pub confidence: f64,
    pub is_final: bool,
    pub timestamp: u64,
}

impl From<crate::voice::TranscriptionResult> for TranscriptionResult {
    fn from(r: crate::voice::TranscriptionResult) -> Self {
        Self {
            text: r.text,
            confidence: r.confidence,
            is_final: r.is_final,
            timestamp: r.timestamp,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct RecordingResult {
    pub audio_data: Vec<u8>,
    pub sample_rate: u32,
    pub channels: u16,
    pub duration_ms: u64,
}

/// 録音状態を管理する Tauri State
pub struct AudioState {
    inner: Mutex<AudioInner>,
}

struct AudioInner {
    is_recording: bool,
    buffer: Arc<Mutex<Vec<f32>>>,
    stop_tx: Option<mpsc::Sender<()>>,
    sample_rate: u32,
    channels: u16,
}

impl AudioState {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(AudioInner {
                is_recording: false,
                buffer: Arc::new(Mutex::new(Vec::new())),
                stop_tx: None,
                sample_rate: 0,
                channels: 0,
            }),
        }
    }
}

/// 音声データを文字起こしする
///
/// フロントエンドから PCM i16 LE のバイト列とサンプルレート・チャンネル数を受け取り、
/// WAV に変換後 Whisper API で日本語の文字起こしを行って結果を返す。
#[tauri::command]
pub async fn transcribe_audio(
    audio_data: Vec<u8>,
    sample_rate: u32,
    channels: u16,
) -> Result<TranscriptionResult, AppError> {
    let client =
        WhisperApiClient::from_env().map_err(|e| AppError::Audio(e.to_string()))?;
    let wav_data = pcm_bytes_to_wav(&audio_data, sample_rate, channels)
        .map_err(|e| AppError::Audio(e.to_string()))?;
    let result = client
        .transcribe(&wav_data, "ja")
        .await
        .map_err(|e| AppError::Audio(e.to_string()))?;
    Ok(result.into())
}

/// マイクからの録音を開始する
///
/// cpal でデフォルト入力デバイスを取得し、専用スレッドで音声データを
/// バッファに蓄積する。録音スレッドとの同期は mpsc チャンネルで行う。
#[tauri::command]
pub fn start_recording(state: State<'_, AudioState>) -> Result<(), AppError> {
    let mut inner = state
        .inner
        .lock()
        .map_err(|_| AppError::Audio("State lock poisoned".into()))?;

    if inner.is_recording {
        return Err(AppError::Audio("Already recording".into()));
    }

    // デフォルト入力デバイスと設定を取得
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or_else(|| AppError::Audio("No input device available".into()))?;
    let supported_config = device
        .default_input_config()
        .map_err(|e| AppError::Audio(format!("Failed to get input config: {}", e)))?;

    let sample_rate = supported_config.sample_rate().0;
    let channels = supported_config.channels();
    let sample_format = supported_config.sample_format();
    let stream_config: cpal::StreamConfig = supported_config.into();

    let buffer = Arc::new(Mutex::new(Vec::<f32>::new()));
    let buffer_for_thread = Arc::clone(&buffer);
    let (stop_tx, stop_rx) = mpsc::channel::<()>();
    let (ready_tx, ready_rx) = mpsc::sync_channel::<Result<(), String>>(1);

    // 録音スレッド: cpal::Stream を保持し、stop シグナルで終了
    thread::spawn(move || {
        let build_result = match sample_format {
            cpal::SampleFormat::F32 => {
                let buf = Arc::clone(&buffer_for_thread);
                device.build_input_stream(
                    &stream_config,
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        if let Ok(mut b) = buf.lock() {
                            b.extend_from_slice(data);
                        }
                    },
                    |err| eprintln!("Audio stream error: {}", err),
                    None,
                )
            }
            cpal::SampleFormat::I16 => {
                let buf = Arc::clone(&buffer_for_thread);
                device.build_input_stream(
                    &stream_config,
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        if let Ok(mut b) = buf.lock() {
                            b.extend(data.iter().map(|&s| s as f32 / 32768.0));
                        }
                    },
                    |err| eprintln!("Audio stream error: {}", err),
                    None,
                )
            }
            _ => {
                let _ = ready_tx.send(Err(format!(
                    "Unsupported sample format: {:?}",
                    sample_format
                )));
                return;
            }
        };

        match build_result {
            Ok(stream) => match stream.play() {
                Ok(()) => {
                    let _ = ready_tx.send(Ok(()));
                    let _ = stop_rx.recv();
                }
                Err(e) => {
                    let _ = ready_tx.send(Err(format!("Failed to start stream: {}", e)));
                }
            },
            Err(e) => {
                let _ = ready_tx.send(Err(format!("Failed to build stream: {}", e)));
            }
        }
    });

    // 録音スレッドの準備完了を待機（タイムアウト5秒）
    match ready_rx.recv_timeout(Duration::from_secs(5)) {
        Ok(Ok(())) => {
            inner.buffer = buffer;
            inner.stop_tx = Some(stop_tx);
            inner.sample_rate = sample_rate;
            inner.channels = channels;
            inner.is_recording = true;
            Ok(())
        }
        Ok(Err(e)) => Err(AppError::Audio(e)),
        Err(_) => Err(AppError::Audio("Recording thread timed out".into())),
    }
}

/// 録音を停止して音声データを返す
///
/// 録音スレッドに停止シグナルを送り、バッファの f32 サンプルを
/// i16 PCM (little-endian) バイト列に変換して返す。
#[tauri::command]
pub fn stop_recording(state: State<'_, AudioState>) -> Result<RecordingResult, AppError> {
    let mut inner = state
        .inner
        .lock()
        .map_err(|_| AppError::Audio("State lock poisoned".into()))?;

    if !inner.is_recording {
        return Err(AppError::Audio("Not recording".into()));
    }

    if let Some(tx) = inner.stop_tx.take() {
        let _ = tx.send(());
    }
    inner.is_recording = false;

    // ストリーム終了の猶予
    thread::sleep(Duration::from_millis(100));

    let samples = {
        let mut buf = inner
            .buffer
            .lock()
            .map_err(|_| AppError::Audio("Buffer lock poisoned".into()))?;
        std::mem::take(&mut *buf)
    };

    let sample_rate = inner.sample_rate;
    let channels = inner.channels;

    // f32 → i16 PCM little-endian
    let audio_data: Vec<u8> = samples
        .iter()
        .map(|&s| (s.clamp(-1.0, 1.0) * 32767.0) as i16)
        .flat_map(|s| s.to_le_bytes())
        .collect();

    let duration_ms = if sample_rate > 0 && channels > 0 {
        (samples.len() as u64 * 1000) / (sample_rate as u64 * channels as u64)
    } else {
        0
    };

    Ok(RecordingResult {
        audio_data,
        sample_rate,
        channels,
        duration_ms,
    })
}
