#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api;
mod commands;
mod config;
mod launch;
mod skin;

use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let base = app
                .path()
                .app_data_dir()
                .map_err(|e| format!("cannot resolve app data dir: {e}"))?;
            let data = config::AppData::initialize(base)?;
            app.manage(data);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::snapshot,
            commands::refresh_server,
            commands::set_server_link,
            commands::login,
            commands::register,
            commands::set_offline_account,
            commands::logout,
            commands::detect_java,
            commands::set_java,
            commands::set_game_dir,
            commands::set_source,
            commands::set_concurrency,
            commands::set_toggle,
            commands::start_launch,
            skin::fetch_skin,
            skin::upload_skin,
            skin::delete_skin,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run the Syncraft launcher");
}
