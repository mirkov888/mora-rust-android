//! App-scoped Android security and permission posture audit.
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct AuditInput {
    package_name: String,
    target_sdk: u32,
    debuggable: bool,
    allow_backup: bool,
    uses_cleartext_traffic: bool,
    security_patch: String,
    permissions: Vec<PermissionInput>,
}

#[derive(Debug, Deserialize)]
struct PermissionInput {
    name: String,
    granted: bool,
}

#[derive(Debug, Serialize)]
struct Finding {
    id: String,
    severity: &'static str,
    title: String,
    detail: String,
    recommendation: String,
}

#[derive(Debug, Serialize)]
struct AuditReport {
    schema_version: u32,
    package_name: String,
    security_patch: String,
    generated_at_ms: i64,
    risk_score: u32,
    risk_level: &'static str,
    permission_count: usize,
    granted_dangerous_count: usize,
    denied_dangerous_count: usize,
    findings: Vec<Finding>,
}

fn dangerous_permission(name: &str) -> Option<(&'static str, &'static str, &'static str)> {
    match name {
        "android.permission.CAMERA" => Some(("medium", "Camera access", "Only request camera access when the feature needs it.")),
        "android.permission.RECORD_AUDIO" => Some(("medium", "Microphone access", "Only request microphone access when recording is actively used.")),
        "android.permission.ACCESS_FINE_LOCATION" | "android.permission.ACCESS_COARSE_LOCATION" | "android.permission.ACCESS_BACKGROUND_LOCATION" =>
            Some(("high", "Location access", "Prefer approximate or foreground-only location and explain its use.")),
        "android.permission.READ_CONTACTS" | "android.permission.WRITE_CONTACTS" =>
            Some(("high", "Contacts access", "Avoid broad contact access; use Android's contact picker where possible.")),
        "android.permission.READ_CALL_LOG" | "android.permission.WRITE_CALL_LOG" | "android.permission.PROCESS_OUTGOING_CALLS" =>
            Some(("high", "Call log access", "Remove call-log access unless it is essential and policy-compliant.")),
        "android.permission.READ_SMS" | "android.permission.RECEIVE_SMS" | "android.permission.SEND_SMS" =>
            Some(("high", "SMS access", "Remove SMS access unless it is essential and policy-compliant.")),
        "android.permission.READ_PHONE_STATE" | "android.permission.CALL_PHONE" =>
            Some(("medium", "Phone access", "Request only the minimum phone permission required by the feature.")),
        "android.permission.READ_MEDIA_IMAGES" | "android.permission.READ_MEDIA_VIDEO" | "android.permission.READ_MEDIA_AUDIO" | "android.permission.READ_EXTERNAL_STORAGE" =>
            Some(("medium", "Media access", "Prefer Android's Photo Picker or scoped media access.")),
        _ => None,
    }
}

fn add(findings: &mut Vec<Finding>, id: &str, severity: &'static str, title: &str, detail: String, recommendation: &str) {
    findings.push(Finding {
        id: id.to_string(),
        severity,
        title: title.to_string(),
        detail,
        recommendation: recommendation.to_string(),
    });
}

