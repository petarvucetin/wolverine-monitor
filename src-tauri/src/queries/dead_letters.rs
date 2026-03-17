use deadpool_postgres::{Object, Pool};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::dead_letter::{BulkReplayResult, DeadLetter, ReplayError, ReplayResult};
use crate::models::envelope::{EnvelopeFilters, PaginatedResult};

/// Query dead letters with optional filters and pagination.
pub async fn query_dead_letters(
    client: &Object,
    schema: &str,
    filters: &EnvelopeFilters,
    page: i64,
    page_size: i64,
) -> Result<PaginatedResult<DeadLetter>, AppError> {
    let mut conditions: Vec<String> = Vec::new();
    let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> = Vec::new();
    let mut param_idx = 1usize;

    if let Some(ref message_type) = filters.message_type {
        conditions.push(format!("message_type ILIKE ${param_idx}"));
        params.push(Box::new(format!("%{message_type}%")));
        param_idx += 1;
    }

    if let Some(ref date_from) = filters.date_from {
        conditions.push(format!("sent_at >= ${param_idx}"));
        params.push(Box::new(*date_from));
        param_idx += 1;
    }

    if let Some(ref date_to) = filters.date_to {
        conditions.push(format!("sent_at <= ${param_idx}"));
        params.push(Box::new(*date_to));
        param_idx += 1;
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    // Count query
    let count_sql = format!(
        "SELECT COUNT(*) FROM {schema}.wolverine_dead_letters {where_clause}"
    );
    let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> =
        params.iter().map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync)).collect();
    let count_row = client.query_one(&count_sql, &param_refs).await?;
    let total: i64 = count_row.get(0);

    // Data query
    let offset = (page - 1) * page_size;
    let data_sql = format!(
        "SELECT id, execution_time, body, message_type, received_at, source, \
         exception_type, exception_message, sent_at, replayable, expires \
         FROM {schema}.wolverine_dead_letters {where_clause} \
         ORDER BY sent_at DESC NULLS LAST \
         LIMIT ${param_idx} OFFSET ${next_idx}",
        param_idx = param_idx,
        next_idx = param_idx + 1,
    );

    let mut data_params = params;
    data_params.push(Box::new(page_size));
    data_params.push(Box::new(offset));

    let data_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> =
        data_params.iter().map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync)).collect();
    let rows = client.query(&data_sql, &data_refs).await?;

    let items: Vec<DeadLetter> = rows
        .iter()
        .map(|row| DeadLetter {
            id: row.get("id"),
            execution_time: row.get("execution_time"),
            body: row.get("body"),
            message_type: row.get("message_type"),
            received_at: row.get("received_at"),
            source: row.get("source"),
            exception_type: row.get("exception_type"),
            exception_message: row.get("exception_message"),
            sent_at: row.get("sent_at"),
            replayable: row.get("replayable"),
            expires: row.get("expires"),
        })
        .collect();

    Ok(PaginatedResult {
        items,
        total,
        page,
        page_size,
    })
}

/// Replay a single dead letter: read it, verify replayable, insert into incoming, delete from dead letters.
/// Takes a Pool so it can obtain a mutable client for the transaction.
pub async fn replay_single(
    pool: &Pool,
    schema: &str,
    id: Uuid,
) -> Result<ReplayResult, AppError> {
    let mut client = pool.get().await?;

    // Start a transaction
    let tx = client
        .transaction()
        .await
        .map_err(AppError::Database)?;

    // Read the dead letter
    let row = tx
        .query_opt(
            &format!(
                "SELECT id, body, message_type, received_at, replayable \
                 FROM {schema}.wolverine_dead_letters WHERE id = $1"
            ),
            &[&id],
        )
        .await?;

    let row = match row {
        Some(r) => r,
        None => {
            return Ok(ReplayResult {
                success: false,
                error: Some(format!("Dead letter {id} not found")),
            });
        }
    };

    let replayable: bool = row.get("replayable");
    if !replayable {
        return Err(AppError::NotReplayable(id));
    }

    let body: Vec<u8> = row.get("body");
    let message_type: String = row.get("message_type");
    let received_at: Option<String> = row.get("received_at");

    // Insert into incoming envelopes with status='Incoming', owner_id=0, attempts=0
    tx.execute(
        &format!(
            "INSERT INTO {schema}.wolverine_incoming_envelopes \
             (id, status, owner_id, attempts, body, message_type, received_at) \
             VALUES ($1, 'Incoming', 0, 0, $2, $3, $4)"
        ),
        &[&id, &body, &message_type, &received_at],
    )
    .await?;

    // Delete from dead letters
    tx.execute(
        &format!("DELETE FROM {schema}.wolverine_dead_letters WHERE id = $1"),
        &[&id],
    )
    .await?;

    tx.commit().await?;

    Ok(ReplayResult {
        success: true,
        error: None,
    })
}

/// Replay multiple dead letters, collecting per-message results.
pub async fn replay_bulk(
    pool: &Pool,
    schema: &str,
    ids: &[Uuid],
) -> Result<BulkReplayResult, AppError> {
    let mut succeeded = 0usize;
    let mut failed = 0usize;
    let mut errors: Vec<ReplayError> = Vec::new();

    for &id in ids {
        match replay_single(pool, schema, id).await {
            Ok(result) if result.success => {
                succeeded += 1;
            }
            Ok(result) => {
                failed += 1;
                errors.push(ReplayError {
                    id,
                    reason: result.error.unwrap_or_else(|| "Unknown error".to_string()),
                });
            }
            Err(e) => {
                failed += 1;
                errors.push(ReplayError {
                    id,
                    reason: e.to_string(),
                });
            }
        }
    }

    Ok(BulkReplayResult {
        succeeded,
        failed,
        errors,
    })
}
