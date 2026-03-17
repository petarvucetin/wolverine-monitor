use tauri::State;

use crate::connections::manager::ConnectionManager;
use crate::error::AppError;
use crate::models::envelope::{EnvelopeFilters, IncomingEnvelope, OutgoingEnvelope, PaginatedResult};
use crate::queries::envelopes;

#[tauri::command]
pub async fn get_incoming(
    connection_id: String,
    filters: EnvelopeFilters,
    page: i64,
    page_size: i64,
    manager: State<'_, ConnectionManager>,
) -> Result<PaginatedResult<IncomingEnvelope>, AppError> {
    let pool = manager.get_pool(&connection_id).await?;
    let schema = manager.get_schema(&connection_id).await?;
    let client = pool.get().await?;
    envelopes::query_incoming(&client, &schema, &filters, page, page_size).await
}

#[tauri::command]
pub async fn get_outgoing(
    connection_id: String,
    filters: EnvelopeFilters,
    page: i64,
    page_size: i64,
    manager: State<'_, ConnectionManager>,
) -> Result<PaginatedResult<OutgoingEnvelope>, AppError> {
    let pool = manager.get_pool(&connection_id).await?;
    let schema = manager.get_schema(&connection_id).await?;
    let client = pool.get().await?;
    envelopes::query_outgoing(&client, &schema, &filters, page, page_size).await
}
