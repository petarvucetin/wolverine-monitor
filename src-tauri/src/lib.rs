mod alerts;
mod commands;
mod config;
mod connections;
mod error;
mod models;
mod monitor;
mod queries;
mod triggers;

use alerts::engine::AlertEngine;
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
        .manage(AlertEngine::new())
        .invoke_handler(tauri::generate_handler![
            // Connection commands
            commands::connection_cmds::add_connection,
            commands::connection_cmds::update_connection,
            commands::connection_cmds::remove_connection,
            commands::connection_cmds::test_connection,
            commands::connection_cmds::list_connections,
            // Trigger commands
            commands::trigger_cmds::install_triggers,
            commands::trigger_cmds::uninstall_triggers,
            // Envelope commands
            commands::envelope_cmds::get_incoming_envelopes,
            commands::envelope_cmds::get_outgoing_envelopes,
            commands::envelope_cmds::get_message_detail,
            // Dead letter commands
            commands::dead_letter_cmds::get_dead_letters,
            commands::dead_letter_cmds::replay_dead_letter,
            commands::dead_letter_cmds::replay_dead_letters_bulk,
            // Node commands
            commands::node_cmds::get_nodes,
            commands::node_cmds::get_node_assignments,
            commands::node_cmds::get_node_records,
            // Queue commands
            commands::queue_cmds::get_queues,
            commands::queue_cmds::get_queue_messages,
            commands::queue_cmds::purge_queue,
            commands::queue_cmds::purge_all_queues,
            // Dashboard commands
            commands::dashboard_cmds::get_dashboard_stats,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
