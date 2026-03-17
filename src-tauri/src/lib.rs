mod commands;
mod config;
mod connections;
mod error;
mod models;

use connections::manager::ConnectionManager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_log::Builder::default().build())
        .manage(ConnectionManager::new())
        .invoke_handler(tauri::generate_handler![
            commands::connection_cmds::add_connection,
            commands::connection_cmds::remove_connection,
            commands::connection_cmds::test_connection,
            commands::connection_cmds::list_connections,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
