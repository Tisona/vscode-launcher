mod commands;
mod config;
mod error;
mod launcher;
mod polling;
mod running;
mod scanner;

use commands::AppState;
use std::path::PathBuf;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config_path = config::default_config_path()
        .unwrap_or_else(|| PathBuf::from("./vscode-launcher-config.json"));
    let state = AppState::new(config_path).expect("failed to init app state");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(state)
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            polling::spawn(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::set_root_folder,
            commands::get_workspaces,
            commands::get_running,
            commands::launch,
            commands::set_pinned,
            commands::set_icon,
            commands::resolved_code_binary,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
