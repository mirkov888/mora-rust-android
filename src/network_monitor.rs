//! App-UID network counter snapshots supplied by Android's public APIs.
use serde::{Deserialize, Serialize};

const MAX_SNAPSHOTS: usize = 200;
const ALLOWED_TRANSPORTS: [&str; 6] = ["none", "wifi", "cellular", "ethernet", "vpn", "other"];

#[derive(Debug, Clone, Deserialize)]
struct SnapshotInput {
    uid: u32,
    rx_bytes: Option<u64>,
    tx_bytes: Option<u64>,
    transport: String,
    connected: bool,
    validated: bool,
    metered: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct Snapshot {
    id: u64,
    timestamp_ms: i64,
    uid: u32,
    rx_bytes: Option<u64>,
    tx_bytes: Option<u64>,
    rx_delta_bytes: Option<u64>,
    tx_delta_bytes: Option<u64>,
    counter_reset: bool,
    transport: String,
    connected: bool,
    validated: bool,
    metered: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct MonitorStore {
    schema_version: u32,
    snapshots: Vec<Snapshot>,
}

impl Default for MonitorStore {
    fn default() -> Self {
        Self {
            schema_version: 1,
            snapshots: Vec::new(),
        }
    }
}

fn parse(input: &str) -> Result<MonitorStore, &'static str> {
    if input.trim().is_empty() {
        return Ok(MonitorStore::default());
    }
    let store: MonitorStore =
        serde_json::from_str(input).map_err(|_| "invalid_network_monitor_json")?;
    if store.schema_version != 1 {
        return Err("unsupported_schema_version");
    }
    if store.snapshots.len() > MAX_SNAPSHOTS {
        return Err("network_snapshot_limit_exceeded");
    }
    let mut previous_id = 0;
    for item in &store.snapshots {
        if item.id == 0
            || item.id <= previous_id
            || item.timestamp_ms < 0
            || item.uid == 0
            || !ALLOWED_TRANSPORTS.contains(&item.transport.as_str())
        {
            return Err("invalid_network_snapshot");
        }
        previous_id = item.id;
    }
    Ok(store)
}

fn summary(store: &MonitorStore) -> serde_json::Value {
    let total_rx = store
        .snapshots
        .iter()
        .filter_map(|s| s.rx_delta_bytes)
        .fold(0_u64, u64::saturating_add);
    let total_tx = store
        .snapshots
        .iter()
        .filter_map(|s| s.tx_delta_bytes)
        .fold(0_u64, u64::saturating_add);
    serde_json::json!({
        "samples": store.snapshots.len(),
        "total_rx_delta_bytes": total_rx,
        "total_tx_delta_bytes": total_tx,
        "latest": store.snapshots.last()
    })
}

/// Record, list, summarize, or clear snapshots. Counters are for this app UID only.
pub fn operate(input: &str, operation: &str, snapshot_json: &str, timestamp: i64) -> String {
    let result = (|| -> Result<MonitorStore, &'static str> {
        let mut store = parse(input)?;
        match operation {
            "list" | "summary" => {}
            "clear" => store.snapshots.clear(),
            "record" => {
                if timestamp < 0 {
                    return Err("invalid_timestamp");
                }
                let sample: SnapshotInput = serde_json::from_str(snapshot_json)
                    .map_err(|_| "invalid_network_snapshot_json")?;
                if sample.uid == 0 {
                    return Err("invalid_app_uid");
                }
                if !ALLOWED_TRANSPORTS.contains(&sample.transport.as_str()) {
                    return Err("invalid_network_transport");
                }
                if sample.validated && !sample.connected {
                    return Err("invalid_network_state");
                }
                let previous = store.snapshots.last();
                let rx_delta =
                    previous.and_then(|old| sample.rx_bytes?.checked_sub(old.rx_bytes?));
                let tx_delta =
                    previous.and_then(|old| sample.tx_bytes?.checked_sub(old.tx_bytes?));
                let counter_reset = previous.is_some_and(|old| {
                    matches!((old.rx_bytes, sample.rx_bytes), (Some(a), Some(b)) if b < a)
                        || matches!((old.tx_bytes, sample.tx_bytes), (Some(a), Some(b)) if b < a)
                });
                let id = match previous {
                    Some(old) => old.id.checked_add(1).ok_or("snapshot_id_overflow")?,
                    None => 1,
                };
                if store.snapshots.len() == MAX_SNAPSHOTS {
                    store.snapshots.remove(0);
                }
                store.snapshots.push(Snapshot {
                    id,
                    timestamp_ms: timestamp,
                    uid: sample.uid,
                    rx_bytes: sample.rx_bytes,
                    tx_bytes: sample.tx_bytes,
                    rx_delta_bytes: if counter_reset { None } else { rx_delta },
                    tx_delta_bytes: if counter_reset { None } else { tx_delta },
                    counter_reset,
                    transport: sample.transport,
                    connected: sample.connected,
                    validated: sample.validated,
                    metered: sample.metered,
                });
            }
            _ => return Err("unknown_operation"),
        }
        Ok(store)
    })();

