//! Bounded activity log for events owned by this application.
use serde::{Deserialize, Serialize};

const MAX_EVENTS: usize = 500;
const MAX_CATEGORY_BYTES: usize = 24;
const MAX_ACTION_BYTES: usize = 48;
const MAX_DETAIL_BYTES: usize = 240;
const ALLOWED_CATEGORIES: [&str; 4] = ["profile", "config", "runtime", "system"];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct Event {
    id: u64,
    timestamp_ms: i64,
    category: String,
    action: String,
    detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct EventLog {
    schema_version: u32,
    events: Vec<Event>,
}

impl Default for EventLog {
    fn default() -> Self {
        Self { schema_version: 1, events: Vec::new() }
    }
}

fn valid_text(value: &str, max_bytes: usize) -> bool {
    !value.trim().is_empty()
        && value.len() <= max_bytes
        && !value.chars().any(char::is_control)
}

fn parse(input: &str) -> Result<EventLog, &'static str> {
    if input.trim().is_empty() {
        return Ok(EventLog::default());
    }
    let log: EventLog = serde_json::from_str(input).map_err(|_| "invalid_event_log_json")?;
    if log.schema_version != 1 {
        return Err("unsupported_schema_version");
    }
    if log.events.len() > MAX_EVENTS {
        return Err("event_log_limit_exceeded");
    }
    let mut previous_id = 0;
    for event in &log.events {
        if event.id == 0
            || event.id <= previous_id
            || event.timestamp_ms < 0
            || !ALLOWED_CATEGORIES.contains(&event.category.as_str())
            || !valid_text(&event.action, MAX_ACTION_BYTES)
            || !valid_text(&event.detail, MAX_DETAIL_BYTES)
        {
            return Err("invalid_event_record");
        }
        previous_id = event.id;
    }
    Ok(log)
}

/// List, append, or clear events. The oldest event is discarded at capacity.
pub fn operate(
    input: &str,
    operation: &str,
    category: &str,
    action: &str,
    detail: &str,
    timestamp: i64,
) -> String {
    let result = (|| -> Result<EventLog, &'static str> {
        let mut log = parse(input)?;
        match operation {
            "list" => {}
            "clear" => log.events.clear(),
            "record" => {
                if timestamp < 0 {
                    return Err("invalid_timestamp");
                }
                if !ALLOWED_CATEGORIES.contains(&category) {
                    return Err("invalid_event_category");
                }
                if !valid_text(action, MAX_ACTION_BYTES) {
                    return Err("invalid_event_action");
                }
                if !valid_text(detail, MAX_DETAIL_BYTES) {
                    return Err("invalid_event_detail");
                }
                let id = log.events.last().map_or(1, |event| event.id.saturating_add(1));
                if id == 0 {
                    return Err("event_id_overflow");
                }
                if log.events.len() == MAX_EVENTS {
                    log.events.remove(0);
                }
                log.events.push(Event {
                    id,
                    timestamp_ms: timestamp,
                    category: category.into(),
                    action: action.into(),
                    detail: detail.into(),
                });
            }
            _ => return Err("unknown_operation"),
        }
        Ok(log)
    })();

    match result {
        Ok(log) => serde_json::json!({"ok": true, "log": log}).to_string(),
        Err(error) => serde_json::json!({"ok": false, "error": error}).to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn log(result: &str) -> EventLog {
        let value: serde_json::Value = serde_json::from_str(result).unwrap();
        serde_json::from_value(value["log"].clone()).unwrap()
    }

    #[test]
    fn empty_log_is_valid() {
        assert!(log(&operate("", "list", "", "", "", 0)).events.is_empty());
    }

    #[test]
    fn records_event() {
        let result = operate("", "record", "runtime", "start", "diagnostics started", 42);
        let events = log(&result).events;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].category, "runtime");
        assert_eq!(events[0].timestamp_ms, 42);
    }

    #[test]
    fn appends_in_order() {
        let first = operate("", "record", "profile", "create", "created profile", 10);
        let json = log(&first);
        let second = operate(&serde_json::to_string(&json).unwrap(), "record", "config", "set", "log level", 20);
        let events = log(&second).events;
        assert_eq!(events.len(), 2);
        assert!(events[0].id < events[1].id);
    }

    #[test]
    fn clear_removes_events() {
        let first = operate("", "record", "system", "startup", "app opened", 10);
        let cleared = operate(&log(&first).pipe_json(), "clear", "", "", "", 20);
        assert!(log(&cleared).events.is_empty());
    }

    #[test]
    fn rejects_unknown_category() {
        assert!(operate("", "record", "other_app", "read", "data", 1).contains("invalid_event_category"));
    }

    #[test]
    fn rejects_control_characters() {
        assert!(operate("", "record", "system", "bad\naction", "detail", 1).contains("invalid_event_action"));
    }

    #[test]
    fn rejects_negative_timestamp() {
        assert!(operate("", "record", "system", "startup", "opened", -1).contains("invalid_timestamp"));
    }

    #[test]
    fn rejects_malformed_json() {
        assert!(operate("{", "list", "", "", "", 0).contains("invalid_event_log_json"));
    }

    #[test]
    fn keeps_only_latest_five_hundred_events() {
        let mut current = String::new();
        for timestamp in 1..=501 {
            current = operate(&current, "record", "system", "tick", "test event", timestamp);
            let value: serde_json::Value = serde_json::from_str(&current).unwrap();
            current = value["log"].to_string();
        }
        let events = log(&format!(r#"{{"ok":true,"log":{current}}}"#)).events;
        assert_eq!(events.len(), MAX_EVENTS);
        assert_eq!(events[0].timestamp_ms, 2);
        assert_eq!(events[499].timestamp_ms, 501);
    }
}
