pub mod models;
pub mod database;
pub mod commands;

use std::sync::Mutex;
use commands::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db = database::init_db().expect("Failed to initialize database");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .manage(AppState { db: Mutex::new(db) })
        .invoke_handler(tauri::generate_handler![
            commands::add_library_source,
            commands::remove_library_source,
            commands::get_library_sources,
            commands::get_all_songs,
            commands::get_all_songs_basic,
            commands::get_song_by_id,
            commands::search_songs,
            commands::mark_song_unavailable,
            commands::create_playlist,
            commands::rename_playlist,
            commands::delete_playlist,
            commands::get_playlists,
            commands::add_song_to_playlist,
            commands::remove_song_from_playlist,
            commands::get_playlist_songs,
            commands::record_recent_play,
            commands::get_recent_plays,
            commands::set_setting,
            commands::get_setting,
            commands::save_playback_state_cmd,
            commands::load_playback_state_cmd,
            commands::cache_lyrics_cmd,
            commands::get_cached_lyrics_cmd,
            commands::get_lyrics,
            commands::scan_library,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
