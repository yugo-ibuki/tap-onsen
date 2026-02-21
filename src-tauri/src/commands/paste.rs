//! クリップボード経由で前面アプリにテキストをペーストするコマンド
//!
//! PTT（Push-to-Talk）完了後に、音声認識＋AI処理の結果を
//! ユーザーが作業中のアプリ（エディタ等）に直接挿入するために使う。

// crate::error::AppError — このプロジェクト共通のエラー型。
// Tauri コマンドから返すエラーは全てこの型に統一されている。
use crate::error::AppError;

// c_void — C言語の `void*` に相当する型。
// macOS のネイティブ API（Core Graphics 等）は C 言語ベースなので、
// Rust から呼び出すときにポインタ型として使う。
use std::ffi::c_void;

// ============================================================
// macOS Core Graphics の FFI（Foreign Function Interface）宣言
// ============================================================
//
// FFI とは：Rust から C/Objective-C のネイティブ関数を呼び出す仕組み。
// `extern "C"` ブロックで関数シグネチャを宣言し、
// `#[link]` でリンクするフレームワーク名を指定する。
//
// ここでは「キーボードイベントを作って OS に送る」ための
// Core Graphics API を宣言している。

/// CGEvent のポインタ型（C 側では `CGEventRef`）。
/// `*mut c_void` = 「何かしらのデータを指すミュータブルな生ポインタ」。
/// Rust の安全性保証の外（unsafe）で扱う必要がある。
type CGEventRef = *mut c_void;

/// ⌘（Command）キーが押されていることを示すフラグ。
/// macOS の仮想キーイベントに「⌘を押しながら」という修飾子を付けるために使う。
/// 値は Apple の公式ドキュメント（CGEventFlags）で定義されている。
const CG_EVENT_FLAG_MASK_COMMAND: u64 = 0x00100000;

/// macOS の仮想キーコードで「V」キーを表す定数。
/// キーコード一覧: https://developer.apple.com/documentation/coregraphics/cgevent
/// 「⌘ + V」＝ペーストを再現するためにこの値を使う。
const KEYCODE_V: u16 = 9;

// CoreGraphics.framework の関数を Rust から呼べるように宣言する。
// `#[link(name = "CoreGraphics", kind = "framework")]` は
// 「CoreGraphics フレームワークとリンクしてね」とコンパイラに伝える。
#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    /// キーボードイベント（キーの押下/離上）を新規作成する。
    /// - source: イベントソース（今回は null = デフォルト）
    /// - virtual_key: どのキーか（例: 9 = V）
    /// - key_down: true=押下, false=離上
    /// - 戻り値: 作成されたイベントのポインタ（失敗時は null）
    fn CGEventCreateKeyboardEvent(
        source: *const c_void,
        virtual_key: u16,
        key_down: bool,
    ) -> CGEventRef;

    /// イベントに修飾キー（⌘, ⇧, ⌥ 等）のフラグをセットする。
    /// 例: CG_EVENT_FLAG_MASK_COMMAND をセットすると「⌘を押しながら」になる。
    fn CGEventSetFlags(event: CGEventRef, flags: u64);

    /// イベントを OS に送信（ポスト）する。
    /// - tap: どのレベルで送信するか
    ///   - 0 = kCGHIDEventTap（ハードウェア入力デバイスレベル、最も低層）
    ///   - これにより、どのアプリが前面でも確実にイベントが届く
    /// - event: 送信するイベント
    fn CGEventPost(tap: u32, event: CGEventRef);
}

// CoreFoundation.framework の関数。
// Core Graphics が返すオブジェクトは CoreFoundation の参照カウント管理下にあるため、
// 使い終わったら CFRelease で解放する必要がある（C の手動メモリ管理）。
#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    /// CoreFoundation オブジェクトの参照カウントを1減らす。
    /// カウントが0になるとメモリが解放される。
    /// Rust の所有権システムとは別の、C 側のメモリ管理。
    fn CFRelease(cf: *const c_void);
}

