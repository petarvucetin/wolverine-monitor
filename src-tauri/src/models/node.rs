use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WolverineNode {
    pub id: Uuid,
    pub node_number: i32,
    pub description: Option<String>,
    pub uri: Option<String>,
    pub started: Option<DateTime<Utc>>,
    pub health_check: Option<DateTime<Utc>>,
    pub version: Option<String>,
    pub capabilities: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NodeHealth {
    Healthy,
    Warning,
    Critical,
    Unknown,
}
