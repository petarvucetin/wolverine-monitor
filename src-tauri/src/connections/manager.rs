use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use deadpool_postgres::{Config, Pool, Runtime};
use tokio_postgres::NoTls;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::connection::{ConnectionConfig, ConnectionStatus, ConnectionInfo, ConnectionUpdate, SslMode};

pub struct ManagedConnection {
    pub config: ConnectionConfig,
    pub pool: Pool,
    pub status: ConnectionStatus,
    pub triggers_installed: bool,
}

pub struct ConnectionManager {
    connections: Arc<RwLock<HashMap<String, ManagedConnection>>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add(&self, mut config: ConnectionConfig) -> Result<String, AppError> {
        ConnectionConfig::validate_schema(&config.schema)
            .map_err(|e| AppError::Config(e))?;

        if config.id.is_empty() {
            config.id = Uuid::new_v4().to_string();
        }
        let id = config.id.clone();
        let pool = self.create_pool(&config)?;

        // Test the connection
        let client = pool.get().await?;
        client.simple_query("SELECT 1").await?;
        drop(client);

        let managed = ManagedConnection {
            config,
            pool,
            status: ConnectionStatus::Connected,
            triggers_installed: false,
        };

        self.connections.write().await.insert(id.clone(), managed);
        Ok(id)
    }

    /// Register a connection from saved config without testing connectivity.
    /// Used on startup to restore persisted connections.
    pub async fn add_saved(&self, mut config: ConnectionConfig) -> Result<String, AppError> {
        ConnectionConfig::validate_schema(&config.schema)
            .map_err(|e| AppError::Config(e))?;

        if config.id.is_empty() {
            config.id = Uuid::new_v4().to_string();
        }
        let id = config.id.clone();
        let pool = self.create_pool(&config)?;

        // Try to connect but don't fail if unreachable
        let status = match pool.get().await {
            Ok(client) => {
                let _ = client.simple_query("SELECT 1").await;
                ConnectionStatus::Connected
            }
            Err(_) => ConnectionStatus::Disconnected,
        };

        let managed = ManagedConnection {
            config,
            pool,
            status,
            triggers_installed: false,
        };

        self.connections.write().await.insert(id.clone(), managed);
        Ok(id)
    }

    pub async fn remove(&self, connection_id: &str) -> Result<(), AppError> {
        self.connections
            .write()
            .await
            .remove(connection_id)
            .ok_or_else(|| AppError::ConnectionNotFound(connection_id.to_string()))?;
        Ok(())
    }

    pub async fn update(&self, connection_id: &str, updates: ConnectionUpdate) -> Result<(), AppError> {
        // Validate new schema name if provided
        if let Some(ref schema) = updates.schema {
            ConnectionConfig::validate_schema(schema)
                .map_err(|e| AppError::Config(e))?;
        }

        let mut conns = self.connections.write().await;
        let managed = conns
            .get_mut(connection_id)
            .ok_or_else(|| AppError::ConnectionNotFound(connection_id.to_string()))?;

        let mut needs_reconnect = false;

        if let Some(name) = updates.name {
            managed.config.name = name;
        }
        if let Some(routes) = updates.routes {
            managed.config.routes = routes;
        }
        if let Some(table_prefix) = updates.table_prefix {
            managed.config.table_prefix = table_prefix;
        }
        if let Some(host) = updates.host {
            managed.config.host = host;
            needs_reconnect = true;
        }
        if let Some(port) = updates.port {
            managed.config.port = port;
            needs_reconnect = true;
        }
        if let Some(database) = updates.database {
            managed.config.database = database;
            needs_reconnect = true;
        }
        if let Some(schema) = updates.schema {
            managed.config.schema = schema;
        }
        if let Some(username) = updates.username {
            managed.config.username = username;
            needs_reconnect = true;
        }
        if let Some(password) = updates.password {
            managed.config.password = password;
            needs_reconnect = true;
        }
        if let Some(ssl_mode) = updates.ssl_mode {
            managed.config.ssl_mode = ssl_mode;
            needs_reconnect = true;
        }

        if needs_reconnect {
            let new_pool = self.create_pool(&managed.config)?;
            managed.pool = new_pool;
            managed.status = ConnectionStatus::Reconnecting;
        }

        Ok(())
    }

    pub async fn get_pool(&self, connection_id: &str) -> Result<Pool, AppError> {
        let conns = self.connections.read().await;
        let managed = conns
            .get(connection_id)
            .ok_or_else(|| AppError::ConnectionNotFound(connection_id.to_string()))?;
        Ok(managed.pool.clone())
    }

    pub async fn get_schema(&self, connection_id: &str) -> Result<String, AppError> {
        let conns = self.connections.read().await;
        let managed = conns
            .get(connection_id)
            .ok_or_else(|| AppError::ConnectionNotFound(connection_id.to_string()))?;
        Ok(managed.config.schema.clone())
    }

    /// Returns the qualified table prefix: "{schema}.{table_prefix}"
    /// e.g. "wolverine.wolverine_" so callers can do `format!("{tp}incoming_envelopes")`
    pub async fn get_table_prefix(&self, connection_id: &str) -> Result<String, AppError> {
        let conns = self.connections.read().await;
        let managed = conns
            .get(connection_id)
            .ok_or_else(|| AppError::ConnectionNotFound(connection_id.to_string()))?;
        Ok(format!("{}.{}", managed.config.schema, managed.config.table_prefix))
    }

    pub async fn get_config(&self, connection_id: &str) -> Result<ConnectionConfig, AppError> {
        let conns = self.connections.read().await;
        let managed = conns
            .get(connection_id)
            .ok_or_else(|| AppError::ConnectionNotFound(connection_id.to_string()))?;
        Ok(managed.config.clone())
    }

    pub async fn list(&self) -> Vec<ConnectionInfo> {
        let conns = self.connections.read().await;
        conns
            .values()
            .map(|m| ConnectionInfo {
                config: m.config.clone(),
                status: m.status.clone(),
                triggers_installed: m.triggers_installed,
            })
            .collect()
    }

    pub async fn set_triggers_installed(&self, connection_id: &str, installed: bool) {
        let mut conns = self.connections.write().await;
        if let Some(managed) = conns.get_mut(connection_id) {
            managed.triggers_installed = installed;
        }
    }

    pub async fn test_connection(
        host: &str, port: u16, database: &str,
        username: &str, password: &str, _ssl_mode: &SslMode,
    ) -> Result<(), AppError> {
        let mut cfg = Config::new();
        cfg.host = Some(host.to_string());
        cfg.port = Some(port);
        cfg.dbname = Some(database.to_string());
        cfg.user = Some(username.to_string());
        cfg.password = Some(password.to_string());

        let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls)
            .map_err(|e| AppError::Config(e.to_string()))?;

        let client = pool.get().await?;
        client.simple_query("SELECT 1").await?;
        Ok(())
    }

    fn create_pool(&self, config: &ConnectionConfig) -> Result<Pool, AppError> {
        let mut cfg = Config::new();
        cfg.host = Some(config.host.clone());
        cfg.port = Some(config.port);
        cfg.dbname = Some(config.database.clone());
        cfg.user = Some(config.username.clone());
        cfg.password = Some(config.password.clone());

        cfg.create_pool(Some(Runtime::Tokio1), NoTls)
            .map_err(|e| AppError::Config(e.to_string()))
    }
}
