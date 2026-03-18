use deadpool_postgres::Object;
use crate::error::AppError;

pub struct TriggerInstaller;

impl TriggerInstaller {
    const BASE_TABLES: &'static [&'static str] = &[
        "incoming_envelopes",
        "outgoing_envelopes",
        "dead_letters",
    ];

    const FUNCTION_NAME: &'static str = "wolverine_monitor_notify";
    const CHANNEL: &'static str = "wolverine_monitor";

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

    pub fn create_trigger_sql(schema: &str, prefix: &str, base_table: &str) -> String {
        let table = format!("{prefix}{base_table}");
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

    pub fn drop_trigger_sql(schema: &str, prefix: &str, base_table: &str) -> String {
        let table = format!("{prefix}{base_table}");
        let trigger_name = format!("wolverine_monitor_{table}_trigger");
        format!(
            "DROP TRIGGER IF EXISTS {trigger_name} ON {schema}.{table};",
            trigger_name = trigger_name,
            schema = schema,
            table = table,
        )
    }

    pub fn drop_function_sql(schema: &str) -> String {
        format!(
            "DROP FUNCTION IF EXISTS {schema}.{func}();",
            schema = schema,
            func = Self::FUNCTION_NAME,
        )
    }

    pub async fn install(client: &Object, schema: &str, prefix: &str) -> Result<(), AppError> {
        client
            .batch_execute(&Self::create_function_sql(schema))
            .await
            .map_err(|e| AppError::TriggerInstallFailed(e.to_string()))?;

        for base_table in Self::BASE_TABLES {
            client
                .batch_execute(&Self::create_trigger_sql(schema, prefix, base_table))
                .await
                .map_err(|e| AppError::TriggerInstallFailed(e.to_string()))?;
        }

        Ok(())
    }

    pub async fn uninstall(client: &Object, schema: &str, prefix: &str) -> Result<(), AppError> {
        for base_table in Self::BASE_TABLES {
            client
                .batch_execute(&Self::drop_trigger_sql(schema, prefix, base_table))
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
