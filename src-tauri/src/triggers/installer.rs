use deadpool_postgres::Object;
use crate::error::AppError;

pub struct TriggerInstaller;

impl TriggerInstaller {
    pub const ENVELOPE_TABLES: &'static [&'static str] = &[
        "wolverine_incoming_envelopes",
        "wolverine_outgoing_envelopes",
        "wolverine_dead_letters",
    ];

    const FUNCTION_NAME: &'static str = "wolverine_monitor_notify";
    const CHANNEL: &'static str = "wolverine_monitor";

    /// Generates CREATE OR REPLACE FUNCTION SQL for the NOTIFY trigger function.
    pub fn create_function_sql(schema: &str) -> String {
        format!(
            r#"CREATE OR REPLACE FUNCTION {schema}.{func}()
RETURNS trigger AS $$
-- wolverine_monitor_version: 0.1.0
BEGIN
    PERFORM pg_notify('{channel}', json_build_object(
        'table', TG_TABLE_NAME,
        'schema', TG_TABLE_SCHEMA,
        'operation', TG_OP
    )::text);
    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;"#,
            schema = schema,
            func = Self::FUNCTION_NAME,
            channel = Self::CHANNEL,
        )
    }

    /// Generates CREATE OR REPLACE TRIGGER SQL for a given table.
    pub fn create_trigger_sql(schema: &str, table: &str) -> String {
        let trigger_name = format!("wolverine_monitor_{table}_trigger");
        format!(
            r#"CREATE OR REPLACE TRIGGER {trigger_name}
AFTER INSERT OR UPDATE OR DELETE ON {schema}.{table}
FOR EACH ROW EXECUTE FUNCTION {schema}.{func}();"#,
            trigger_name = trigger_name,
            schema = schema,
            table = table,
            func = Self::FUNCTION_NAME,
        )
    }

    /// Generates DROP TRIGGER IF EXISTS SQL for a given table.
    pub fn drop_trigger_sql(schema: &str, table: &str) -> String {
        let trigger_name = format!("wolverine_monitor_{table}_trigger");
        format!(
            "DROP TRIGGER IF EXISTS {trigger_name} ON {schema}.{table};",
            trigger_name = trigger_name,
            schema = schema,
            table = table,
        )
    }

    /// Generates DROP FUNCTION IF EXISTS SQL.
    pub fn drop_function_sql(schema: &str) -> String {
        format!(
            "DROP FUNCTION IF EXISTS {schema}.{func}();",
            schema = schema,
            func = Self::FUNCTION_NAME,
        )
    }

    /// Creates the notify function then installs triggers on all envelope tables.
    pub async fn install(client: &Object, schema: &str) -> Result<(), AppError> {
        client
            .batch_execute(&Self::create_function_sql(schema))
            .await
            .map_err(|e| AppError::TriggerInstallFailed(e.to_string()))?;

        for table in Self::ENVELOPE_TABLES {
            client
                .batch_execute(&Self::create_trigger_sql(schema, table))
                .await
                .map_err(|e| AppError::TriggerInstallFailed(e.to_string()))?;
        }

        Ok(())
    }

    /// Drops all triggers then drops the notify function.
    pub async fn uninstall(client: &Object, schema: &str) -> Result<(), AppError> {
        for table in Self::ENVELOPE_TABLES {
            client
                .batch_execute(&Self::drop_trigger_sql(schema, table))
                .await
                .map_err(|e| AppError::TriggerInstallFailed(e.to_string()))?;
        }

        client
            .batch_execute(&Self::drop_function_sql(schema))
            .await
            .map_err(|e| AppError::TriggerInstallFailed(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_function_sql_contains_schema_and_version() {
        let sql = TriggerInstaller::create_function_sql("my_schema");
        assert!(sql.contains("my_schema.wolverine_monitor_notify"));
        assert!(sql.contains("wolverine_monitor_version: 0.1.0"));
        assert!(sql.contains("pg_notify"));
        assert!(sql.contains("CREATE OR REPLACE FUNCTION"));
    }

    #[test]
    fn create_trigger_sql_contains_schema_and_table() {
        let sql = TriggerInstaller::create_trigger_sql("public", "wolverine_incoming_envelopes");
        assert!(sql.contains("public.wolverine_incoming_envelopes"));
        assert!(sql.contains("AFTER INSERT OR UPDATE OR DELETE"));
        assert!(sql.contains("wolverine_monitor_wolverine_incoming_envelopes_trigger"));
        assert!(sql.contains("public.wolverine_monitor_notify"));
    }

    #[test]
    fn drop_trigger_sql_contains_schema_and_table() {
        let sql = TriggerInstaller::drop_trigger_sql("public", "wolverine_dead_letters");
        assert!(sql.contains("DROP TRIGGER IF EXISTS"));
        assert!(sql.contains("wolverine_monitor_wolverine_dead_letters_trigger"));
        assert!(sql.contains("public.wolverine_dead_letters"));
    }

    #[test]
    fn drop_function_sql_contains_schema() {
        let sql = TriggerInstaller::drop_function_sql("custom_schema");
        assert!(sql.contains("DROP FUNCTION IF EXISTS"));
        assert!(sql.contains("custom_schema.wolverine_monitor_notify"));
    }

    #[test]
    fn envelope_tables_has_all_expected_tables() {
        assert_eq!(TriggerInstaller::ENVELOPE_TABLES.len(), 3);
        assert!(TriggerInstaller::ENVELOPE_TABLES.contains(&"wolverine_incoming_envelopes"));
        assert!(TriggerInstaller::ENVELOPE_TABLES.contains(&"wolverine_outgoing_envelopes"));
        assert!(TriggerInstaller::ENVELOPE_TABLES.contains(&"wolverine_dead_letters"));
    }

    #[test]
    fn create_trigger_sql_for_each_table() {
        for table in TriggerInstaller::ENVELOPE_TABLES {
            let sql = TriggerInstaller::create_trigger_sql("public", table);
            assert!(sql.contains("AFTER INSERT OR UPDATE OR DELETE"));
            assert!(sql.contains(&format!("public.{table}")));
            assert!(sql.contains("CREATE OR REPLACE TRIGGER"));
        }
    }
}
