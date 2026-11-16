//! Platform-independent identity-label helpers.
//! These helpers validate app data; they do not alter device identifiers.

/// Trims and validates a human-readable profile label.
pub fn normalize_profile_label(input: &str) -> Result<String, &'static str> {
    let value = input.trim();

    if value.is_empty() {
        return Err("profile label cannot be empty");
    }
    if value.chars().count() > 64 {
        return Err("profile label must be 64 characters or fewer");
    }
    if value.chars().any(char::is_control) {
        return Err("profile label cannot contain control characters");
    }

    Ok(value.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_label() {
        assert_eq!(normalize_profile_label("  Daily  "), Ok("Daily".to_owned()));
    }

    #[test]
    fn rejects_empty_label() {
        assert!(normalize_profile_label("   ").is_err());
    }
}