/// 前面アプリのカーソル位置にテキストをペーストする Tauri コマンド
///
/// ## 処理フロー
/// 1. 現在のクリップボード内容を退避（ユーザーの既存コピー内容を壊さないため）
/// 2. クリップボードに指定テキストをセット
/// 3. ⌘V キーイベントをシミュレーション（前面アプリがペーストを実行）
/// 4. 100ms 待機（前面アプリがクリップボードを読み取る時間を確保）
/// 5. クリップボードを元の内容に復元
///
/// ## 属性の説明
/// - `#[tauri::command]`: この関数を Tauri の IPC コマンドとして登録する。
///   フロントエンド（TypeScript）から `invoke("paste_to_foreground", ...)` で呼べる。
/// - `pub async fn`: `async` は非同期関数。`tokio::time::sleep` で待機するために必要。
///   Tauri は非同期コマンドを自動的にバックグラウンドスレッドで実行する。
/// - `Result<(), AppError>`: 成功時は空（`()`）、失敗時は `AppError` を返す。
///   `?` 演算子でエラーを早期リターンできる。
#[tauri::command]
pub async fn paste_to_foreground(text: String) -> Result<(), AppError> {
    // ── 1. クリップボード退避 ──
    // `arboard::Clipboard::new()` でシステムクリップボードへのハンドルを取得。
    // `map_err(...)` は Result のエラー型を AppError に変換する。
    // `?` はエラーだった場合に即座に関数から return する演算子。
    let mut clipboard =
        arboard::Clipboard::new().map_err(|e| AppError::Ai(format!("Clipboard error: {e}")))?;

    // 現在のクリップボード内容をテキストとして取得。
    // `.ok()` は Result<T, E> を Option<T> に変換する（エラーなら None）。
    // クリップボードに画像等が入っている場合は get_text() が失敗するので、
    // その場合は None として扱い、復元時に clear する。
    let saved = clipboard.get_text().ok();

    // ── 2. ペーストしたいテキストをクリップボードにセット ──
    clipboard
        .set_text(&text)
        .map_err(|e| AppError::Ai(format!("Clipboard set error: {e}")))?;

    // ── 3. ⌘V キーイベントをシミュレーション ──
    // `unsafe` ブロック: Rust のメモリ安全性をコンパイラが保証できない操作。
    // C 言語の関数呼び出しや生ポインタの操作はすべて unsafe が必要。
    // 「危険」ではなく「プログラマが安全性を保証する責任がある」という意味。
    unsafe {
        // V キーの「押す」イベントと「離す」イベントを作成。
        // std::ptr::null() = C の NULL ポインタ。source に null を渡すとデフォルトソースを使う。
        let key_down = CGEventCreateKeyboardEvent(std::ptr::null(), KEYCODE_V, true);
        let key_up = CGEventCreateKeyboardEvent(std::ptr::null(), KEYCODE_V, false);

        // イベント作成に失敗した場合（null が返った場合）はエラーを返す。
        // .is_null() は生ポインタが null かどうかをチェックする。
        if key_down.is_null() || key_up.is_null() {
            return Err(AppError::Ai(
                "Failed to create keyboard event".to_string(),
            ));
        }

        // 両方のイベントに「⌘キーを押しながら」フラグをセット。
        // これで「V」→「⌘+V」になる。
        CGEventSetFlags(key_down, CG_EVENT_FLAG_MASK_COMMAND);
        CGEventSetFlags(key_up, CG_EVENT_FLAG_MASK_COMMAND);

        // OS にイベントを送信。tap=0 は HID（ハードウェア）レベル。
        // key_down → key_up の順番が実際のキー操作と同じ。
        CGEventPost(0, key_down);
        CGEventPost(0, key_up);

        // 使い終わったイベントオブジェクトを解放（C のメモリ管理）。
        // `as *const c_void` は型キャスト。CFRelease は const ポインタを受け取る。
        CFRelease(key_down as *const c_void);
        CFRelease(key_up as *const c_void);
    }

    // ── 4. ペースト完了待ち ──
    // 前面アプリが ⌘V を受け取ってクリップボードからテキストを読み取るまでの
    // 時間を確保する。`.await` は非同期処理の完了を待つ Rust の構文。
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // ── 5. クリップボード復元 ──
    // `match` は Rust のパターンマッチ（switch 文の強化版）。
    // Option<String> の Some/None で分岐する。
    match saved {
        // 元のテキストがあった場合 → 復元
        Some(original) => {
            // `let _ =` は「戻り値を意図的に無視する」イディオム。
            // 復元に失敗しても致命的ではないので、エラーを握りつぶす。
            let _ = clipboard.set_text(&original);
        }
        // 元のテキストがなかった場合（画像等）→ クリップボードをクリア
        None => {
            let _ = clipboard.clear();
        }
    }

    // 成功を示す `Ok(())` を返す。`()` は「値なし」を表す Unit 型。
    Ok(())
}
