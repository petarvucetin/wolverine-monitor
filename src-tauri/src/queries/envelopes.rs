use deadpool_postgres::Object;

use crate::error::AppError;
use crate::models::envelope::{
    EnvelopeFilters, EnvelopeStatus, IncomingEnvelope, OutgoingEnvelope, PaginatedResult,
};

/// Query incoming envelopes with optional filters and pagination.
pub async fn query_incoming(
    client: &Object,
    schema: &str,
    filters: &EnvelopeFilters,
    page: i64,
    page_size: i64,
) -> Result<PaginatedResult<IncomingEnvelope>, AppError> {
    let mut conditions: Vec<String> = Vec::new();
    let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> = Vec::new();
    let mut param_idx = 1usize;

    if let Some(ref status) = filters.status {
        conditions.push(format!("status = ${param_idx}"));
        params.push(Box::new(status.clone()));
        param_idx += 1;
    }

    if let Some(ref message_type) = filters.message_type {
        conditions.push(format!("message_type ILIKE ${param_idx}"));
        params.push(Box::new(format!("%{message_type}%")));
        param_idx += 1;
    }

    if let Some(ref date_from) = filters.date_from {
        conditions.push(format!("execution_time >= ${param_idx}"));
        params.push(Box::new(*date_from));
        param_idx += 1;
    }

    if let Some(ref date_to) = filters.date_to {
        conditions.push(format!("execution_time <= ${param_idx}"));
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
        "SELECT COUNT(*) FROM {schema}.wolverine_incoming_envelopes {where_clause}"
    );
    let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> =
        params.iter().map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync)).collect();
    let count_row = client.query_one(&count_sql, &param_refs).await?;
    let total: i64 = count_row.get(0);

    // Data query
    let offset = (page - 1) * page_size;
    let data_sql = format!(
        "SELECT id, status, owner_id, execution_time, attempts, body, message_type, \
         received_at, keep_until \
         FROM {schema}.wolverine_incoming_envelopes {where_clause} \
         ORDER BY execution_time DESC NULLS LAST \
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

    let items: Vec<IncomingEnvelope> = rows
        .iter()
        .map(|row| {
            let status_str: String = row.get("status");
            IncomingEnvelope {
                id: row.get("id"),
                status: EnvelopeStatus::from_str(&status_str).unwrap_or(EnvelopeStatus::Incoming),
                owner_id: row.get("owner_id"),
                execution_time: row.get("execution_time"),
                attempts: row.get("attempts"),
                body: row.get("body"),
                message_type: row.get("message_type"),
                received_at: row.get("received_at"),
                keep_until: row.get("keep_until"),
            }
        })
        .collect();

    Ok(PaginatedResult {
        items,
        total,
        page,
        page_size,
    })
}

/// Query outgoing envelopes with optional filters and pagination.
pub async fn query_outgoing(
    client: &Object,
    schema: &str,
    filters: &EnvelopeFilters,
    page: i64,
    page_size: i64,
) -> Result<PaginatedResult<OutgoingEnvelope>, AppError> {
    let mut conditions: Vec<String> = Vec::new();
    let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> = Vec::new();
    let mut param_idx = 1usize;

    if let Some(ref message_type) = filters.message_type {
        conditions.push(format!("message_type ILIKE ${param_idx}"));
        params.push(Box::new(format!("%{message_type}%")));
        param_idx += 1;
    }

    if let Some(ref date_from) = filters.date_from {
        conditions.push(format!("deliver_by >= ${param_idx}"));
        params.push(Box::new(*date_from));
        param_idx += 1;
    }

    if let Some(ref date_to) = filters.date_to {
        conditions.push(format!("deliver_by <= ${param_idx}"));
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
        "SELECT COUNT(*) FROM {schema}.wolverine_outgoing_envelopes {where_clause}"
    );
    let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> =
        params.iter().map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync)).collect();
    let count_row = client.query_one(&count_sql, &param_refs).await?;
    let total: i64 = count_row.get(0);

    // Data query
    let offset = (page - 1) * page_size;
    let data_sql = format!(
        "SELECT id, owner_id, destination, deliver_by, body, attempts, message_type \
         FROM {schema}.wolverine_outgoing_envelopes {where_clause} \
         ORDER BY deliver_by DESC NULLS LAST \
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

    let items: Vec<OutgoingEnvelope> = rows
        .iter()
        .map(|row| OutgoingEnvelope {
            id: row.get("id"),
            owner_id: row.get("owner_id"),
            destination: row.get("destination"),
            deliver_by: row.get("deliver_by"),
            body: row.get("body"),
            attempts: row.get("attempts"),
            message_type: row.get("message_type"),
        })
        .collect();

    Ok(PaginatedResult {
        items,
        total,
        page,
        page_size,
    })
}
