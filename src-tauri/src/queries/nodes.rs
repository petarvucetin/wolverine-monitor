use deadpool_postgres::Object;

use crate::error::AppError;
use crate::models::node::WolverineNode;

/// Query all Wolverine nodes, ordered by node_number.
pub async fn query_nodes(
    client: &Object,
    schema: &str,
) -> Result<Vec<WolverineNode>, AppError> {
    let sql = format!(
        "SELECT id, node_number, description, uri, started, health_check, version, capabilities \
         FROM {schema}.wolverine_nodes ORDER BY node_number"
    );

    let rows = client.query(&sql, &[]).await?;

    let nodes: Vec<WolverineNode> = rows
        .iter()
        .map(|row| WolverineNode {
            id: row.get("id"),
            node_number: row.get("node_number"),
            description: row.get("description"),
            uri: row.get("uri"),
            started: row.get("started"),
            health_check: row.get("health_check"),
            version: row.get("version"),
            capabilities: row.get("capabilities"),
        })
        .collect();

    Ok(nodes)
}
