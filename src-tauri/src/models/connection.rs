use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SslMode {
    Disable,
    Prefer,
    Require,
    VerifyCa,
}

impl Default for SslMode {
    fn default() -> Self {
        Self::Prefer
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub schema: String,
    pub username: String,
    pub password: String,
    pub ssl_mode: SslMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Reconnecting,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub config: ConnectionConfig,
    pub status: ConnectionStatus,
    pub triggers_installed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionUpdate {
    pub name: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub database: Option<String>,
    pub schema: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub ssl_mode: Option<SslMode>,
}