    match result {
        Ok(store) => {
            let totals = summary(&store);
            serde_json::json!({"ok": true, "monitor": store, "summary": totals}).to_string()
        }
        Err(error) => serde_json::json!({"ok": false, "error": error}).to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store(result: &str) -> MonitorStore {
        let value: serde_json::Value = serde_json::from_str(result).unwrap();
        serde_json::from_value(value["monitor"].clone()).unwrap()
    }

    fn sample(rx: u64, tx: u64) -> String {
        serde_json::json!({
            "uid": 10123,
            "rx_bytes": rx,
            "tx_bytes": tx,
            "transport": "wifi",
            "connected": true,
            "validated": true,
            "metered": false
        })
        .to_string()
    }

    #[test]
    fn empty_store_lists_successfully() {
        assert!(store(&operate("", "list", "", 0)).snapshots.is_empty());
    }

    #[test]
    fn first_sample_has_no_delta() {
        let current = store(&operate("", "record", &sample(1000, 2000), 10));
        assert_eq!(current.snapshots.len(), 1);
        assert_eq!(current.snapshots[0].rx_delta_bytes, None);
    }

    #[test]
    fn calculates_counter_deltas() {
        let first = store(&operate("", "record", &sample(1000, 2000), 10));
        let second = store(&operate(
            &serde_json::to_string(&first).unwrap(),
            "record",
            &sample(1300, 2600),
            20,
        ));
        assert_eq!(second.snapshots[1].rx_delta_bytes, Some(300));
        assert_eq!(second.snapshots[1].tx_delta_bytes, Some(600));
    }

    #[test]
    fn detects_counter_reset() {
        let first = store(&operate("", "record", &sample(1000, 2000), 10));
        let second = store(&operate(
            &serde_json::to_string(&first).unwrap(),
            "record",
            &sample(10, 20),
            20,
        ));
        assert!(second.snapshots[1].counter_reset);
        assert_eq!(second.snapshots[1].rx_delta_bytes, None);
    }

    #[test]
    fn rejects_unknown_transport() {
        let bad = r#"{"uid":10123,"rx_bytes":1,"tx_bytes":1,"transport":"bluetooth","connected":true,"validated":true,"metered":false}"#;
        assert!(operate("", "record", bad, 1).contains("invalid_network_transport"));
    }

    #[test]
    fn rejects_invalid_network_state() {
        let bad = r#"{"uid":10123,"rx_bytes":1,"tx_bytes":1,"transport":"wifi","connected":false,"validated":true,"metered":false}"#;
        assert!(operate("", "record", bad, 1).contains("invalid_network_state"));
    }

    #[test]
    fn clear_removes_snapshots() {
        let first = store(&operate("", "record", &sample(1000, 2000), 10));
        let cleared = store(&operate(
            &serde_json::to_string(&first).unwrap(),
            "clear",
            "",
            20,
        ));
        assert!(cleared.snapshots.is_empty());
    }

    #[test]
    fn rejects_malformed_store() {
        assert!(operate("{", "list", "", 0).contains("invalid_network_monitor_json"));
    }
}
