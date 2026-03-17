use tauri::State;

use crate::connections::manager::ConnectionManager;
use crate::error::AppError;
use crate::models::node::WolverineNode;
use crate::queries::nodes;

#[tauri::command]
pub async fn get_nodes(
    connection_id: String,
    manager: State<'_, ConnectionManager>,
) -> Result<Vec<WolverineNode>, AppError> {
    let pool = manager.get_pool(&connection_id).await?;
    let schema = manager.get_schema(&connection_id).await?;
    let client = pool.get().await?;
    nodes::query_nodes(&client, &schema).await
}
