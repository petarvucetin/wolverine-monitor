use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadLetter {
    pub id: Uuid,
    pub execution_time: Option<DateTime<Utc>>,
    pub body: Vec<u8>,
    pub message_type: String,
    pub received_at: Option<String>,
    pub source: Option<String>,
    pub exception_type: Option<String>,
    pub exception_message: Option<String>,
    pub sent_at: Option<DateTime<Utc>>,
    pub replayable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayResult {
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkReplayResult {
    pub succeeded: usize,
    pub failed: usize,
    pub errors: Vec<ReplayError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayError {
    pub id: Uuid,
    pub reason: String,
}
