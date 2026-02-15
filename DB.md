# DB.md

tap-onsen の SQLite 履歴保存機能に関するドキュメント。

## 概要

AI処理完了後のテキストを SQLite に自動保存する。rusqlite (bundled) を使用し、Tauri の State 管理パターンに準拠。

## DBファイル

```
~/Library/Application Support/com.yugo-ibuki.voice-input-app/tap-onsen.db
```

Tauri の `app.path().app_data_dir()` で解決される。WAL モードで動作。

## スキーマ

### entries テーブル

| カラム | 型 | 制約 | 説明 |
|--------|------|------|------|
| id | INTEGER | PRIMARY KEY AUTOINCREMENT | 一意ID |
| raw_text | TEXT | NOT NULL | Whisper文字起こし結果（AI処理前） |
| processed_text | TEXT | NOT NULL | AI処理後のテキスト（AI無効時は raw_text と同一） |
| mode_id | TEXT | NOT NULL | 使用モード（proofread, summary 等） |
| model | TEXT | NOT NULL | 使用AIモデル（gpt-4o-mini, claude-haiku, none 等） |
| prompt_tokens | INTEGER | nullable | プロンプトトークン数（AI無効時は NULL） |
| completion_tokens | INTEGER | nullable | 補完トークン数 |
| total_tokens | INTEGER | nullable | 合計トークン数 |
| created_at | TEXT | NOT NULL, DEFAULT | ISO 8601 形式（UTC） |

### インデックス

```sql
CREATE INDEX idx_entries_created_at ON entries(created_at);
CREATE INDEX idx_entries_mode_id    ON entries(mode_id);
```

## マイグレーション

`PRAGMA user_version` で管理。現在のバージョンは **1**。

`DbState::new()` 呼び出し時に `schema::migrate()` が実行され、`user_version` を確認して未適用のマイグレーションを順次適用する。

将来カラム追加やテーブル追加が必要な場合は `schema.rs` の `migrate()` に `if version < N` ブロックを追加する。

## アーキテクチャ

```
src-tauri/src/db/
├── mod.rs          DbState { conn: Mutex<Connection> } — Tauri State
├── schema.rs       マイグレーション管理（PRAGMA user_version）
└── repository.rs   CRUD 関数 + Entry / NewEntry 構造体
```

### DbState

```rust
pub struct DbState {
    pub conn: Mutex<Connection>,
}
```

`AudioState` と同じ `Mutex` パターン。シングルユーザーのデスクトップアプリなのでコネクションプールは不要。

### Tauri コマンド

`src-tauri/src/commands/db.rs` に定義。

| コマンド | 引数 | 戻り値 | 説明 |
|----------|------|--------|------|
| `save_entry` | `NewEntry` | `i64` | エントリを保存し、挿入IDを返す |
| `get_entries` | `limit: u32, offset: u32` | `Vec<Entry>` | 新しい順で一覧取得 |
| `get_entry` | `id: i64` | `Option<Entry>` | ID指定で1件取得 |
| `delete_entry` | `id: i64` | `bool` | 削除。成否を返す |

### フロントエンド IPC

`src/lib/ipc.ts` に対応する関数を定義。

```typescript
saveEntry(entry: NewEntry): Promise<number>
getEntries(limit: number, offset: number): Promise<Entry[]>
getEntry(id: number): Promise<Entry | null>
deleteEntry(id: number): Promise<boolean>
```

型定義は `src/types/db.ts`。

## 自動保存の仕組み

`src/hooks/useAIProcess.ts` の `process()` 内で、AI処理完了後に `saveEntry()` を fire-and-forget で呼び出す。

- **AI有効モード**: `raw_text` = 文字起こし結果、`processed_text` = AI処理結果、トークン情報あり
- **AI無効モード**: `raw_text` = `processed_text` = 文字起こし結果、`model` = "none"、トークン情報は NULL

保存失敗時も AI 処理結果の表示は継続される（`.catch()` で warn ログのみ）。

## 確認方法

```bash
sqlite3 ~/Library/Application\ Support/com.yugo-ibuki.voice-input-app/tap-onsen.db "SELECT * FROM entries;"
```

## テスト

```bash
cd src-tauri && cargo test db
```

インメモリDBを使った9件のユニットテスト（insert, get, list, delete, edge cases）。
