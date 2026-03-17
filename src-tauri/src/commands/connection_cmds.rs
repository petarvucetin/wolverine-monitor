use tauri::State;

use crate::connections::manager::ConnectionManager;
use crate::connections::persistence;
use crate::error::AppError;
use crate::models::connection::{ConnectionConfig, ConnectionInfo, ConnectionUpdate, SslMode};
use crate::monitor::listener::NotifyListener;

/// Collect all current configs from the manager and persist to disk.
async fn persist(app: &tauri::AppHandle, manager: &ConnectionManager) {
    let conns = manager.list().await;
    let configs: Vec<ConnectionConfig> = conns.into_iter().map(|c| c.config).collect();
    match persistence::save_connections(app, &configs) {
        Ok(_) => tracing::info!("Persisted {} connection(s) to disk", configs.len()),
        Err(e) => tracing::error!("Failed to persist connections: {e}"),
    }
}

#[tauri::command]
pub async fn add_connection(
    config: ConnectionConfig,
    manager: State<'_, ConnectionManager>,
    listener: State<'_, NotifyListener>,
    app: tauri::AppHandle,
) -> Result<String, AppError> {
    let listen_config = config.clone();
    let id = manager.add(config).await?;
    let mut listen_config = listen_config;
    listen_config.id = id.clone();
    listener.start_listening(app.clone(), listen_config).await;
    persist(&app, &manager).await;
    Ok(id)
}

#[tauri::command]
pub async fn remove_connection(
    connection_id: String,
    manager: State<'_, ConnectionManager>,
    app: tauri::AppHandle,
) -> Result<(), AppError> {
    manager.remove(&connection_id).await?;
    persist(&app, &manager).await;
    Ok(())
}

#[tauri::command]
pub async fn update_connection(
    connection_id: String,
    updates: ConnectionUpdate,
    manager: State<'_, ConnectionManager>,
    app: tauri::AppHandle,
) -> Result<(), AppError> {
    manager.update(&connection_id, updates).await?;
    persist(&app, &manager).await;
    Ok(())
}

#[tauri::command]
pub async fn test_connection(
    host: String,
    port: u16,
    database: String,
    username: String,
    password: String,
    ssl_mode: SslMode,
) -> Result<(), AppError> {
    ConnectionManager::test_connection(&host, port, &database, &username, &password, &ssl_mode).await
}

#[tauri::command]
pub async fn list_connections(
    manager: State<'_, ConnectionManager>,
    listener: State<'_, NotifyListener>,
    app: tauri::AppHandle,
) -> Result<Vec<ConnectionInfo>, AppError> {
    let list = manager.list().await;
    if !list.is_empty() {
        return Ok(list);
    }

    // First call with empty manager — try restoring from disk
    let saved = persistence::load_connections(&app).unwrap_or_default();
    tracing::info!("Restoring {} saved connection(s) from disk", saved.len());
    for config in saved {
        let listen_config = config.clone();
        match manager.add_saved(config).await {
            Ok(id) => {
                let mut lc = listen_config;
                lc.id = id;
                listener.start_listening(app.clone(), lc).await;
            }
            Err(e) => {
                tracing::warn!("Failed to restore connection '{}': {e}", listen_config.name);
            }
        }
    }
    Ok(manager.list().await)
}
