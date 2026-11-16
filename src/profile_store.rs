//! Rust-owned profile document operations; Kotlin performs private-file I/O.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Profile {
    pub id: String,
    pub label: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ProfileStore {
    pub profiles: Vec<Profile>,
}

fn parse(input: &str) -> Result<ProfileStore, String> {
    if input.trim().is_empty() {
        Ok(ProfileStore::default())
    } else {
        serde_json::from_str(input).map_err(|_| "Invalid profile store JSON".to_string())
    }
}

fn response(store: &ProfileStore, error: Option<&str>) -> String {
    let mut obj = serde_json::Map::new();
    obj.insert("ok".into(), serde_json::Value::Bool(error.is_none()));
    if let Some(e) = error {
        obj.insert("error".into(), serde_json::Value::String(e.into()));
    }
    obj.insert(
        "store".into(),
        serde_json::to_value(store).unwrap_or_default(),
    );
    serde_json::Value::Object(obj).to_string()
}

pub fn operate(input: &str, op: &str, id: &str, label: &str, now: i64) -> String {
    let mut s = match parse(input) {
        Ok(v) => v,
        Err(e) => return response(&ProfileStore::default(), Some(&e)),
    };
    let err = match op {
        "list" => None,
        "create" => {
            let name = crate::identity::normalize_profile_label(label);
            if !crate::profiles::is_valid_profile_id(id) {
                Some("Invalid profile ID")
            } else if s.profiles.iter().any(|p| p.id == id) {
                Some("Profile ID already exists")
            } else if let Ok(name) = name {
                s.profiles.push(Profile {
                    id: id.into(),
                    label: name,
                    created_at: now,
                    updated_at: now,
                });
                None
            } else {
                Some("Invalid profile label")
            }
        }
        "update" => {
            let name = crate::identity::normalize_profile_label(label);
            if let Some(p) = s.profiles.iter_mut().find(|p| p.id == id) {
                if let Ok(name) = name {
                    p.label = name;
                    p.updated_at = now.max(p.created_at);
                    None
                } else {
                    Some("Invalid profile label")
                }
            } else {
                Some("Profile not found")
            }
        }
        "delete" => {
            let n = s.profiles.len();
            s.profiles.retain(|p| p.id != id);
            if n == s.profiles.len() {
                Some("Profile not found")
            } else {
                None
            }
        }
        _ => Some("Unsupported profile operation"),
    };
    response(&s, err)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crud_round_trip() {
        let a = operate(
            r#"{"profiles":[]}"#,
            "create",
            "daily_01",
            " Daily Profile ",
            1,
        );
        let v: serde_json::Value = serde_json::from_str(&a).unwrap();
        assert_eq!(v["ok"], true);
        assert_eq!(v["store"]["profiles"][0]["label"], "Daily Profile");
        let b = operate(&v["store"].to_string(), "update", "daily_01", "Work", 2);
        let v: serde_json::Value = serde_json::from_str(&b).unwrap();
        assert_eq!(v["store"]["profiles"][0]["label"], "Work");
        let c = operate(&v["store"].to_string(), "delete", "daily_01", "", 3);
        let v: serde_json::Value = serde_json::from_str(&c).unwrap();
        assert_eq!(v["store"]["profiles"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn rejects_bad_and_duplicate_ids() {
        let a = operate(r#"{"profiles":[]}"#, "create", "../bad", "Name", 1);
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&a).unwrap()["ok"],
            false
        );
        let a = operate(r#"{"profiles":[]}"#, "create", "same", "One", 1);
        let s = serde_json::from_str::<serde_json::Value>(&a).unwrap()["store"].to_string();
        let b = operate(&s, "create", "same", "Two", 2);
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&b).unwrap()["ok"],
            false
        );
    }
}
