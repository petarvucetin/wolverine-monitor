use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertRuleKind {
    AnyDeadLetter,
    DeadLetterMessageType { message_type: String },
    IncomingQueueDepth { threshold: i64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: String,
    pub enabled: bool,
    pub kind: AlertRuleKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub rule_id: String,
    pub connection_id: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}
