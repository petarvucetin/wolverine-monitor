mod commands;
mod config;
mod connections;
mod error;
mod models;
mod monitor;
mod queries;
mod triggers;

use connections::manager::ConnectionManager;
use monitor::listener::NotifyListener;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_log::Builder::default().build())
        .manage(ConnectionManager::new())
        .manage(NotifyListener::new())
        .invoke_handler(tauri::generate_handler![
            // Connection commands
            commands::connection_cmds::add_connection,
            commands::connection_cmds::remove_connection,
            commands::connection_cmds::test_connection,
            commands::connection_cmds::list_connections,
            // Trigger commands
            commands::trigger_cmds::install_triggers,
            commands::trigger_cmds::uninstall_triggers,
            // Envelope commands
            commands::envelope_cmds::get_incoming,
            commands::envelope_cmds::get_outgoing,
            // Dead letter commands
            commands::dead_letter_cmds::get_dead_letters,
            commands::dead_letter_cmds::replay_dead_letter,
            commands::dead_letter_cmds::replay_dead_letters_bulk,
            // Node commands
            commands::node_cmds::get_nodes,
            // Dashboard commands
            commands::dashboard_cmds::get_dashboard_stats,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
