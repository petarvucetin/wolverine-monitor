use tauri::State;

use crate::connections::manager::ConnectionManager;
use crate::error::AppError;
use crate::models::dashboard::DashboardStats;
use crate::queries::dashboard;

#[tauri::command]
pub async fn get_dashboard_stats(
    connection_id: String,
    manager: State<'_, ConnectionManager>,
) -> Result<DashboardStats, AppError> {
    let pool = manager.get_pool(&connection_id).await?;
    let tp = manager.get_table_prefix(&connection_id).await?;
    let client = pool.get().await?;
    dashboard::query_stats(&client, &tp).await
}
