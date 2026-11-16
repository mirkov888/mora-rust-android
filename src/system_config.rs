//! App-owned runtime configuration with strict, typed validation.
use serde::{Deserialize, Serialize};

const DEFAULT_CACHE_LIMIT_MB: u32 = 512;
const MIN_CACHE_LIMIT_MB: u32 = 64;
const MAX_CACHE_LIMIT_MB: u32 = 4096;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct SystemConfig {
    pub schema_version: u32,
    pub log_level: String,
    pub network_mode: String,
    pub background_sync: bool,
    pub cache_limit_mb: u32,
    pub telemetry_enabled: bool,
    pub updated_at_ms: i64,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            schema_version: 1,
            log_level: "info".into(),
            network_mode: "auto".into(),
            background_sync: true,
            cache_limit_mb: DEFAULT_CACHE_LIMIT_MB,
            telemetry_enabled: false,
            updated_at_ms: 0,
        }
    }
}

fn parse_config(input: &str) -> Result<SystemConfig, &'static str> {
    if input.trim().is_empty() {
        return Ok(SystemConfig::default());
    }
    let config: SystemConfig = serde_json::from_str(input).map_err(|_| "invalid_config_json")?;
    validate(&config)?;
    Ok(config)
}

fn validate(config: &SystemConfig) -> Result<(), &'static str> {
    if config.schema_version != 1 {
        return Err("unsupported_schema_version");
    }
    if !["error", "warn", "info", "debug"].contains(&config.log_level.as_str()) {
        return Err("invalid_log_level");
    }
    if !["auto", "wifi_only", "offline"].contains(&config.network_mode.as_str()) {
        return Err("invalid_network_mode");
    }
    if !(MIN_CACHE_LIMIT_MB..=MAX_CACHE_LIMIT_MB).contains(&config.cache_limit_mb) {
        return Err("cache_limit_out_of_range");
    }
    if config.updated_at_ms < 0 {
        return Err("invalid_timestamp");
    }
    Ok(())
}

fn set_value(config: &mut SystemConfig, key: &str, value: &str) -> Result<(), &'static str> {
    match key {
        "log_level" => {
            if !["error", "warn", "info", "debug"].contains(&value) {
                return Err("invalid_log_level");
            }
            config.log_level = value.into();
        }
        "network_mode" => {
            if !["auto", "wifi_only", "offline"].contains(&value) {
                return Err("invalid_network_mode");
            }
            config.network_mode = value.into();
        }
        "background_sync" => {
            config.background_sync = value.parse().map_err(|_| "expected_boolean")?;
        }
        "cache_limit_mb" => {
            let parsed: u32 = value.parse().map_err(|_| "expected_integer")?;
            if !(MIN_CACHE_LIMIT_MB..=MAX_CACHE_LIMIT_MB).contains(&parsed) {
                return Err("cache_limit_out_of_range");
            }
            config.cache_limit_mb = parsed;
        }
        "telemetry_enabled" => {
            config.telemetry_enabled = value.parse().map_err(|_| "expected_boolean")?;
        }
        _ => return Err("unknown_config_key"),
    }
    Ok(())
}

/// Apply get/set/reset to a JSON document. The result is always JSON.
pub fn operate(input: &str, operation: &str, key: &str, value: &str, timestamp: i64) -> String {
    let result = (|| -> Result<SystemConfig, &'static str> {
        let mut config = parse_config(input)?;
        match operation {
            "get" => {}
            "reset" => config = SystemConfig::default(),
            "set" => {
                set_value(&mut config, key, value)?;
                if timestamp < 0 {
                    return Err("invalid_timestamp");
                }
                config.updated_at_ms = timestamp;
            }
            _ => return Err("unknown_operation"),
        }
        validate(&config)?;
        Ok(config)
    })();

    match result {
        Ok(config) => serde_json::json!({ "ok": true, "config": config }).to_string(),
        Err(error) => serde_json::json!({ "ok": false, "error": error }).to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config_from(result: &str) -> SystemConfig {
        serde_json::from_str(
            &serde_json::from_str::<serde_json::Value>(result).unwrap()["config"].to_string(),
        )
        .unwrap()
    }

    #[test]
    fn defaults_are_safe() {
        let config = config_from(&operate("", "get", "", "", 0));
        assert_eq!(config.log_level, "info");
        assert_eq!(config.network_mode, "auto");
        assert!(!config.telemetry_enabled);
        assert_eq!(config.cache_limit_mb, 512);
    }

    #[test]
    fn updates_known_setting() {
        let result = operate("", "set", "log_level", "debug", 42);
        assert_eq!(config_from(&result).log_level, "debug");
    }

    #[test]
    fn rejects_unknown_key() {
        assert!(operate("", "set", "root_access", "true", 1).contains("unknown_config_key"));
    }

    #[test]
    fn rejects_invalid_enum() {
        assert!(operate("", "set", "network_mode", "anything", 1).contains("invalid_network_mode"));
    }

    #[test]
    fn rejects_invalid_boolean() {
        assert!(operate("", "set", "background_sync", "yes", 1).contains("expected_boolean"));
    }

    #[test]
    fn rejects_cache_out_of_range() {
        assert!(
            operate("", "set", "cache_limit_mb", "99999", 1).contains("cache_limit_out_of_range")
        );
    }

    #[test]
    fn rejects_malformed_json() {
        assert!(operate("{", "get", "", "", 0).contains("invalid_config_json"));
    }

    #[test]
    fn reset_restores_defaults() {
        let config = config_from(&operate("", "reset", "", "", 0));
        assert_eq!(config, SystemConfig::default());
    }

    #[test]
    fn rejects_negative_timestamp() {
        assert!(operate("", "set", "log_level", "info", -1).contains("invalid_timestamp"));
    }

    #[test]
    fn rejects_unknown_operation() {
        assert!(operate("", "launch_shell", "", "", 0).contains("unknown_operation"));
    }
}
