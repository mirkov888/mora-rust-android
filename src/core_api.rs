//! Stable JSON dispatcher shared by JNI callers.
use serde::Deserialize;
use serde_json::{json, Value};

const REQUEST_SCHEMA_VERSION: u32 = 1;
const MAX_REQUEST_BYTES: usize = 256 * 1024;
const MAX_REQUEST_TOKEN_BYTES: usize = 64;

#[derive(Debug, Deserialize)]
struct CoreRequest {
    schema_version: u32,
    request_id: Option<String>,
    module: String,
    operation: String,
    timestamp_ms: i64,
    #[serde(default)]
    state_json: String,
    #[serde(default)]
    params: Value,
}

impl CoreRequest {
    fn param(&self, key: &str) -> &str {
        self.params.get(key).and_then(Value::as_str).unwrap_or("")
    }
}

fn valid_token(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_REQUEST_TOKEN_BYTES
        && !value.chars().any(char::is_control)
}

fn failure(
    error: &str,
    request_id: Option<&str>,
    module: Option<&str>,
    operation: Option<&str>,
) -> String {
    json!({
        "api_version": crate::CORE_API_VERSION,
        "schema_version": REQUEST_SCHEMA_VERSION,
        "request_id": request_id,
        "module": module,
        "operation": operation,
        "ok": false,
        "error": error
    })
    .to_string()
}

/// Returns the public contract and supported modules for client-side negotiation.
pub fn descriptor() -> String {
    json!({
        "api_version": crate::CORE_API_VERSION,
        "schema_version": REQUEST_SCHEMA_VERSION,
        "capabilities": {
            "mask": crate::mora_core_capabilities(),
            "hex": format!("0x{:x}", crate::mora_core_capabilities())
        },
        "request_fields": [
            "schema_version", "request_id", "module", "operation",
            "timestamp_ms", "state_json", "params"
        ],
        "response_fields": [
            "api_version", "schema_version", "request_id",
            "module", "operation", "ok", "data", "error"
        ],
        "modules": [
            {
                "id": "profile_store",
                "operations": ["list", "create", "update", "delete"],
                "state": "state_json",
                "params": ["id", "label"],
                "scope": "app-owned profiles"
            },
            {
                "id": "system_config",
                "operations": ["get", "set", "reset"],
                "state": "state_json",
                "params": ["key", "value"],
                "scope": "app-owned configuration"
            },
            {
                "id": "runtime_manager",
                "operations": ["list", "start", "stop", "enable", "disable"],
                "state": "state_json",
                "params": ["service_id"],
                "scope": "app-owned logical services"
            },
            {
                "id": "event_logger",
                "operations": ["list", "record", "clear"],
                "state": "state_json",
                "params": ["category", "action", "detail"],
                "scope": "events explicitly submitted by this app"
            },
            {
                "id": "network_monitor",
                "operations": ["list", "summary", "record", "clear"],
                "state": "state_json",
                "params": ["snapshot_json"],
                "scope": "network counters for the owning app UID only"
            },
            {
                "id": "security_auditor",
                "operations": ["audit"],
                "state": "none",
                "params": ["input_json"],
                "scope": "metadata and permissions supplied by the owning app"
            }
        ]
    })
    .to_string()
}

