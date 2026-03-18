use tauri::State;

use crate::connections::manager::ConnectionManager;
use crate::error::AppError;
use crate::triggers::installer::TriggerInstaller;

#[tauri::command]
pub async fn install_triggers(
    connection_id: String,
    manager: State<'_, ConnectionManager>,
) -> Result<(), AppError> {
    let pool = manager.get_pool(&connection_id).await?;
    let config = manager.get_config(&connection_id).await?;
    let client = pool.get().await?;
    TriggerInstaller::install(&client, &config.schema, &config.table_prefix).await?;
    manager.set_triggers_installed(&connection_id, true).await;
    Ok(())
}

#[tauri::command]
pub async fn uninstall_triggers(
    connection_id: String,
    manager: State<'_, ConnectionManager>,
) -> Result<(), AppError> {
    let pool = manager.get_pool(&connection_id).await?;
    let config = manager.get_config(&connection_id).await?;
    let client = pool.get().await?;
    TriggerInstaller::uninstall(&client, &config.schema, &config.table_prefix).await?;
    manager.set_triggers_installed(&connection_id, false).await;
    Ok(())
}
