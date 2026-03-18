use std::collections::HashSet;
use deadpool_postgres::Object;
use tokio_postgres::types::Type;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::queue::QueueInfo;
use crate::models::envelope::PaginatedResult;

/// Discover queue tables and return stats for each queue.
pub async fn discover_queues(
    client: &Object,
    schema: &str,
    table_prefix: &str,
) -> Result<Vec<QueueInfo>, AppError> {
    let pattern = format!("{table_prefix}queue_%");
    let rows = client
        .query(
            "SELECT table_name FROM information_schema.tables \
             WHERE table_schema = $1 AND table_name LIKE $2 AND table_type = 'BASE TABLE' \
             ORDER BY table_name",
            &[&schema, &pattern],
        )
        .await?;

    let all_tables: Vec<String> = rows.iter().map(|r| r.get(0)).collect();

    let scheduled_set: HashSet<&str> = all_tables
        .iter()
        .filter(|t| t.ends_with("_scheduled"))
        .map(|t| t.as_str())
        .collect();

    let queue_prefix = format!("{table_prefix}queue_");
    let mut queues: Vec<QueueInfo> = Vec::new();

    for table in &all_tables {
        if table.ends_with("_scheduled") {
            continue;
        }
        let name = table
            .strip_prefix(&queue_prefix)
            .unwrap_or(table)
            .to_string();

        let scheduled_table = format!("{table}_scheduled");
        let has_scheduled = scheduled_set.contains(scheduled_table.as_str());

        let count: i64 = client
            .query_one(&format!("SELECT COUNT(*) FROM {schema}.{table}"), &[])
            .await?
            .get(0);

        let scheduled_count: i64 = if has_scheduled {
            client
                .query_one(
                    &format!("SELECT COUNT(*) FROM {schema}.{scheduled_table}"),
                    &[],
                )
                .await?
                .get(0)
        } else {
            0
        };

        queues.push(QueueInfo {
            name,
            table_name: table.clone(),
            count,
            scheduled_count,
            has_scheduled_table: has_scheduled,
        });
    }

    Ok(queues)
}

/// Query messages from a specific queue table with pagination.
pub async fn query_queue_messages(
    client: &Object,
    schema: &str,
    table_prefix: &str,
    queue_name: &str,
    scheduled: bool,
    page: i64,
    page_size: i64,
) -> Result<PaginatedResult<serde_json::Value>, AppError> {
    // Validate queue name
    if !queue_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(AppError::Config(format!(
            "Queue name '{}' contains invalid characters",
            queue_name
        )));
    }

    let table = if scheduled {
        format!("{schema}.{table_prefix}queue_{queue_name}_scheduled")
    } else {
        format!("{schema}.{table_prefix}queue_{queue_name}")
    };

    let count: i64 = client
        .query_one(&format!("SELECT COUNT(*) FROM {table}"), &[])
        .await?
        .get(0);

    let offset = (page - 1) * page_size;
    let sql = format!("SELECT * FROM {table} LIMIT $1 OFFSET $2");
    let rows = client.query(&sql, &[&page_size, &offset]).await?;

    let items: Vec<serde_json::Value> = rows
        .iter()
        .map(|row| {
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
                    Type::BYTEA => {
                        let v: Option<Vec<u8>> = row.get(i);
                        match v {
                            Some(bytes) => {
                                serde_json::from_slice::<serde_json::Value>(&bytes)
                                    .unwrap_or_else(|_| {
                                        use base64::Engine;
                                        serde_json::Value::String(
                                            base64::engine::general_purpose::STANDARD.encode(&bytes),
                                        )
                                    })
                            }
                            None => serde_json::Value::Null,
                        }
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
        })
        .collect();

    Ok(PaginatedResult {
        items,
        total: count,
        page,
        page_size,
    })
}
