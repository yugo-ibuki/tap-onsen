use serde::Serialize;
use thiserror::Error;

/// アプリケーション共通エラー型
///
/// Tauri v2 ではコマンドのエラー型に `Serialize` が必要。
/// `thiserror` でDisplay/Error を自動導出し、手動 Serialize で文字列化する。
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Audio error: {0}")]
    Audio(String),

    #[error("AI processing error: {0}")]
    Ai(String),

    #[error("File system error: {0}")]
    FileSystem(String),

    #[error("{0}")]
    Io(#[from] std::io::Error),
}

/// Tauri v2 のフロントエンドへのエラー伝搬用
/// エラーを文字列としてシリアライズする
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
