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
    #[serde(default)]
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub label: String,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub schema: String,
    pub username: String,
    pub password: String,
    pub ssl_mode: SslMode,
}

impl ConnectionConfig {
    /// Validate a schema name to prevent SQL injection.
    /// Only allows alphanumeric characters and underscores.
    pub fn validate_schema(schema: &str) -> Result<(), String> {
        if schema.is_empty() {
            return Err("Schema name cannot be empty".to_string());
        }
        if !schema.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(format!(
                "Schema name '{}' contains invalid characters. Only alphanumeric and underscores allowed.",
                schema
            ));
        }
        Ok(())
    }
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
    pub label: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub database: Option<String>,
    pub schema: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub ssl_mode: Option<SslMode>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_schema_accepts_simple_name() {
        assert!(ConnectionConfig::validate_schema("public").is_ok());
    }

    #[test]
    fn validate_schema_accepts_underscores() {
        assert!(ConnectionConfig::validate_schema("my_schema").is_ok());
    }

    #[test]
    fn validate_schema_accepts_alphanumeric() {
        assert!(ConnectionConfig::validate_schema("schema2").is_ok());
    }

    #[test]
    fn validate_schema_accepts_wolverine_default() {
        assert!(ConnectionConfig::validate_schema("wolverine").is_ok());
    }

    #[test]
    fn validate_schema_rejects_empty() {
        let result = ConnectionConfig::validate_schema("");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("cannot be empty"));
    }

    #[test]
    fn validate_schema_rejects_spaces() {
        let result = ConnectionConfig::validate_schema("my schema");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("invalid characters"));
    }

    #[test]
    fn validate_schema_rejects_semicolons() {
        let result = ConnectionConfig::validate_schema("public; DROP TABLE");
        assert!(result.is_err());
    }

    #[test]
    fn validate_schema_rejects_quotes() {
        let result = ConnectionConfig::validate_schema("schema'");
        assert!(result.is_err());
    }

    #[test]
    fn validate_schema_rejects_dashes() {
        let result = ConnectionConfig::validate_schema("my-schema");
        assert!(result.is_err());
    }

    #[test]
    fn validate_schema_rejects_dots() {
        let result = ConnectionConfig::validate_schema("public.tables");
        assert!(result.is_err());
    }
}
