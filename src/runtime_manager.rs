//! Logical lifecycle manager for services owned by this app.
use serde::{Deserialize, Serialize};

const ALLOWED: [&str; 3] = ["profile_store", "config_store", "diagnostics"];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct Service {
    id: String,
    enabled: bool,
    running: bool,
    updated_at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct RuntimeStore {
    schema_version: u32,
    services: Vec<Service>,
}

impl Default for RuntimeStore {
    fn default() -> Self {
        Self {
            schema_version: 1,
            services: ALLOWED
                .iter()
                .map(|id| Service {
                    id: (*id).into(),
                    enabled: true,
                    running: false,
                    updated_at_ms: 0,
                })
                .collect(),
        }
    }
}

fn parse(input: &str) -> Result<RuntimeStore, &'static str> {
    if input.trim().is_empty() {
        return Ok(RuntimeStore::default());
    }
    let store: RuntimeStore = serde_json::from_str(input).map_err(|_| "invalid_runtime_json")?;
    if store.schema_version != 1 {
        return Err("unsupported_schema_version");
    }
    if store.services.len() != ALLOWED.len() {
        return Err("invalid_service_registry");
    }
    for (index, id) in ALLOWED.iter().enumerate() {
        let item = &store.services[index];
        if item.id != *id || item.updated_at_ms < 0 || (item.running && !item.enabled) {
            return Err("invalid_service_state");
        }
    }
    Ok(store)
}

pub fn operate(input: &str, operation: &str, id: &str, timestamp: i64) -> String {
    let result = (|| -> Result<RuntimeStore, &'static str> {
        let mut store = parse(input)?;
        if operation == "list" {
            return Ok(store);
        }
        if timestamp < 0 {
            return Err("invalid_timestamp");
        }
        if !ALLOWED.contains(&id) {
            return Err("unknown_service");
        }
        let item = store
            .services
            .iter_mut()
            .find(|item| item.id == id)
            .ok_or("unknown_service")?;
        match operation {
            "start" => {
                if !item.enabled {
                    return Err("service_disabled");
                }
                item.running = true;
            }
            "stop" => item.running = false,
            "enable" => item.enabled = true,
            "disable" => {
                item.enabled = false;
                item.running = false;
            }
            _ => return Err("unknown_operation"),
        }
        item.updated_at_ms = timestamp;
        Ok(store)
    })();
    match result {
        Ok(runtime) => serde_json::json!({"ok": true, "runtime": runtime}).to_string(),
        Err(error) => serde_json::json!({"ok": false, "error": error}).to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(s: &str) -> RuntimeStore {
        let v: serde_json::Value = serde_json::from_str(s).unwrap();
        serde_json::from_value(v["runtime"].clone()).unwrap()
    }

    #[test]
    fn defaults_have_three_services() {
        assert_eq!(state(&operate("", "list", "", 0)).services.len(), 3);
    }

    #[test]
    fn starts_enabled_service() {
        assert!(state(&operate("", "start", "diagnostics", 1)).services[2].running);
    }

    #[test]
    fn disable_stops_service() {
        let started = state(&operate("", "start", "profile_store", 1));
        let json = serde_json::to_string(&started).unwrap();
        let stopped = state(&operate(&json, "disable", "profile_store", 2));
        assert!(!stopped.services[0].enabled && !stopped.services[0].running);
    }

    #[test]
    fn cannot_start_disabled_service() {
        let disabled = state(&operate("", "disable", "diagnostics", 1));
        let json = serde_json::to_string(&disabled).unwrap();
        assert!(operate(&json, "start", "diagnostics", 2).contains("service_disabled"));
    }

    #[test]
    fn rejects_unknown_service() {
        assert!(operate("", "start", "system_shell", 1).contains("unknown_service"));
    }

    #[test]
    fn rejects_bad_timestamp() {
        assert!(operate("", "start", "diagnostics", -1).contains("invalid_timestamp"));
    }

    #[test]
    fn rejects_malformed_json() {
        assert!(operate("{", "list", "", 0).contains("invalid_runtime_json"));
    }
}
