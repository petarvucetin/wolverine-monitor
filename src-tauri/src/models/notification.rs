use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NotifyOp {
    #[serde(rename = "INSERT")]
    Insert,
    #[serde(rename = "UPDATE")]
    Update,
    #[serde(rename = "DELETE")]
    Delete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifyPayload {
    pub op: NotifyOp,
    pub id: Uuid,
    pub message_type: String,
}

impl NotifyPayload {
    pub fn parse(json_str: &str) -> Option<Self> {
        serde_json::from_str(json_str).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_insert_payload() {
        let json = r#"{"op":"INSERT","id":"a1b2c3d4-e5f6-7890-abcd-ef1234567890","message_type":"MyApp.Commands.PlaceOrder"}"#;
        let payload = NotifyPayload::parse(json).unwrap();
        assert_eq!(payload.op, NotifyOp::Insert);
        assert_eq!(payload.message_type, "MyApp.Commands.PlaceOrder");
    }

    #[test]
    fn test_parse_delete_payload() {
        let json = r#"{"op":"DELETE","id":"a1b2c3d4-e5f6-7890-abcd-ef1234567890","message_type":"MyApp.Events.OrderShipped"}"#;
        let payload = NotifyPayload::parse(json).unwrap();
        assert_eq!(payload.op, NotifyOp::Delete);
    }

    #[test]
    fn test_parse_malformed_returns_none() {
        assert!(NotifyPayload::parse("not json").is_none());
        assert!(NotifyPayload::parse(r#"{"op":"INVALID"}"#).is_none());
    }
}
