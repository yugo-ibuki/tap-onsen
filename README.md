# tap-onsen

macOS 向け音声入力デスクトップアプリ。ボタンを押して話すだけで音声をテキスト化し、AI による校正・要約も可能。

## 機能

- **音声入力** — マイクで録音 → macOS ネイティブ音声認識でテキスト化（オフライン対応）
- **3つのモード** — そのまま入力 / 校正入力（AI） / 要約入力（AI）
- **AI処理** — OpenAI GPT-4o-mini または Claude Haiku で自動テキスト加工
- **クリップボードコピー** — 変換結果をワンクリックでコピー

## 必要なもの

- macOS 10.15 (Catalina) 以上
- [Node.js](https://nodejs.org/) v18+
- [pnpm](https://pnpm.io/)
- [Rust](https://rustup.rs/)
- Tauri 2 の[システム依存](https://v2.tauri.app/start/prerequisites/)（macOS: Xcode Command Line Tools）
- OpenAI API キー（AI処理に必要。音声認識はネイティブエンジンのため不要）

## セットアップ

```bash
# 依存インストール
pnpm install

# 環境変数（どちらか一方は必須）
export OPENAI_API_KEY="sk-..."        # Whisper + GPT-4o-mini
export ANTHROPIC_API_KEY="sk-ant-..." # Claude Haiku（AI処理のフォールバック）
```

## 起動

```bash
# Tauriウィンドウで起動（推奨）
pnpm tauri dev

# フロントエンドのみ（ブラウザで確認、Rust機能は使えない）
pnpm dev
```

## 使い方

1. アプリを起動する
2. **モードを選択** — 「そのまま入力」「校正入力」「要約入力」から選ぶ
3. **🎤 録音開始** ボタンを押して話す
4. **■ 停止** ボタンを押すと音声認識が実行される
5. モードに応じてAI処理が自動適用される
6. **コピー** ボタンで結果をクリップボードにコピー

### モード説明

| モード | 動作 |
|--------|------|
| そのまま入力 | 音声をそのままテキスト化（AI処理なし） |
| 校正入力 | 誤字脱字を修正し自然な日本語に校正 |
| 要約入力 | テキストを簡潔に要約 |

## ビルド

```bash
# macOS アプリバンドル生成
pnpm tauri build
```

`src-tauri/target/release/bundle/` にアプリが生成される。

## プロジェクト構成

```
src/                  # React フロントエンド (TypeScript)
├── components/       # UI コンポーネント
├── hooks/            # useVoiceInput, useAIProcess
├── lib/ipc.ts        # Tauri IPC ラッパー
└── types/            # 型定義

src-tauri/            # Rust バックエンド
├── src/commands/     # Tauri コマンド（IPC エンドポイント）
├── src/voice/        # 音声認識（Whisper API, PCM→WAV変換）
├── src/ai/           # AI処理（OpenAI, Anthropic, ストリーミング）
└── src/config/       # モード設定（YAML読み込み）

config/modes.yaml     # モード定義（カスタマイズ可能）
```

## カスタマイズ

`config/modes.yaml` を編集して独自モードを追加できる:

```yaml
modes:
  - id: "translate"
    label: "英訳入力"
    description: "音声テキストを英語に翻訳"
    ai_enabled: true
    ai_prompt: "以下の日本語テキストを自然な英語に翻訳してください"
```

## 技術スタック

- **フロントエンド**: React 19 + TypeScript + Vite
- **バックエンド**: Rust (Tauri 2)
- **音声認識**: OpenAI Whisper API
- **AI処理**: OpenAI GPT-4o-mini / Anthropic Claude Haiku
- **音声キャプチャ**: cpal (CoreAudio)

## ライセンス

Private
