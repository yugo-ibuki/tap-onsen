use crate::error::AppError;
use std::ffi::c_void;

// --- Core Graphics FFI (⌘V シミュレーション用) ---
type CGEventRef = *mut c_void;

/// kCGEventFlagMaskCommand — ⌘ キーフラグ
const CG_EVENT_FLAG_MASK_COMMAND: u64 = 0x00100000;

/// macOS 仮想キーコード: V = 9
const KEYCODE_V: u16 = 9;

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGEventCreateKeyboardEvent(
        source: *const c_void,
        virtual_key: u16,
        key_down: bool,
    ) -> CGEventRef;
    fn CGEventSetFlags(event: CGEventRef, flags: u64);
    fn CGEventPost(tap: u32, event: CGEventRef);
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFRelease(cf: *const c_void);
}

/// 前面アプリのカーソル位置にテキストをペーストする
///
/// 処理フロー:
/// 1. 現在のクリップボード内容を退避
/// 2. クリップボードに指定テキストをセット
/// 3. ⌘V キーイベントをシミュレーション
/// 4. ペースト完了を待機
/// 5. クリップボードを元の内容に復元
#[tauri::command]
pub async fn paste_to_foreground(text: String) -> Result<(), AppError> {
    // 1. クリップボード退避
    let mut clipboard =
        arboard::Clipboard::new().map_err(|e| AppError::Ai(format!("Clipboard error: {e}")))?;

    let saved = clipboard.get_text().ok();

    // 2. テキストをセット
    clipboard
        .set_text(&text)
        .map_err(|e| AppError::Ai(format!("Clipboard set error: {e}")))?;

    // 3. ⌘V シミュレーション
    unsafe {
        let key_down = CGEventCreateKeyboardEvent(std::ptr::null(), KEYCODE_V, true);
        let key_up = CGEventCreateKeyboardEvent(std::ptr::null(), KEYCODE_V, false);

        if key_down.is_null() || key_up.is_null() {
            return Err(AppError::Ai(
                "Failed to create keyboard event".to_string(),
            ));
        }

        CGEventSetFlags(key_down, CG_EVENT_FLAG_MASK_COMMAND);
        CGEventSetFlags(key_up, CG_EVENT_FLAG_MASK_COMMAND);

        // HID レベルでポスト (tap = 0: kCGHIDEventTap)
        CGEventPost(0, key_down);
        CGEventPost(0, key_up);

        CFRelease(key_down as *const c_void);
        CFRelease(key_up as *const c_void);
    }

    // 4. ペースト完了待ち
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // 5. クリップボード復元
    match saved {
        Some(original) => {
            let _ = clipboard.set_text(&original);
        }
        None => {
            let _ = clipboard.clear();
        }
    }

    Ok(())
}
