use deadpool_postgres::Object;
use tokio_postgres::types::Type;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::node::WolverineNode;

/// Query all Wolverine nodes, ordered by node_number.
pub async fn query_nodes(
    client: &Object,
    tp: &str,
) -> Result<Vec<WolverineNode>, AppError> {
    let sql = format!(
        "SELECT id, node_number, description, uri, started, health_check, version, capabilities::text \
         FROM {tp}nodes ORDER BY node_number"
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

/// Generic helper: query all rows from a table and return as JSON.
fn row_to_json(row: &tokio_postgres::Row) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    for (i, col) in row.columns().iter().enumerate() {
        let name = col.name().to_string();
        let value = match *col.type_() {
            Type::UUID => {
                let v: Option<Uuid> = row.get(i);
                v.map(|u| serde_json::Value::String(u.to_string()))
                    .unwrap_or(serde_json::Value::Null)
            }
            Type::INT4 => {
                let v: Option<i32> = row.get(i);
                v.map(|n| serde_json::Value::Number(n.into()))
                    .unwrap_or(serde_json::Value::Null)
            }
            Type::INT8 => {
                let v: Option<i64> = row.get(i);
                v.map(|n| serde_json::Value::Number(n.into()))
                    .unwrap_or(serde_json::Value::Null)
            }
            Type::BOOL => {
                let v: Option<bool> = row.get(i);
                v.map(serde_json::Value::Bool)
                    .unwrap_or(serde_json::Value::Null)
            }
            _ => {
                let v: Option<String> = row.try_get(i).ok().flatten();
                v.map(serde_json::Value::String)
                    .unwrap_or(serde_json::Value::Null)
            }
        };
        map.insert(name, value);
    }
    serde_json::Value::Object(map)
}

/// Query all rows from node_assignments table.
pub async fn query_node_assignments(
    client: &Object,
    tp: &str,
) -> Result<Vec<serde_json::Value>, AppError> {
    let sql = format!("SELECT * FROM {tp}node_assignments ORDER BY 1");
    let rows = client.query(&sql, &[]).await?;
    Ok(rows.iter().map(row_to_json).collect())
}

/// Query all rows from node_records table.
pub async fn query_node_records(
    client: &Object,
    tp: &str,
) -> Result<Vec<serde_json::Value>, AppError> {
    let sql = format!("SELECT * FROM {tp}node_records ORDER BY 1");
    let rows = client.query(&sql, &[]).await?;
    Ok(rows.iter().map(row_to_json).collect())
}
