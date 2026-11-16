//! Mora Rust Android Core.
//!
//! This crate exposes a small C ABI and JNI entry points for Kotlin.

pub mod identity;
pub mod jni_bridge;
pub mod profiles;
pub mod security;

/// Capability flags exposed by API v2.
pub const CAP_INTEGER_ARITHMETIC: u32 = 1 << 0;
pub const CAP_PROFILE_LABELS: u32 = 1 << 1;
pub const CAP_PROFILE_IDS: u32 = 1 << 2;
pub const CAP_SAFE_TEXT: u32 = 1 << 3;

/// Returns the native core API version.
#[no_mangle]
pub extern "C" fn mora_core_api_version() -> u32 {
    2
}

/// Returns the bitmask of capabilities supported by this core.
#[no_mangle]
pub extern "C" fn mora_core_capabilities() -> u32 {
    CAP_INTEGER_ARITHMETIC | CAP_PROFILE_LABELS | CAP_PROFILE_IDS | CAP_SAFE_TEXT
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
    fn exposes_api_version_two() {
        assert_eq!(mora_core_api_version(), 2);
    }

    #[test]
    fn exposes_all_v2_capabilities() {
        assert_eq!(mora_core_capabilities(), 0b1111);
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
