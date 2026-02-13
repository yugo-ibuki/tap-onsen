# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

tap-onsen は macOS 向けの音声入力デスクトップアプリ。Tauri 2 (React + Rust) で構築。音声をWhisper APIでテキスト変換し、モードに応じてAI処理（校正・要約）を適用する。

## Development Commands

```bash
# フロントエンド開発サーバー（Vite単体、ブラウザ確認用）
pnpm dev

# Tauriウィンドウ付き開発（フロントエンド+Rustバックエンド同時起動）
pnpm tauri dev

# ビルド
pnpm build              # フロントエンドのみ (tsc + vite build)
pnpm tauri build        # macOSバイナリ生成

# Rust側
cd src-tauri
cargo check             # コンパイルチェック
cargo test              # テスト実行（prompt, format, context にユニットテスト有り）
cargo test prompt       # 特定テストモジュールのみ
cargo clippy            # Lint

# TypeScript
npx tsc --noEmit        # 型チェック
```

## Architecture

### データフロー

```
User → RecordButton → start_recording (cpal) → stop_recording → PCM i16 bytes
  → transcribe_audio → pcm_bytes_to_wav → Whisper API → TranscriptionResult
  → useAIProcess.process() → process_with_ai → render_prompt + AIProvider → TextArea表示
```

### フロントエンド ↔ Rust IPC

フロントエンドは `src/lib/ipc.ts` を唯一のTauriブリッジとして使う。直接 `invoke()` を呼ばない。IPC関数とRustコマンドは1:1対応:

| ipc.ts | Rust コマンド (lib.rs登録) |
|--------|--------------------------|
| getModes() | commands::get_modes |
| startRecording() | commands::audio::start_recording |
| stopRecording() | commands::audio::stop_recording |
| transcribeAudio() | commands::audio::transcribe_audio |
| processWithAI() | commands::ai::process_with_ai |

### 音声録音の仕組み (commands/audio.rs)

`AudioState` をTauri Stateとして管理。`start_recording` で cpal の入力ストリームを別スレッドで起動し、`mpsc` チャンネルで停止シグナルを送る設計。録音データは `Arc<Mutex<Vec<f32>>>` バッファに蓄積→停止時にi16 PCM LEバイト列に変換して返す。

### AI処理のプロバイダー抽象化

`ai::AIProvider` trait で OpenAI / Anthropic を統一的に扱う。`commands/ai.rs` の `process_with_ai` は環境変数 (`OPENAI_API_KEY` → `ANTHROPIC_API_KEY`) の存在順でプロバイダーを自動選択する。ストリーミング対応は `process_stream` + `tokio::sync::mpsc` で実装済み（現在コマンドからは非ストリーミング呼び出し）。

### 音声認識エンジンの抽象化

`voice::SpeechRecognizer` trait でバックエンドを切替可能に設計。現在は `WhisperApiClient` のみ実装。whisper.cpp やmacOS native は将来追加予定。

### モード設定の読み込み優先順位 (config/modes.rs)

1. Tauriリソースディレクトリ（本番ビルド）
2. `../config/modes.yaml`（開発時、CWD=src-tauri）
3. `include_str!` によるコンパイル時埋め込み（フォールバック）

### プロンプトテンプレート (ai/prompt.rs)

`{input}` と `{context}` プレースホルダーを展開。`{input}` がテンプレートに無い場合は末尾に自動追加。

### エラーハンドリング (error.rs)

`AppError` enum（Config / Audio / Ai / FileSystem / Io）を共通エラー型として使用。Tauri v2 では `Serialize` が必要なため、`Display` の文字列としてシリアライズする。

## Environment Variables

- `OPENAI_API_KEY` — Whisper音声認識 + GPT-4o-mini テキスト処理（必須、どちらか一方）
- `ANTHROPIC_API_KEY` — Claude Haiku テキスト処理（OpenAI未設定時のフォールバック）

## Key Conventions

- パッケージマネージャ: **pnpm**
- フロントエンドの型定義: `src/types/` に domain ごとに分離（mode.ts, voice.ts, ai.ts）
- Rust側の型はコマンド層(`commands/`)とドメイン層(`voice/`, `ai/`)で分離し、`From` trait で変換
- tsconfig: `strict: true`, `noUnusedLocals: true`, `noUnusedParameters: true`
- UI言語: 日本語（ラベル、説明文）

## Spec Reference

詳細仕様書: `/Users/yugo/ghq/github.com/yugo-ibuki/private-service-document/voice-input-app/spec.md`
