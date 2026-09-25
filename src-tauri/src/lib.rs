pub mod cache;
pub mod clients;
pub mod commands;
pub mod db;
pub mod discord_rpc;
pub mod downloads;
pub mod error;
pub mod models;
pub mod providers;
pub mod state;

use state::AppState;

pub fn run() {
    let pool = tauri::async_runtime::block_on(async {
        db::init_db()
            .await
            .expect("Failed to initialize SQLite database")
    });

    let app_state = AppState::new(pool);

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_deep_link::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            // AniList
            commands::fetch_trending,
            commands::fetch_popular,
            commands::fetch_top_rated,
            commands::fetch_home_data,
            commands::search_anime,
            commands::fetch_anime_details,
            commands::get_anilist_viewer,
            commands::fetch_anilist_library,
            commands::save_anilist_entry,
            // Providers & Streams
            commands::get_all_episodes,
            commands::get_anime_catalog_episodes,
            commands::get_episode_sources,
            commands::get_skip_times,
            commands::get_konoha_episodes,
            commands::get_filler_episodes,
            commands::fetch_mal_anime_synopsis,
            // Library & History
            commands::get_watched_episodes,
            commands::get_episode_progress,
            commands::get_continue_watching,
            commands::update_watch_progress,
            commands::add_to_watchlist,
            commands::remove_from_watchlist,
            commands::is_in_watchlist,
            commands::get_watchlist,
            commands::upsert_library_entry,
            commands::get_library_entries,
            // Downloads
            commands::create_download,
            commands::start_episode_download,
            commands::update_download_status,
            commands::get_downloads,
            commands::get_all_downloads,
            commands::delete_download,
            commands::get_download_by_id,
            commands::check_storage_for_download,
            commands::get_download_directory,
            // Schedule & Notifications
            commands::fetch_airing_schedule,
            commands::save_notification_preference,
            commands::get_notification_preference,
            commands::list_notification_preferences,
            // Settings
            commands::get_settings,
            commands::save_settings,
            commands::reset_settings,
            // Player & Discord RPC
            commands::launch_external_player,
            commands::update_discord_presence,
            commands::clear_discord_presence,
            // AnimeThemes & Local Media
            commands::fetch_anime_themes,
            commands::scan_local_anime_directory,
            commands::open_file_in_player,
            // Updates
            commands::check_for_update,
            // Custom Playlists & Queue
            commands::create_playlist,
            commands::list_playlists,
            commands::get_playlist_items,
            commands::add_to_playlist,
            commands::add_multiple_to_playlist,
            commands::remove_from_playlist,
            commands::delete_playlist,
            commands::launch_playlist_external_player,
            // Auth & MAL
            commands::get_auth_urls,
            commands::exchange_mal_code,
            commands::mal_refresh_token,
            commands::mal_update_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
