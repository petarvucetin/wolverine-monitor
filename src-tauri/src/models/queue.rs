use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueInfo {
    pub name: String,
    pub table_name: String,
    pub count: i64,
    pub scheduled_count: i64,
    pub has_scheduled_table: bool,
}
