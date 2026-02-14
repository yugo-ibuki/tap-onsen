use rusqlite::Connection;

use crate::error::AppError;

const CURRENT_VERSION: u32 = 1;

/// スキーマバージョンを取得
fn get_user_version(conn: &Connection) -> Result<u32, AppError> {
    let version: u32 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    Ok(version)
}

/// スキーマバージョンを設定
fn set_user_version(conn: &Connection, version: u32) -> Result<(), AppError> {
    conn.pragma_update(None, "user_version", version)?;
    Ok(())
}

/// マイグレーションを実行してスキーマを最新にする
pub fn migrate(conn: &Connection) -> Result<(), AppError> {
    let version = get_user_version(conn)?;

    if version < 1 {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS entries (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                raw_text        TEXT NOT NULL,
                processed_text  TEXT NOT NULL,
                mode_id         TEXT NOT NULL,
                model           TEXT NOT NULL,
                prompt_tokens   INTEGER,
                completion_tokens INTEGER,
                total_tokens    INTEGER,
                created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
            );

            CREATE INDEX IF NOT EXISTS idx_entries_created_at ON entries(created_at);
            CREATE INDEX IF NOT EXISTS idx_entries_mode_id ON entries(mode_id);",
        )?;
        set_user_version(conn, 1)?;
    }

    debug_assert_eq!(get_user_version(conn)?, CURRENT_VERSION);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrate_creates_table() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();

        let count: u32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='entries'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_migrate_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        migrate(&conn).unwrap(); // 2回目もエラーにならない
        assert_eq!(get_user_version(&conn).unwrap(), CURRENT_VERSION);
    }
}
