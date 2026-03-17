use serde_json::Value;

/// Apply all migrations from current version to latest.
pub fn migrate(config: &mut Value, current_version: u32, target_version: u32) -> Result<(), String> {
    for version in current_version..target_version {
        match version {
            // Future migrations go here:
            // 1 => migrate_v1_to_v2(config)?,
            _ => {} // No migration needed
        }
    }
    config["version"] = serde_json::json!(target_version);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrate_noop_same_version() {
        let mut config = serde_json::json!({"version": 1});
        migrate(&mut config, 1, 1).unwrap();
        assert_eq!(config["version"], 1);
    }
}
