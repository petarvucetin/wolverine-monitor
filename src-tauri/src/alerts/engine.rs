use chrono::Utc;
use tauri::{AppHandle, Emitter};
use tauri_plugin_notification::NotificationExt;

use crate::models::alert::{Alert, AlertRule, AlertRuleKind};
use crate::models::notification::{NotifyOp, NotifyPayload};

pub struct AlertEngine {
    pub rules: Vec<AlertRule>,
}

impl NotifyPayload {
    /// Returns the last segment of a dotted message type.
    /// e.g. "MyApp.Commands.PlaceOrder" -> "PlaceOrder"
    pub fn short_type(&self) -> &str {
        self.message_type
            .rsplit('.')
            .next()
            .unwrap_or(&self.message_type)
    }
}

impl AlertEngine {
    pub fn new() -> Self {
        Self {
            rules: vec![AlertRule {
                id: "default-dead-letter".to_string(),
                enabled: true,
                kind: AlertRuleKind::AnyDeadLetter,
            }],
        }
    }

    /// Check payload against all enabled rules. Only INSERT operations trigger alerts.
    pub fn check(&self, table: &str, payload: &NotifyPayload) -> Vec<Alert> {
        if payload.op != NotifyOp::Insert {
            return Vec::new();
        }

        let mut alerts = Vec::new();
        for rule in &self.rules {
            if !rule.enabled {
                continue;
            }

            let matched = match &rule.kind {
                AlertRuleKind::AnyDeadLetter => table == "dead_letter",
                AlertRuleKind::DeadLetterMessageType { message_type } => {
                    table == "dead_letter"
                        && payload.message_type.contains(message_type.as_str())
                }
                AlertRuleKind::IncomingQueueDepth { .. } => {
                    // Checked during stats polling, not via NOTIFY
                    false
                }
            };

            if matched {
                alerts.push(Alert {
                    rule_id: rule.id.clone(),
                    connection_id: String::new(),
                    message: format!(
                        "Dead letter: {} ({})",
                        payload.short_type(),
                        payload.id
                    ),
                    timestamp: Utc::now(),
                });
            }
        }

        alerts
    }

    /// Emit alert events to the frontend and show system notifications.
    pub fn emit_alerts(&self, app: &AppHandle, connection_id: &str, alerts: Vec<Alert>) {
        for mut alert in alerts {
            alert.connection_id = connection_id.to_string();

            let _ = app.emit("alert:triggered", &alert);

            let _ = app
                .notification()
                .builder()
                .title("Wolverine Monitor Alert")
                .body(&alert.message)
                .show();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn make_payload(op: NotifyOp, message_type: &str) -> NotifyPayload {
        NotifyPayload {
            op,
            id: Uuid::new_v4(),
            message_type: message_type.to_string(),
        }
    }

    #[test]
    fn test_insert_dead_letter_matches_default_rule() {
        let engine = AlertEngine::new();
        let payload = make_payload(NotifyOp::Insert, "MyApp.Commands.PlaceOrder");
        let alerts = engine.check("dead_letter", &payload);
        assert_eq!(alerts.len(), 1);
        assert!(alerts[0].message.contains("PlaceOrder"));
    }

    #[test]
    fn test_update_op_ignored() {
        let engine = AlertEngine::new();
        let payload = make_payload(NotifyOp::Update, "MyApp.Commands.PlaceOrder");
        let alerts = engine.check("dead_letter", &payload);
        assert!(alerts.is_empty());
    }

    #[test]
    fn test_incoming_table_ignored_by_dead_letter_rule() {
        let engine = AlertEngine::new();
        let payload = make_payload(NotifyOp::Insert, "MyApp.Commands.PlaceOrder");
        let alerts = engine.check("incoming", &payload);
        assert!(alerts.is_empty());
    }

    #[test]
    fn test_message_type_filter_matches() {
        let mut engine = AlertEngine::new();
        engine.rules.push(AlertRule {
            id: "type-filter".to_string(),
            enabled: true,
            kind: AlertRuleKind::DeadLetterMessageType {
                message_type: "PlaceOrder".to_string(),
            },
        });
        let payload = make_payload(NotifyOp::Insert, "MyApp.Commands.PlaceOrder");
        let alerts = engine.check("dead_letter", &payload);
        // Both default AnyDeadLetter and the type filter match
        assert_eq!(alerts.len(), 2);
    }

    #[test]
    fn test_message_type_filter_misses() {
        let mut engine = AlertEngine::new();
        engine.rules.clear();
        engine.rules.push(AlertRule {
            id: "type-filter".to_string(),
            enabled: true,
            kind: AlertRuleKind::DeadLetterMessageType {
                message_type: "ShipOrder".to_string(),
            },
        });
        let payload = make_payload(NotifyOp::Insert, "MyApp.Commands.PlaceOrder");
        let alerts = engine.check("dead_letter", &payload);
        assert!(alerts.is_empty());
    }

    #[test]
    fn test_disabled_rule_ignored() {
        let mut engine = AlertEngine::new();
        engine.rules.clear();
        engine.rules.push(AlertRule {
            id: "disabled".to_string(),
            enabled: false,
            kind: AlertRuleKind::AnyDeadLetter,
        });
        let payload = make_payload(NotifyOp::Insert, "MyApp.Commands.PlaceOrder");
        let alerts = engine.check("dead_letter", &payload);
        assert!(alerts.is_empty());
    }

    #[test]
    fn test_short_type_helper() {
        let payload = make_payload(NotifyOp::Insert, "MyApp.Commands.PlaceOrder");
        assert_eq!(payload.short_type(), "PlaceOrder");
    }
}
