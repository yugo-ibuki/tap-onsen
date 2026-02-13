pub mod ai;
pub mod commands;
pub mod config;
pub mod error;
pub mod voice;

use commands::audio::AudioState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AudioState::new())
        .invoke_handler(tauri::generate_handler![
            commands::get_modes,
            commands::audio::transcribe_audio,
            commands::audio::start_recording,
            commands::audio::stop_recording,
            commands::ai::process_with_ai,
            commands::fs::save_audio_file,
            commands::fs::delete_audio_file,
            commands::fs::cleanup_audio_files,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
