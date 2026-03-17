use tauri::State;
use tokio_postgres::types::Type;
use uuid::Uuid;

use crate::connections::manager::ConnectionManager;
use crate::error::AppError;
use crate::models::connection::ConnectionConfig;
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

#[tauri::command]
pub async fn get_message_detail(
    connection_id: String,
    table: String,
    id: String,
    manager: State<'_, ConnectionManager>,
) -> Result<serde_json::Value, AppError> {
    let pool = manager.get_pool(&connection_id).await?;
    let schema = manager.get_schema(&connection_id).await?;

    // Validate schema
    ConnectionConfig::validate_schema(&schema)
        .map_err(|e| AppError::Config(e))?;

    // Map table name to actual table
    let table_name = match table.as_str() {
        "incoming" => "wolverine_incoming_envelopes",
        "outgoing" => "wolverine_outgoing_envelopes",
        "dead_letter" => "wolverine_dead_letters",
        _ => return Err(AppError::Config(format!("Unknown table: {table}"))),
    };

    let client = pool.get().await?;
    let sql = format!("SELECT * FROM {schema}.{table_name} WHERE id = $1");

    let uuid = Uuid::parse_str(&id)
        .map_err(|e| AppError::Config(format!("Invalid UUID: {e}")))?;

    let row = client.query_opt(&sql, &[&uuid]).await?;
    let row = row.ok_or_else(|| AppError::Config(format!("Row not found: {id}")))?;

    // Convert row to JSON using column type inspection
    let mut map = serde_json::Map::new();
    for (i, col) in row.columns().iter().enumerate() {
        let name = col.name().to_string();
        let value = match *col.type_() {
            Type::UUID => {
                let v: Option<Uuid> = row.get(i);
                match v {
                    Some(u) => serde_json::Value::String(u.to_string()),
                    None => serde_json::Value::Null,
                }
            }
            Type::INT4 => {
                let v: Option<i32> = row.get(i);
                match v {
                    Some(n) => serde_json::Value::Number(n.into()),
                    None => serde_json::Value::Null,
                }
            }
            Type::INT8 => {
                let v: Option<i64> = row.get(i);
                match v {
                    Some(n) => serde_json::Value::Number(n.into()),
                    None => serde_json::Value::Null,
                }
            }
            Type::BOOL => {
                let v: Option<bool> = row.get(i);
                match v {
                    Some(b) => serde_json::Value::Bool(b),
                    None => serde_json::Value::Null,
                }
            }
            Type::BYTEA => {
                let v: Option<Vec<u8>> = row.get(i);
                match v {
                    Some(bytes) => {
                        // Try to decode as JSON first, fall back to base64
                        match serde_json::from_slice::<serde_json::Value>(&bytes) {
                            Ok(json) => json,
                            Err(_) => {
                                use base64::Engine;
                                let encoded = base64::engine::general_purpose::STANDARD.encode(&bytes);
                                serde_json::Value::String(encoded)
                            }
                        }
                    }
                    None => serde_json::Value::Null,
                }
            }
            _ => {
                // Default: try as string, fall back to null
                let v: Option<String> = row.try_get(i).ok().flatten();
                match v {
                    Some(s) => serde_json::Value::String(s),
                    None => serde_json::Value::Null,
                }
            }
        };
        map.insert(name, value);
    }

    Ok(serde_json::Value::Object(map))
}
