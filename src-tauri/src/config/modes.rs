use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::Manager;

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeConfig {
    pub id: String,
    pub label: String,
    pub description: String,
    pub ai_enabled: bool,
    pub ai_prompt: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ModesFile {
    modes: Vec<ModeConfig>,
}

const FALLBACK_MODES_YAML: &str = include_str!("../../../config/modes.yaml");

/// AppHandle なしで設定を読み込む（Team E 等の内部呼び出し用）
///
/// 開発時相対パス → include_str! フォールバック の順で読み込む。
pub fn load_modes() -> Result<Vec<ModeConfig>, String> {
    let dev_path = Path::new("../config/modes.yaml");
    if dev_path.exists() {
        let content = std::fs::read_to_string(dev_path)
            .map_err(|e| format!("Failed to read {}: {}", dev_path.display(), e))?;
        return parse_yaml_str(&content);
    }
    parse_yaml_str(FALLBACK_MODES_YAML)
}

/// AppHandle ありで設定を読み込む（Tauri コマンド用）
///
/// 読み込み優先順位:
/// 1. Tauri リソースディレクトリ（本番ビルド）
/// 2. ../config/modes.yaml（開発時、CWD = src-tauri）
/// 3. コンパイル時埋め込み（フォールバック）
pub fn load_modes_from_app(app: &tauri::AppHandle) -> Result<Vec<ModeConfig>, AppError> {
    // 1. リソースディレクトリから読み込み（本番環境）
    if let Ok(resource_dir) = app.path().resource_dir() {
        let yaml_path = resource_dir.join("config").join("modes.yaml");
        if yaml_path.exists() {
            return load_from_path(&yaml_path);
        }
    }

    // 2. 開発時の相対パスから読み込み
    let dev_path = Path::new("../config/modes.yaml");
    if dev_path.exists() {
        return load_from_path(dev_path);
    }

    // 3. コンパイル時埋め込みにフォールバック
    parse_yaml(FALLBACK_MODES_YAML)
}

fn load_from_path(path: &Path) -> Result<Vec<ModeConfig>, AppError> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| AppError::Config(format!("Failed to read {}: {}", path.display(), e)))?;
    parse_yaml(&content)
}

fn parse_yaml(content: &str) -> Result<Vec<ModeConfig>, AppError> {
    let modes_file: ModesFile = serde_yaml::from_str(content)
        .map_err(|e| AppError::Config(format!("Failed to parse modes.yaml: {}", e)))?;
    Ok(modes_file.modes)
}

fn parse_yaml_str(content: &str) -> Result<Vec<ModeConfig>, String> {
    let modes_file: ModesFile = serde_yaml::from_str(content)
        .map_err(|e| format!("Failed to parse modes.yaml: {}", e))?;
    Ok(modes_file.modes)
}
