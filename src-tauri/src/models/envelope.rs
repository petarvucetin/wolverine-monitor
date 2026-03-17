use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EnvelopeStatus {
    Incoming,
    Scheduled,
    Handled,
}

impl EnvelopeStatus {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Incoming" => Some(Self::Incoming),
            "Scheduled" => Some(Self::Scheduled),
            "Handled" => Some(Self::Handled),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncomingEnvelope {
    pub id: Uuid,
    pub status: EnvelopeStatus,
    pub owner_id: i32,
    pub execution_time: Option<DateTime<Utc>>,
    pub attempts: i32,
    pub body: Vec<u8>,
    pub message_type: String,
    pub received_at: Option<String>,
    pub keep_until: Option<DateTime<Utc>>,
    pub timestamp: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutgoingEnvelope {
    pub id: Uuid,
    pub owner_id: i32,
    pub destination: String,
    pub deliver_by: Option<DateTime<Utc>>,
    pub body: Vec<u8>,
    pub attempts: i32,
    pub message_type: String,
    pub timestamp: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvelopeFilters {
    pub status: Option<String>,
    pub message_type: Option<String>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}
