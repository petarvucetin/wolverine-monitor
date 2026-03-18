use tauri::State;

use crate::connections::manager::ConnectionManager;
use crate::error::AppError;
use crate::models::queue::QueueInfo;
use crate::models::envelope::PaginatedResult;
use crate::queries::queues;

#[tauri::command]
pub async fn get_queues(
    connection_id: String,
    manager: State<'_, ConnectionManager>,
) -> Result<Vec<QueueInfo>, AppError> {
    let pool = manager.get_pool(&connection_id).await?;
    let config = manager.get_config(&connection_id).await?;
    let client = pool.get().await?;
    queues::discover_queues(&client, &config.schema, &config.table_prefix).await
}

#[tauri::command]
pub async fn get_queue_messages(
    connection_id: String,
    queue_name: String,
    scheduled: bool,
    page: i64,
    page_size: i64,
    manager: State<'_, ConnectionManager>,
) -> Result<PaginatedResult<serde_json::Value>, AppError> {
    let pool = manager.get_pool(&connection_id).await?;
    let config = manager.get_config(&connection_id).await?;
    let client = pool.get().await?;
    queues::query_queue_messages(
        &client,
        &config.schema,
        &config.table_prefix,
        &queue_name,
        scheduled,
        page,
        page_size,
    )
    .await
}
