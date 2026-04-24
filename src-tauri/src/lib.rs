mod commands;
mod config;
mod error;
mod launcher;
mod polling;
mod running;
mod scanner;
mod window_enum;

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
            commands::launch,
            commands::set_pinned,
            commands::set_icon,
            commands::resolved_code_binary,
            commands::focus_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
