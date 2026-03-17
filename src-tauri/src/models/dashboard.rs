use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DashboardStats {
    pub incoming_count: i64,
    pub incoming_scheduled: i64,
    pub incoming_handled: i64,
    pub outgoing_count: i64,
    pub dead_letter_count: i64,
    pub throughput: Vec<ThroughputPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThroughputPoint {
    pub timestamp: DateTime<Utc>,
    pub incoming: i64,
    pub outgoing: i64,
}
