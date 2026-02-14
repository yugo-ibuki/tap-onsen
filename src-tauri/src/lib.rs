pub mod ai;
pub mod commands;
pub mod config;
pub mod error;
#[cfg(target_os = "macos")]
pub mod hotkey;
pub mod voice;

use commands::audio::AudioState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // .env ファイルから環境変数を読み込む（なくてもエラーにしない）
    let _ = dotenvy::dotenv();
    tauri::Builder::default()
        .manage(AudioState::new())
        .setup(|app| {
            // macOS: Push-to-Talk（右Optionキー長押し）リスナーを起動
            #[cfg(target_os = "macos")]
            hotkey::start_listener(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_modes,
            commands::audio::transcribe_audio,
            commands::audio::start_recording,
            commands::audio::stop_recording,
            commands::ai::process_with_ai,
            commands::fs::save_audio_file,
            commands::fs::delete_audio_file,
            commands::fs::cleanup_audio_files,
            commands::check_accessibility_permission,
            commands::paste::paste_to_foreground,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
