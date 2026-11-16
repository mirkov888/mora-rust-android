//! Profile identifier validation helpers.

/// Returns true when an ID contains only ASCII letters, digits, '-' or '_'
/// and has a length between 1 and 64 bytes.
pub fn is_valid_profile_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_simple_ids() {
        assert!(is_valid_profile_id("daily_profile-01"));
    }

    #[test]
    fn rejects_unsafe_or_empty_ids() {
        assert!(!is_valid_profile_id(""));
        assert!(!is_valid_profile_id("../profile"));
        assert!(!is_valid_profile_id("profile name"));
    }
}
