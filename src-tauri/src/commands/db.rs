use tauri::State;

use crate::db::repository::{self, Entry, NewEntry};
use crate::db::DbState;
use crate::error::AppError;

/// エントリを保存し、挿入IDを返す
#[tauri::command]
pub fn save_entry(state: State<'_, DbState>, entry: NewEntry) -> Result<i64, AppError> {
    let conn = state.conn.lock().map_err(|e| AppError::Database(e.to_string()))?;
    repository::insert_entry(&conn, &entry)
}

/// エントリ一覧を取得（新しい順）
#[tauri::command]
pub fn get_entries(
    state: State<'_, DbState>,
    limit: u32,
    offset: u32,
) -> Result<Vec<Entry>, AppError> {
    let conn = state.conn.lock().map_err(|e| AppError::Database(e.to_string()))?;
    repository::get_entries(&conn, limit, offset)
}

/// IDでエントリを1件取得
#[tauri::command]
pub fn get_entry(state: State<'_, DbState>, id: i64) -> Result<Option<Entry>, AppError> {
    let conn = state.conn.lock().map_err(|e| AppError::Database(e.to_string()))?;
    repository::get_entry(&conn, id)
}

/// エントリを削除
#[tauri::command]
pub fn delete_entry(state: State<'_, DbState>, id: i64) -> Result<bool, AppError> {
    let conn = state.conn.lock().map_err(|e| AppError::Database(e.to_string()))?;
    repository::delete_entry(&conn, id)
}
