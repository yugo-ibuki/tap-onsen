pub mod repository;
pub mod schema;

use std::path::Path;
use std::sync::Mutex;

use rusqlite::Connection;

use crate::error::AppError;

/// データベース接続を保持する Tauri State
///
/// AudioState と同じ Mutex パターンで排他制御する。
/// シングルユーザーのデスクトップアプリなのでコネクションプールは不要。
pub struct DbState {
    pub conn: Mutex<Connection>,
}

impl DbState {
    /// 指定パスにDBファイルを作成（または開く）し、WALモード有効化 + スキーマ初期化
    pub fn new(db_path: &Path) -> Result<Self, AppError> {
        // 親ディレクトリが無ければ作成
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(db_path)?;

        // WALモードで並行読み取り性能を向上
        conn.pragma_update(None, "journal_mode", "WAL")?;

        // スキーマの初期化 / マイグレーション
        schema::migrate(&conn)?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
}
