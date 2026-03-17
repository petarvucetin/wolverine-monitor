use deadpool_postgres::Object;

use crate::error::AppError;
use crate::models::dashboard::DashboardStats;

/// Query aggregate counts for the dashboard.
pub async fn query_stats(
    client: &Object,
    schema: &str,
) -> Result<DashboardStats, AppError> {
    // Incoming envelope counts grouped by status
    let incoming_sql = format!(
        "SELECT status, COUNT(*) as cnt \
         FROM {schema}.wolverine_incoming_envelopes \
         GROUP BY status"
    );
    let incoming_rows = client.query(&incoming_sql, &[]).await?;

    let mut incoming_count: i64 = 0;
    let mut incoming_scheduled: i64 = 0;
    let mut incoming_handled: i64 = 0;

    for row in &incoming_rows {
        let status: String = row.get("status");
        let cnt: i64 = row.get("cnt");
        match status.as_str() {
            "Incoming" => incoming_count = cnt,
            "Scheduled" => incoming_scheduled = cnt,
            "Handled" => incoming_handled = cnt,
            _ => {}
        }
    }

    // Outgoing count
    let outgoing_sql = format!(
        "SELECT COUNT(*) FROM {schema}.wolverine_outgoing_envelopes"
    );
    let outgoing_row = client.query_one(&outgoing_sql, &[]).await?;
    let outgoing_count: i64 = outgoing_row.get(0);

    // Dead letter count
    let dead_letter_sql = format!(
        "SELECT COUNT(*) FROM {schema}.wolverine_dead_letters"
    );
    let dead_letter_row = client.query_one(&dead_letter_sql, &[]).await?;
    let dead_letter_count: i64 = dead_letter_row.get(0);

    Ok(DashboardStats {
        incoming_count,
        incoming_scheduled,
        incoming_handled,
        outgoing_count,
        dead_letter_count,
        throughput: Vec::new(),
    })
}
