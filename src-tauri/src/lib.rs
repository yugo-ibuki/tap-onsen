pub mod ai;
pub mod commands;
pub mod config;
pub mod db;
pub mod error;
#[cfg(target_os = "macos")]
pub mod hotkey;
pub mod voice;

use tauri::Manager;

use commands::audio::AudioState;
use db::DbState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // .env ファイルから環境変数を読み込む（なくてもエラーにしない）
    let _ = dotenvy::dotenv();
    tauri::Builder::default()
        .manage(AudioState::new())
        .setup(|app| {
            // SQLite DB を Application Support ディレクトリに初期化
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data directory");
            let db_path = app_data_dir.join("tap-onsen.db");
            let db_state =
                DbState::new(&db_path).expect("failed to initialize database");
            app.manage(db_state);

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
            commands::db::save_entry,
            commands::db::get_entries,
            commands::db::get_entry,
            commands::db::delete_entry,
            commands::paste::paste_to_foreground,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
