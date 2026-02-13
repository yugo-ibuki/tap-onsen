use crate::voice::VoiceError;
use std::io::Cursor;

/// Whisper API が推奨するサンプリングレート
pub const WHISPER_SAMPLE_RATE: u32 = 16_000;
/// モノラルチャンネル
pub const MONO_CHANNELS: u16 = 1;
/// PCM 16-bit サンプルのビット数
pub const BITS_PER_SAMPLE: u16 = 16;

/// PCM f32 サンプルデータを WAV フォーマットのバイト列に変換する
///
/// Whisper API は WAV ファイル形式での音声入力を受け付けるため、
/// 生の PCM データにヘッダーを付与して正しい WAV 形式に変換する。
///
/// # Arguments
/// * `pcm_data` - f32 形式の PCM サンプルデータ（-1.0 〜 1.0）
/// * `sample_rate` - サンプリングレート（Hz）
/// * `channels` - チャンネル数（1 = モノラル, 2 = ステレオ）
pub fn pcm_f32_to_wav(
    pcm_data: &[f32],
    sample_rate: u32,
    channels: u16,
) -> Result<Vec<u8>, VoiceError> {
    let spec = hound::WavSpec {
        channels,
        sample_rate,
        bits_per_sample: BITS_PER_SAMPLE,
        sample_format: hound::SampleFormat::Int,
    };

    let mut buffer = Cursor::new(Vec::new());
    let mut writer = hound::WavWriter::new(&mut buffer, spec)
        .map_err(|e| VoiceError::FormatError(format!("Failed to create WAV writer: {}", e)))?;

    for &sample in pcm_data {
        let clamped = sample.clamp(-1.0, 1.0);
        let int_sample = (clamped * i16::MAX as f32) as i16;
        writer
            .write_sample(int_sample)
            .map_err(|e| VoiceError::FormatError(format!("Failed to write sample: {}", e)))?;
    }

    writer
        .finalize()
        .map_err(|e| VoiceError::FormatError(format!("Failed to finalize WAV: {}", e)))?;

    Ok(buffer.into_inner())
}

/// 生バイト列（PCM i16 リトルエンディアン）を WAV に変換する
///
/// フロントエンドから受け取った `Vec<u8>` を直接 WAV に変換するケースで使用。
/// バイト列は i16 リトルエンディアンのサンプルとして解釈される。
pub fn pcm_bytes_to_wav(
    raw_bytes: &[u8],
    sample_rate: u32,
    channels: u16,
) -> Result<Vec<u8>, VoiceError> {
    if raw_bytes.len() % 2 != 0 {
        return Err(VoiceError::FormatError(
            "PCM byte data length must be even (i16 samples)".to_string(),
        ));
    }

    let spec = hound::WavSpec {
        channels,
        sample_rate,
        bits_per_sample: BITS_PER_SAMPLE,
        sample_format: hound::SampleFormat::Int,
    };

    let mut buffer = Cursor::new(Vec::new());
    let mut writer = hound::WavWriter::new(&mut buffer, spec)
        .map_err(|e| VoiceError::FormatError(format!("Failed to create WAV writer: {}", e)))?;

    for chunk in raw_bytes.chunks_exact(2) {
        let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
        writer
            .write_sample(sample)
            .map_err(|e| VoiceError::FormatError(format!("Failed to write sample: {}", e)))?;
    }

    writer
        .finalize()
        .map_err(|e| VoiceError::FormatError(format!("Failed to finalize WAV: {}", e)))?;

    Ok(buffer.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pcm_f32_to_wav_produces_valid_wav() {
        // 1秒分の無音データ
        let samples = vec![0.0f32; WHISPER_SAMPLE_RATE as usize];
        let result = pcm_f32_to_wav(&samples, WHISPER_SAMPLE_RATE, MONO_CHANNELS);
        assert!(result.is_ok());

        let wav_data = result.unwrap();
        // WAV ヘッダーは "RIFF" で始まる
        assert_eq!(&wav_data[0..4], b"RIFF");
        // フォーマットは "WAVE"
        assert_eq!(&wav_data[8..12], b"WAVE");
    }

    #[test]
    fn test_pcm_f32_clamps_values() {
        let samples = vec![-2.0f32, 2.0, 0.5, -0.5];
        let result = pcm_f32_to_wav(&samples, WHISPER_SAMPLE_RATE, MONO_CHANNELS);
        assert!(result.is_ok());
    }

    #[test]
    fn test_pcm_bytes_to_wav_odd_length_error() {
        let odd_bytes = vec![0u8, 1, 2]; // 奇数長
        let result = pcm_bytes_to_wav(&odd_bytes, WHISPER_SAMPLE_RATE, MONO_CHANNELS);
        assert!(result.is_err());
    }

    #[test]
    fn test_pcm_bytes_to_wav_valid() {
        // 2サンプル分のバイトデータ (i16 LE)
        let bytes = vec![0x00, 0x00, 0xFF, 0x7F]; // 0, 32767
        let result = pcm_bytes_to_wav(&bytes, WHISPER_SAMPLE_RATE, MONO_CHANNELS);
        assert!(result.is_ok());
    }
}