/// Dispatches one versioned request and wraps module output in a stable response envelope.
pub fn dispatch(input: &str) -> String {
    if input.len() > MAX_REQUEST_BYTES {
        return failure("request_too_large", None, None, None);
    }

    let request: CoreRequest = match serde_json::from_str(input) {
        Ok(value) => value,
        Err(_) => return failure("invalid_request_json", None, None, None),
    };

    let request_id = request.request_id.as_deref();
    if request.schema_version != REQUEST_SCHEMA_VERSION {
        return failure(
            "unsupported_request_schema",
            request_id,
            Some(&request.module),
            Some(&request.operation),
        );
    }
    if request_id.is_some_and(|value| !valid_token(value))
        || !valid_token(&request.module)
        || !valid_token(&request.operation)
        || request.timestamp_ms < 0
    {
        return failure(
            "invalid_request_fields",
            request_id,
            Some(&request.module),
            Some(&request.operation),
        );
    }

    let module_output = match request.module.as_str() {
        "profile_store" => crate::profile_store::operate(
            &request.state_json,
            &request.operation,
            request.param("id"),
            request.param("label"),
            request.timestamp_ms,
        ),
        "system_config" => crate::system_config::operate(
            &request.state_json,
            &request.operation,
            request.param("key"),
            request.param("value"),
            request.timestamp_ms,
        ),
        "runtime_manager" => crate::runtime_manager::operate(
            &request.state_json,
            &request.operation,
            request.param("service_id"),
            request.timestamp_ms,
        ),
        "event_logger" => crate::event_logger::operate(
            &request.state_json,
            &request.operation,
            request.param("category"),
            request.param("action"),
            request.param("detail"),
            request.timestamp_ms,
        ),
        "network_monitor" => crate::network_monitor::operate(
            &request.state_json,
            &request.operation,
            request.param("snapshot_json"),
            request.timestamp_ms,
        ),
        "security_auditor" if request.operation == "audit" => {
            crate::security_auditor::audit(request.param("input_json"), request.timestamp_ms)
        }
        "security_auditor" => {
            return failure(
                "unknown_operation",
                request_id,
                Some(&request.module),
                Some(&request.operation),
            );
        }
        _ => {
            return failure(
                "unknown_module",
                request_id,
                Some(&request.module),
                Some(&request.operation),
            );
        }
    };

    let mut module_result: Value = match serde_json::from_str(&module_output) {
        Ok(value) => value,
        Err(_) => {
            return failure(
                "invalid_module_response",
                request_id,
                Some(&request.module),
                Some(&request.operation),
            );
        }
    };
    let ok = module_result
        .get("ok")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let mut response = json!({
        "api_version": crate::CORE_API_VERSION,
        "schema_version": REQUEST_SCHEMA_VERSION,
        "request_id": request_id,
        "module": request.module,
        "operation": request.operation,
        "ok": ok
    });

    if ok {
        if let Some(object) = module_result.as_object_mut() {
            object.remove("ok");
        }
        response["data"] = module_result;
    } else {
        response["error"] = module_result
            .get("error")
            .cloned()
            .unwrap_or_else(|| json!("module_operation_failed"));
    }
    response.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(module: &str, operation: &str, params: Value) -> String {
        json!({
            "schema_version": 1,
            "request_id": "test-01",
            "module": module,
            "operation": operation,
            "timestamp_ms": 10,
            "state_json": "",
            "params": params
        })
        .to_string()
    }

    #[test]
    fn descriptor_lists_six_modules_and_current_contract() {
        let value: Value = serde_json::from_str(&descriptor()).unwrap();
        assert_eq!(value["api_version"], 9);
        assert_eq!(value["schema_version"], 1);
        assert_eq!(value["capabilities"]["mask"], 2047);
        assert_eq!(value["modules"].as_array().unwrap().len(), 6);
    }

    #[test]
    fn dispatches_profile_create_with_standard_envelope() {
        let input = request(
            "profile_store",
            "create",
            json!({"id": "unified_01", "label": "Unified Profile"}),
        );
        let value: Value = serde_json::from_str(&dispatch(&input)).unwrap();
        assert_eq!(value["ok"], true);
        assert_eq!(value["api_version"], 9);
        assert_eq!(value["request_id"], "test-01");
        assert_eq!(
            value["data"]["store"]["profiles"][0]["label"],
            "Unified Profile"
        );
    }

    #[test]
    fn dispatches_system_configuration() {
        let input = request(
            "system_config",
            "set",
            json!({"key": "log_level", "value": "debug"}),
        );
        let value: Value = serde_json::from_str(&dispatch(&input)).unwrap();
        assert_eq!(value["ok"], true);
        assert_eq!(value["data"]["config"]["log_level"], "debug");
    }

    #[test]
    fn dispatches_security_audit() {
        let audit_input = json!({
            "package_name": "com.example.app",
            "target_sdk": 35,
            "debuggable": false,
            "allow_backup": false,
            "uses_cleartext_traffic": false,
            "security_patch": "",
            "permissions": []
        });
        let input = request(
            "security_auditor",
            "audit",
            json!({"input_json": audit_input.to_string()}),
        );
        let value: Value = serde_json::from_str(&dispatch(&input)).unwrap();
        assert_eq!(value["ok"], true);
        assert_eq!(value["data"]["report"]["risk_level"], "good");
    }

    #[test]
    fn rejects_unknown_module_and_schema() {
        let unknown: Value =
            serde_json::from_str(&dispatch(&request("other_app", "list", json!({})))).unwrap();
        assert_eq!(unknown["error"], "unknown_module");

        let mut bad_schema: Value =
            serde_json::from_str(&request("profile_store", "list", json!({}))).unwrap();
        bad_schema["schema_version"] = json!(99);
        assert_eq!(
            serde_json::from_str::<Value>(&dispatch(&bad_schema.to_string())).unwrap()["error"],
            "unsupported_request_schema"
        );
    }

    #[test]
    fn rejects_malformed_oversized_and_invalid_fields() {
        assert_eq!(
            serde_json::from_str::<Value>(&dispatch("{")).unwrap()["error"],
            "invalid_request_json"
        );
        assert_eq!(
            serde_json::from_str::<Value>(&dispatch(&"x".repeat(MAX_REQUEST_BYTES + 1))).unwrap()
                ["error"],
            "request_too_large"
        );

        let mut bad: Value =
            serde_json::from_str(&request("profile_store", "list", json!({}))).unwrap();
        bad["timestamp_ms"] = json!(-1);
        assert_eq!(
            serde_json::from_str::<Value>(&dispatch(&bad.to_string())).unwrap()["error"],
            "invalid_request_fields"
        );
    }

    #[test]
    fn preserves_module_validation_errors() {
        let input = request(
            "system_config",
            "set",
            json!({"key": "unknown_key", "value": "x"}),
        );
        let value: Value = serde_json::from_str(&dispatch(&input)).unwrap();
        assert_eq!(value["ok"], false);
        assert_eq!(value["error"], "unknown_config_key");
        assert_eq!(value["module"], "system_config");
    }
}
