//! Mora Rust Android Core starter library.
//!
//! This crate is independent from any Android application.
//! Kotlin/JNI integration will be added in a later, separately tested step.

pub mod identity;
pub mod profiles;
pub mod security;

/// Returns the native core API version.
#[no_mangle]
pub extern "C" fn mora_core_api_version() -> u32 {
    1
}

/// Adds two 32-bit integers using explicitly wrapping arithmetic.
#[no_mangle]
pub extern "C" fn mora_add_i32(left: i32, right: i32) -> i32 {
    left.wrapping_add(right)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_api_version() {
        assert_eq!(mora_core_api_version(), 1);
    }

    #[test]
    fn adds_values() {
        assert_eq!(mora_add_i32(20, 22), 42);
    }

    #[test]
    fn addition_wraps_explicitly() {
        assert_eq!(mora_add_i32(i32::MAX, 1), i32::MIN);
    }
}
