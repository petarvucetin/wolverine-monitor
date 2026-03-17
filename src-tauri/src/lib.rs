mod commands;
mod config;
mod connections;
mod error;
mod models;
mod triggers;

use connections::manager::ConnectionManager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_log::Builder::default().build())
        .manage(ConnectionManager::new())
        .invoke_handler(tauri::generate_handler![
            // Connection commands
            commands::connection_cmds::add_connection,
            commands::connection_cmds::remove_connection,
            commands::connection_cmds::test_connection,
            commands::connection_cmds::list_connections,
            // Trigger commands
            commands::trigger_cmds::install_triggers,
            commands::trigger_cmds::uninstall_triggers,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
