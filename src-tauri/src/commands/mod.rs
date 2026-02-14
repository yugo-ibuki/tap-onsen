pub mod ai;
pub mod audio;
pub mod db;
pub mod fs;

use crate::config::modes;
use crate::error::AppError;

#[tauri::command]
pub fn get_modes(app: tauri::AppHandle) -> Result<Vec<modes::ModeConfig>, AppError> {
    modes::load_modes_from_app(&app)
}

/// Accessibility 権限の状態を返す（PTT機能に必要）
/// prompt=true でmacOSの許可ダイアログを表示する
#[cfg(target_os = "macos")]
#[tauri::command]
pub fn check_accessibility_permission(prompt: bool) -> bool {
    crate::hotkey::is_accessibility_trusted(prompt)
}
