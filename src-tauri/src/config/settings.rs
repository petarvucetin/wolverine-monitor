use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub version: u32,
    pub polling_interval_secs: u64,
    pub rolling_window_size: usize,
    pub query_timeout_secs: u64,
    pub node_health_warning_secs: u64,
    pub node_health_critical_secs: u64,
    pub node_poll_interval_secs: u64,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            version: 1,
            polling_interval_secs: 5,
            rolling_window_size: 500,
            query_timeout_secs: 10,
            node_health_warning_secs: 30,
            node_health_critical_secs: 120,
            node_poll_interval_secs: 10,
        }
    }
}
