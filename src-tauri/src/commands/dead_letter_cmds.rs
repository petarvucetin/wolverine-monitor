use tauri::State;
use uuid::Uuid;

use crate::connections::manager::ConnectionManager;
use crate::error::AppError;
use crate::models::dead_letter::{BulkReplayResult, DeadLetter, ReplayResult};
use crate::models::envelope::{EnvelopeFilters, PaginatedResult};
use crate::queries::dead_letters;

#[tauri::command]
pub async fn get_dead_letters(
    connection_id: String,
    filters: EnvelopeFilters,
    page: i64,
    page_size: i64,
    manager: State<'_, ConnectionManager>,
) -> Result<PaginatedResult<DeadLetter>, AppError> {
    let pool = manager.get_pool(&connection_id).await?;
    let schema = manager.get_schema(&connection_id).await?;
    let client = pool.get().await?;
    dead_letters::query_dead_letters(&client, &schema, &filters, page, page_size).await
}

#[tauri::command]
pub async fn replay_dead_letter(
    connection_id: String,
    id: String,
    manager: State<'_, ConnectionManager>,
) -> Result<ReplayResult, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|e| AppError::Config(format!("Invalid UUID: {e}")))?;
    let pool = manager.get_pool(&connection_id).await?;
    let schema = manager.get_schema(&connection_id).await?;
    dead_letters::replay_single(&pool, &schema, uuid).await
}

#[tauri::command]
pub async fn replay_dead_letters_bulk(
    connection_id: String,
    ids: Vec<String>,
    manager: State<'_, ConnectionManager>,
) -> Result<BulkReplayResult, AppError> {
    let uuids: Vec<Uuid> = ids
        .iter()
        .map(|s| Uuid::parse_str(s).map_err(|e| AppError::Config(format!("Invalid UUID: {e}"))))
        .collect::<Result<Vec<_>, _>>()?;
    let pool = manager.get_pool(&connection_id).await?;
    let schema = manager.get_schema(&connection_id).await?;
    dead_letters::replay_bulk(&pool, &schema, &uuids).await
}
