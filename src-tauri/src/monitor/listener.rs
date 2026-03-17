use std::sync::Arc;
use std::time::Duration;

use futures_util::stream::poll_fn;
use futures_util::StreamExt;
use tauri::{AppHandle, Emitter};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio_postgres::AsyncMessage;

use crate::models::connection::ConnectionConfig;
use crate::models::notification::NotifyPayload;

pub struct NotifyListener {
    handles: Arc<RwLock<Vec<JoinHandle<()>>>>,
}

impl NotifyListener {
    pub fn new() -> Self {
        Self {
            handles: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn start_listening(&self, app: AppHandle, config: ConnectionConfig) {
        let connection_id = config.id.clone();
        let schema = config.schema.clone();
        let conn_string = build_connection_string(&config);
        let app_clone = app.clone();

        let handle = tokio::spawn(async move {
            let mut retry_count: u32 = 0;

            loop {
                match tokio_postgres::connect(&conn_string, tokio_postgres::NoTls).await {
                    Ok((client, mut connection)) => {
                        retry_count = 0;

                        // Emit connected status
                        let _ = app_clone.emit(
                            "connection:status",
                            serde_json::json!({
                                "connection_id": connection_id,
                                "status": "listening"
                            }),
                        );

                        // Subscribe to channels
                        let channels = vec![
                            format!("{}_wolverine_incoming_envelopes_changed", schema),
                            format!("{}_wolverine_outgoing_envelopes_changed", schema),
                            format!("{}_wolverine_dead_letters_changed", schema),
                        ];

                        let mut listen_failed = false;
                        for channel in &channels {
                            let query = format!("LISTEN {}", channel);
                            if let Err(e) = client.batch_execute(&query).await {
                                tracing::error!(
                                    "Failed to LISTEN on channel {}: {}",
                                    channel,
                                    e
                                );
                                listen_failed = true;
                                break;
                            }
                        }

                        if listen_failed {
                            // Will retry via the outer loop
                            continue;
                        }

                        // Poll for notifications using poll_fn stream
                        // poll_message returns Poll<Option<Result<AsyncMessage, Error>>>
                        // poll_fn yields items of type Option<Result<...>>,
                        // stream ends when poll_fn yields None (i.e., poll_message returns Ready(None))
                        let mut stream = poll_fn(|cx| connection.poll_message(cx).map(Some));

                        let mut disconnected = false;
                        while let Some(msg) = stream.next().await {
                            match msg {
                                Some(Ok(AsyncMessage::Notification(notification))) => {
                                    let channel = notification.channel();
                                    let payload_str = notification.payload();

                                    if let Some(payload) = NotifyPayload::parse(payload_str) {
                                        let table = table_from_channel(channel, &schema);

                                        let event_data = serde_json::json!({
                                            "connection_id": connection_id,
                                            "table": table,
                                            "op": payload.op,
                                            "id": payload.id,
                                            "message_type": payload.message_type,
                                        });

                                        let _ = app_clone.emit("envelope:changed", event_data);
                                    }
                                }
                                Some(Ok(AsyncMessage::Notice(notice))) => {
                                    tracing::debug!("DB notice: {}", notice.message());
                                }
                                Some(Ok(_)) => {}
                                Some(Err(e)) => {
                                    tracing::error!(
                                        "Connection error for {}: {}",
                                        connection_id,
                                        e
                                    );
                                    disconnected = true;
                                    break;
                                }
                                None => {
                                    // Connection closed
                                    break;
                                }
                            }
                        }

                        if !disconnected {
                            // Stream ended without error - connection closed
                            tracing::warn!(
                                "Notification stream ended for {}",
                                connection_id
                            );
                        }
                    }
                    Err(e) => {
                        tracing::error!(
                            "Failed to connect for LISTEN on {}: {}",
                            connection_id,
                            e
                        );
                    }
                }

                // Reconnect with exponential backoff
                retry_count += 1;
                if retry_count > 5 {
                    let _ = app_clone.emit(
                        "connection:status",
                        serde_json::json!({
                            "connection_id": connection_id,
                            "status": "error",
                            "message": "LISTEN connection failed after 5 retries"
                        }),
                    );
                    break;
                }

                let backoff_secs = (1u64 << (retry_count - 1)).min(30);
                let _ = app_clone.emit(
                    "connection:status",
                    serde_json::json!({
                        "connection_id": connection_id,
                        "status": "reconnecting",
                        "retry": retry_count,
                    }),
                );

                tokio::time::sleep(Duration::from_secs(backoff_secs)).await;
            }
        });

        self.handles.write().await.push(handle);
    }

    pub async fn stop_all(&self) {
        let mut handles = self.handles.write().await;
        for handle in handles.drain(..) {
            handle.abort();
        }
    }
}

fn build_connection_string(config: &ConnectionConfig) -> String {
    format!(
        "host={} port={} dbname={} user={} password={}",
        config.host, config.port, config.database, config.username, config.password
    )
}

fn table_from_channel(channel: &str, schema: &str) -> &'static str {
    let prefix = format!("{}_wolverine_", schema);
    let suffix = channel.strip_prefix(&prefix).unwrap_or(channel);
    match suffix {
        "incoming_envelopes_changed" => "incoming",
        "outgoing_envelopes_changed" => "outgoing",
        "dead_letters_changed" => "dead_letter",
        _ => "unknown",
    }
}
