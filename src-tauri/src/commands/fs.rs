use std::path::PathBuf;

use crate::error::AppError;

/// アプリ用の一時音声ファイルディレクトリを取得（なければ作成）
fn audio_temp_dir() -> Result<PathBuf, AppError> {
    let dir = std::env::temp_dir().join("tap-onsen").join("audio");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// 音声データを一時ファイルとして保存する
#[tauri::command]
pub fn save_audio_file(audio_data: Vec<u8>, filename: String) -> Result<String, AppError> {
    let dir = audio_temp_dir()?;
    let path = dir.join(&filename);
    std::fs::write(&path, &audio_data)?;
    Ok(path.to_string_lossy().to_string())
}

/// 指定した一時音声ファイルを削除する
#[tauri::command]
pub fn delete_audio_file(filename: String) -> Result<(), AppError> {
    let dir = audio_temp_dir()?;
    let path = dir.join(&filename);
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    Ok(())
}

/// すべての一時音声ファイルを削除し、削除件数を返す
#[tauri::command]
pub fn cleanup_audio_files() -> Result<u32, AppError> {
    let dir = audio_temp_dir()?;
    let mut count = 0u32;
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        if entry.path().is_file() {
            std::fs::remove_file(entry.path())?;
            count += 1;
        }
    }
    Ok(count)
}