/// Audits only metadata and permission states supplied by the owning Android app.
pub fn audit(input_json: &str, now_ms: i64) -> String {
    let result = (|| -> Result<AuditReport, &'static str> {
        let input: AuditInput = serde_json::from_str(input_json).map_err(|_| "invalid_audit_input")?;
        if input.package_name.is_empty() || input.package_name.len() > 255
            || input.package_name.bytes().any(|b| b.is_ascii_control())
            || input.security_patch.len() > 32
            || input.permissions.len() > 256
            || now_ms < 0
        {
            return Err("invalid_audit_input");
        }
        let mut findings = Vec::new();
        let mut score = 0u32;
        let mut granted_dangerous_count = 0usize;
        let mut denied_dangerous_count = 0usize;

        if input.debuggable {
            score += 35;
            add(&mut findings, "debuggable_build", "high", "Debuggable application",
                "The installed app is marked debuggable.".into(),
                "Use a non-debuggable release build for production.");
        }
        if input.allow_backup {
            score += 10;
            add(&mut findings, "backup_enabled", "low", "Application backup enabled",
                "Android backup is enabled for this application.".into(),
                "Review backup rules and exclude sensitive app data.");
        }
        if input.uses_cleartext_traffic {
            score += 25;
            add(&mut findings, "cleartext_traffic", "high", "Cleartext network traffic allowed",
                "The app manifest allows unencrypted HTTP traffic.".into(),
                "Disable cleartext traffic and use HTTPS.");
        }
        if input.target_sdk < 29 {
            score += 20;
            add(&mut findings, "old_target_sdk", "medium", "Outdated target SDK",
                format!("The app targets SDK {}.", input.target_sdk),
                "Raise targetSdkVersion and test modern Android behavior.");
        }

        for permission in &input.permissions {
            if permission.name.len() > 255 || permission.name.bytes().any(|b| b.is_ascii_control()) {
                return Err("invalid_permission_name");
            }
            if let Some((severity, title, recommendation)) = dangerous_permission(&permission.name) {
                if permission.granted {
                    granted_dangerous_count += 1;
                    let weight = if severity == "high" { 12 } else { 7 };
                    score += weight;
                    add(&mut findings, &format!("granted_{}", permission.name.rsplit('.').next().unwrap_or("permission").to_lowercase()),
                        severity, title,
                        format!("{} is currently granted to this app.", permission.name),
                        recommendation);
                } else {
                    denied_dangerous_count += 1;
                }
            }
        }

        score = score.min(100);
        let risk_level = if score >= 60 { "high" } else if score >= 25 { "medium" } else if score > 0 { "low" } else { "good" };
        Ok(AuditReport {
            schema_version: 1,
            package_name: input.package_name,
            security_patch: input.security_patch,
            generated_at_ms: now_ms,
            risk_score: score,
            risk_level,
            permission_count: input.permissions.len(),
            granted_dangerous_count,
            denied_dangerous_count,
            findings,
        })
    })();

    match result {
        Ok(report) => serde_json::json!({"ok": true, "report": report}).to_string(),
        Err(error) => serde_json::json!({"ok": false, "error": error}).to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(extra: &str) -> String {
        format!(r#"{{"package_name":"com.example.app","target_sdk":35,"debuggable":false,"allow_backup":false,"uses_cleartext_traffic":false,"security_patch":"2026-10-01","permissions":[] {extra}}}"#)
    }

    #[test]
    fn clean_app_has_good_rating() {
        let out: serde_json::Value = serde_json::from_str(&audit(&input(""), 10)).unwrap();
        assert_eq!(out["report"]["risk_level"], "good");
        assert_eq!(out["report"]["risk_score"], 0);
    }

    #[test]
    fn flags_debuggable_and_cleartext() {
        let raw = r#"{"package_name":"com.example.app","target_sdk":35,"debuggable":true,"allow_backup":false,"uses_cleartext_traffic":true,"security_patch":"","permissions":[]}"#;
        let out: serde_json::Value = serde_json::from_str(&audit(raw, 10)).unwrap();
        assert_eq!(out["report"]["risk_score"], 60);
        assert_eq!(out["report"]["risk_level"], "high");
    }

    #[test]
    fn counts_granted_and_denied_dangerous_permissions() {
        let raw = r#"{"package_name":"com.example.app","target_sdk":35,"debuggable":false,"allow_backup":false,"uses_cleartext_traffic":false,"security_patch":"","permissions":[{"name":"android.permission.CAMERA","granted":true},{"name":"android.permission.READ_CONTACTS","granted":false}]}"#;
        let out: serde_json::Value = serde_json::from_str(&audit(raw, 10)).unwrap();
        assert_eq!(out["report"]["granted_dangerous_count"], 1);
        assert_eq!(out["report"]["denied_dangerous_count"], 1);
        assert_eq!(out["report"]["risk_score"], 7);
    }

    #[test]
    fn old_target_sdk_adds_finding() {
        let raw = r#"{"package_name":"com.example.app","target_sdk":28,"debuggable":false,"allow_backup":false,"uses_cleartext_traffic":false,"security_patch":"","permissions":[]}"#;
        let out: serde_json::Value = serde_json::from_str(&audit(raw, 10)).unwrap();
        assert_eq!(out["report"]["risk_score"], 20);
    }

    #[test]
    fn rejects_malformed_input() {
        let out: serde_json::Value = serde_json::from_str(&audit("{}", 10)).unwrap();
        assert_eq!(out["ok"], false);
    }

    #[test]
    fn rejects_negative_timestamp() {
        let out: serde_json::Value = serde_json::from_str(&audit(&input(""), -1)).unwrap();
        assert_eq!(out["ok"], false);
    }

    #[test]
    fn rejects_too_many_permissions() {
        let permissions = (0..257).map(|_| r#"{"name":"android.permission.CAMERA","granted":false}"#).collect::<Vec<_>>().join(",");
        let raw = format!(r#"{{"package_name":"com.example.app","target_sdk":35,"debuggable":false,"allow_backup":false,"uses_cleartext_traffic":false,"security_patch":"","permissions":[{}]}}"#, permissions);
        let out: serde_json::Value = serde_json::from_str(&audit(&raw, 10)).unwrap();
        assert_eq!(out["ok"], false);
    }

    #[test]
    fn unknown_permissions_are_not_misclassified() {
        let raw = r#"{"package_name":"com.example.app","target_sdk":35,"debuggable":false,"allow_backup":false,"uses_cleartext_traffic":false,"security_patch":"","permissions":[{"name":"com.example.CUSTOM","granted":true}]}"#;
        let out: serde_json::Value = serde_json::from_str(&audit(raw, 10)).unwrap();
        assert_eq!(out["report"]["risk_score"], 0);
    }
}
