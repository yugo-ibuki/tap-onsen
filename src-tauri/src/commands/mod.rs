pub mod ai;
pub mod audio;
pub mod fs;

use crate::config::modes;
use crate::error::AppError;

#[tauri::command]
pub fn get_modes(app: tauri::AppHandle) -> Result<Vec<modes::ModeConfig>, AppError> {
    modes::load_modes_from_app(&app)
}
