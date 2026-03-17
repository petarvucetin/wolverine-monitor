use tauri::State;

use crate::connections::manager::ConnectionManager;
use crate::error::AppError;
use crate::models::connection::{ConnectionConfig, ConnectionInfo, SslMode};

#[tauri::command]
pub async fn add_connection(
    config: ConnectionConfig,
    manager: State<'_, ConnectionManager>,
) -> Result<String, AppError> {
    manager.add(config).await
}

#[tauri::command]
pub async fn remove_connection(
    connection_id: String,
    manager: State<'_, ConnectionManager>,
) -> Result<(), AppError> {
    manager.remove(&connection_id).await
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
) -> Result<Vec<ConnectionInfo>, AppError> {
    Ok(manager.list().await)
}
