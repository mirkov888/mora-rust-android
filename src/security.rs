//! Basic input-safety helpers for the native core.

/// Checks whether text is within a byte limit and contains no ASCII controls.
pub fn is_safe_text(value: &str, max_bytes: usize) -> bool {
    value.len() <= max_bytes && !value.bytes().any(|byte| byte.is_ascii_control())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_regular_text() {
        assert!(is_safe_text("hello Android", 32));
    }

    #[test]
    fn rejects_control_characters_and_oversized_text() {
        assert!(!is_safe_text("hello\nworld", 32));
        assert!(!is_safe_text("too long", 3));
    }
}
