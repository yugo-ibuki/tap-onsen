use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::error::AppError;

/// DBから取得したエントリ
#[derive(Debug, Serialize, Deserialize)]
pub struct Entry {
    pub id: i64,
    pub raw_text: String,
    pub processed_text: String,
    pub mode_id: String,
    pub model: String,
    pub prompt_tokens: Option<u32>,
    pub completion_tokens: Option<u32>,
    pub total_tokens: Option<u32>,
    pub created_at: String,
}

/// 新規保存用の入力データ
#[derive(Debug, Deserialize)]
pub struct NewEntry {
    pub raw_text: String,
    pub processed_text: String,
    pub mode_id: String,
    pub model: String,
    pub prompt_tokens: Option<u32>,
    pub completion_tokens: Option<u32>,
    pub total_tokens: Option<u32>,
}

/// エントリを保存し、挿入されたIDを返す
pub fn insert_entry(conn: &Connection, entry: &NewEntry) -> Result<i64, AppError> {
    conn.execute(
        "INSERT INTO entries (raw_text, processed_text, mode_id, model, prompt_tokens, completion_tokens, total_tokens)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            entry.raw_text,
            entry.processed_text,
            entry.mode_id,
            entry.model,
            entry.prompt_tokens,
            entry.completion_tokens,
            entry.total_tokens,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// エントリ一覧を取得（新しい順、limit/offset対応）
pub fn get_entries(conn: &Connection, limit: u32, offset: u32) -> Result<Vec<Entry>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, raw_text, processed_text, mode_id, model, prompt_tokens, completion_tokens, total_tokens, created_at
         FROM entries ORDER BY created_at DESC LIMIT ?1 OFFSET ?2",
    )?;

    let entries = stmt
        .query_map(params![limit, offset], |row| {
            Ok(Entry {
                id: row.get(0)?,
                raw_text: row.get(1)?,
                processed_text: row.get(2)?,
                mode_id: row.get(3)?,
                model: row.get(4)?,
                prompt_tokens: row.get(5)?,
                completion_tokens: row.get(6)?,
                total_tokens: row.get(7)?,
                created_at: row.get(8)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(entries)
}

/// IDでエントリを1件取得
pub fn get_entry(conn: &Connection, id: i64) -> Result<Option<Entry>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, raw_text, processed_text, mode_id, model, prompt_tokens, completion_tokens, total_tokens, created_at
         FROM entries WHERE id = ?1",
    )?;

    let entry = stmt
        .query_row(params![id], |row| {
            Ok(Entry {
                id: row.get(0)?,
                raw_text: row.get(1)?,
                processed_text: row.get(2)?,
                mode_id: row.get(3)?,
                model: row.get(4)?,
                prompt_tokens: row.get(5)?,
                completion_tokens: row.get(6)?,
                total_tokens: row.get(7)?,
                created_at: row.get(8)?,
            })
        })
        .optional()?;

    Ok(entry)
}

/// エントリを削除し、削除された行数を返す
pub fn delete_entry(conn: &Connection, id: i64) -> Result<bool, AppError> {
    let affected = conn.execute("DELETE FROM entries WHERE id = ?1", params![id])?;
    Ok(affected > 0)
}

/// 指定日数より古いエントリを削除し、削除件数を返す
pub fn delete_old_entries(conn: &Connection, days: u32) -> Result<usize, AppError> {
    let affected = conn.execute(
        "DELETE FROM entries WHERE created_at < strftime('%Y-%m-%dT%H:%M:%fZ', 'now', ?1)",
        params![format!("-{} days", days)],
    )?;
    Ok(affected)
}

/// rusqlite の optional() を使うためのトレイト
trait OptionalExt<T> {
    fn optional(self) -> Result<Option<T>, rusqlite::Error>;
}

impl<T> OptionalExt<T> for Result<T, rusqlite::Error> {
    fn optional(self) -> Result<Option<T>, rusqlite::Error> {
        match self {
            Ok(val) => Ok(Some(val)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        schema::migrate(&conn).unwrap();
        conn
    }

    fn sample_entry() -> NewEntry {
        NewEntry {
            raw_text: "こんにちは世界".to_string(),
            processed_text: "こんにちは、世界。".to_string(),
            mode_id: "proofread".to_string(),
            model: "gpt-4o-mini".to_string(),
            prompt_tokens: Some(10),
            completion_tokens: Some(15),
            total_tokens: Some(25),
        }
    }

    #[test]
    fn test_insert_and_get() {
        let conn = setup_db();
        let entry = sample_entry();

        let id = insert_entry(&conn, &entry).unwrap();
        assert!(id > 0);

        let fetched = get_entry(&conn, id).unwrap().expect("entry should exist");
        assert_eq!(fetched.raw_text, "こんにちは世界");
        assert_eq!(fetched.processed_text, "こんにちは、世界。");
        assert_eq!(fetched.mode_id, "proofread");
        assert_eq!(fetched.model, "gpt-4o-mini");
        assert_eq!(fetched.prompt_tokens, Some(10));
    }

    #[test]
    fn test_get_entries_ordering() {
        let conn = setup_db();

        for i in 0..5 {
            let entry = NewEntry {
                raw_text: format!("text {}", i),
                processed_text: format!("processed {}", i),
                mode_id: "proofread".to_string(),
                model: "gpt-4o-mini".to_string(),
                prompt_tokens: None,
                completion_tokens: None,
                total_tokens: None,
            };
            insert_entry(&conn, &entry).unwrap();
        }

        let entries = get_entries(&conn, 3, 0).unwrap();
        assert_eq!(entries.len(), 3);
        // 新しい順なのでid=5が最初
        assert!(entries[0].id > entries[1].id);
    }

    #[test]
    fn test_get_entries_offset() {
        let conn = setup_db();

        for i in 0..5 {
            let entry = NewEntry {
                raw_text: format!("text {}", i),
                processed_text: format!("processed {}", i),
                mode_id: "proofread".to_string(),
                model: "gpt-4o-mini".to_string(),
                prompt_tokens: None,
                completion_tokens: None,
                total_tokens: None,
            };
            insert_entry(&conn, &entry).unwrap();
        }

        let entries = get_entries(&conn, 10, 3).unwrap();
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_delete_entry() {
        let conn = setup_db();
        let id = insert_entry(&conn, &sample_entry()).unwrap();

        let deleted = delete_entry(&conn, id).unwrap();
        assert!(deleted);

        let fetched = get_entry(&conn, id).unwrap();
        assert!(fetched.is_none());
    }

    #[test]
    fn test_delete_nonexistent() {
        let conn = setup_db();
        let deleted = delete_entry(&conn, 99999).unwrap();
        assert!(!deleted);
    }

    #[test]
    fn test_get_nonexistent() {
        let conn = setup_db();
        let entry = get_entry(&conn, 99999).unwrap();
        assert!(entry.is_none());
    }

    #[test]
    fn test_delete_old_entries() {
        let conn = setup_db();

        // 古いエントリを手動で挿入（4日前）
        conn.execute(
            "INSERT INTO entries (raw_text, processed_text, mode_id, model, created_at)
             VALUES ('old', 'old', 'plain', 'none', strftime('%Y-%m-%dT%H:%M:%fZ', 'now', '-4 days'))",
            [],
        )
        .unwrap();

        // 新しいエントリを挿入（今）
        insert_entry(&conn, &sample_entry()).unwrap();

        let deleted = delete_old_entries(&conn, 3).unwrap();
        assert_eq!(deleted, 1);

        // 新しいエントリは残っている
        let entries = get_entries(&conn, 100, 0).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].raw_text, "こんにちは世界");
    }

    #[test]
    fn test_delete_old_entries_none_old() {
        let conn = setup_db();
        insert_entry(&conn, &sample_entry()).unwrap();

        let deleted = delete_old_entries(&conn, 3).unwrap();
        assert_eq!(deleted, 0);
    }

    #[test]
    fn test_entry_without_tokens() {
        let conn = setup_db();
        let entry = NewEntry {
            raw_text: "hello".to_string(),
            processed_text: "hello".to_string(),
            mode_id: "plain".to_string(),
            model: "none".to_string(),
            prompt_tokens: None,
            completion_tokens: None,
            total_tokens: None,
        };

        let id = insert_entry(&conn, &entry).unwrap();
        let fetched = get_entry(&conn, id).unwrap().unwrap();
        assert!(fetched.prompt_tokens.is_none());
        assert!(fetched.total_tokens.is_none());
    }
}
