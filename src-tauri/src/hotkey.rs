//! Push-to-Talk: 右Optionキー長押しで録音を開始/停止する
//!
//! macOS の CGEventTap API を使い、keycode 61（右Option）の
//! flagsChanged イベントを監視する。Accessibility 権限が必要。

use core_foundation::base::TCFType;
use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop, CFRunLoopSource};
use std::ffi::c_void;
use std::ptr;
use tauri::{AppHandle, Emitter};

/// 右 Option キーの macOS keycode
const RIGHT_OPTION_KEYCODE: i64 = 61;

/// kCGKeyboardEventKeycode（CGEventField）
const CG_KEYBOARD_EVENT_KEYCODE: u32 = 9;

/// kCGEventFlagMaskAlternate — Option キーが押されている時のフラグ
const CG_EVENT_FLAG_MASK_ALTERNATE: u64 = 0x00080000;

/// CGEventType の定数（core-graphics の enum は PartialEq 未実装のため数値で扱う）
const CG_EVENT_FLAGS_CHANGED: u32 = 12;
const CG_EVENT_TAP_DISABLED_BY_TIMEOUT: u32 = 0xFFFFFFFE;
const CG_EVENT_TAP_DISABLED_BY_USER_INPUT: u32 = 0xFFFFFFFF;

// --- Core Graphics FFI ---
type CGEventRef = *mut c_void;
type CGEventTapProxy = *mut c_void;
type CFMachPortRef = *mut c_void;
type CFAllocatorRef = *const c_void;
type CGEventTapCallBack = unsafe extern "C" fn(
    proxy: CGEventTapProxy,
    event_type: u32,
    event: CGEventRef,
    user_info: *mut c_void,
) -> CGEventRef;

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGEventTapCreate(
        tap: u32,        // CGEventTapLocation
        place: u32,      // CGEventTapPlacement
        options: u32,    // CGEventTapOptions
        events_of_interest: u64,
        callback: CGEventTapCallBack,
        user_info: *mut c_void,
    ) -> CFMachPortRef;

    fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);

    fn CGEventGetIntegerValueField(event: CGEventRef, field: u32) -> i64;

    fn CGEventGetFlags(event: CGEventRef) -> u64;
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFMachPortCreateRunLoopSource(
        allocator: CFAllocatorRef,
        port: CFMachPortRef,
        order: i64,
    ) -> core_foundation::runloop::CFRunLoopSourceRef;
}

// --- Accessibility FFI ---
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrustedWithOptions(options: *const c_void) -> bool;
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFDictionaryCreate(
        allocator: CFAllocatorRef,
        keys: *const *const c_void,
        values: *const *const c_void,
        num_values: i64,
        key_callbacks: *const c_void,
        value_callbacks: *const c_void,
    ) -> *const c_void;

    static kCFTypeDictionaryKeyCallBacks: c_void;
    static kCFTypeDictionaryValueCallBacks: c_void;
}

/// CGEventTap のコールバック関数
///
/// flagsChanged イベントを受け取り、右 Option キーの押下/離上を判定する。
/// 押下時は "ptt-start"、離上時は "ptt-stop" イベントを Tauri に発火する。
unsafe extern "C" fn event_tap_callback(
    _proxy: CGEventTapProxy,
    event_type: u32,
    event: CGEventRef,
    user_info: *mut c_void,
) -> CGEventRef {
    // タップが無効化された場合はイベントをそのまま返す
    if event_type == CG_EVENT_TAP_DISABLED_BY_TIMEOUT
        || event_type == CG_EVENT_TAP_DISABLED_BY_USER_INPUT
    {
        return event;
    }

    // flagsChanged 以外のイベントはスルー
    if event_type != CG_EVENT_FLAGS_CHANGED {
        return event;
    }

    // keycode を取得
    let keycode = CGEventGetIntegerValueField(event, CG_KEYBOARD_EVENT_KEYCODE);

    if keycode != RIGHT_OPTION_KEYCODE {
        return event;
    }

    // フラグから Alternate（Option）キーの状態を判定
    let flags = CGEventGetFlags(event);
    let is_pressed = (flags & CG_EVENT_FLAG_MASK_ALTERNATE) != 0;

    // user_info から AppHandle を復元（所有権は移さない）
    let app_handle = &*(user_info as *const AppHandle);

    let event_name = if is_pressed { "ptt-start" } else { "ptt-stop" };

    let _ = app_handle.emit(event_name, ());

    event
}

/// Accessibility 権限をチェックする
///
/// 未許可の場合、`prompt` が true なら macOS の許可ダイアログを表示する。
pub fn is_accessibility_trusted(prompt: bool) -> bool {
    use core_foundation::boolean::CFBoolean;
    use core_foundation::string::CFString;

    unsafe {
        let key = CFString::new("AXTrustedCheckOptionPrompt");
        let value = if prompt {
            CFBoolean::true_value()
        } else {
            CFBoolean::false_value()
        };

        let keys = [key.as_concrete_TypeRef() as *const c_void];
        let values = [value.as_concrete_TypeRef() as *const c_void];

        let options = CFDictionaryCreate(
            ptr::null(),
            keys.as_ptr(),
            values.as_ptr(),
            1,
            &kCFTypeDictionaryKeyCallBacks as *const c_void,
            &kCFTypeDictionaryValueCallBacks as *const c_void,
        );

        AXIsProcessTrustedWithOptions(options)
    }
}

/// CGEventTap リスナーを専用スレッドで起動する
///
/// `app_handle` を使ってフロントエンドにイベントを送信する。
/// Accessibility 権限がない場合はログを出力して静かに失敗する。
pub fn start_listener(app_handle: AppHandle) {
    // prompt: true で未許可ならmacOSの許可ダイアログを表示
    if !is_accessibility_trusted(true) {
        eprintln!("[hotkey] Accessibility permission not granted. PTT will not work.");
    }

    std::thread::spawn(move || {
        unsafe {
            // AppHandle を生ポインタに変換（スレッドの生存期間中ずっと有効）
            let app_handle_ptr = Box::into_raw(Box::new(app_handle)) as *mut c_void;

            // flagsChanged (12) のみ監視
            let event_mask = 1u64 << CG_EVENT_FLAGS_CHANGED;

            // CGEventTapLocation::Session = 1（HID = 0, Session = 1, AnnotatedSession = 2）
            // CGEventTapPlacement::HeadInsertEventTap = 0
            // CGEventTapOptions::ListenOnly = 1（Default = 0, ListenOnly = 1）
            let tap = CGEventTapCreate(
                1, // Session
                0, // HeadInsertEventTap
                1, // ListenOnly
                event_mask,
                event_tap_callback,
                app_handle_ptr,
            );

            if tap.is_null() {
                eprintln!(
                    "[hotkey] Failed to create CGEventTap. Check Accessibility permissions."
                );
                return;
            }

            let source_ref = CFMachPortCreateRunLoopSource(ptr::null(), tap, 0);

            if source_ref.is_null() {
                eprintln!("[hotkey] Failed to create CFRunLoopSource.");
                return;
            }

            let source = CFRunLoopSource::wrap_under_create_rule(source_ref);
            let run_loop = CFRunLoop::get_current();
            run_loop.add_source(&source, kCFRunLoopCommonModes);

            // タップを有効化
            CGEventTapEnable(tap, true);

            // CFRunLoop を開始（このスレッドはここでブロックされる）
            CFRunLoop::run_current();
        }
    });
}
