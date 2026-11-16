//! Mora Rust Android Core.
//!
//! This crate exposes a small C ABI and JNI entry points for Kotlin.

pub mod identity;
pub mod jni_bridge;
pub mod profiles;
pub mod security;

/// Returns the native core API version.
#[no_mangle]
pub extern "C" fn mora_core_api_version() -> u32 { 1 }

/// Adds two 32-bit integers using explicitly wrapping arithmetic.
#[no_mangle]
pub extern "C" fn mora_add_i32(left: i32, right: i32) -> i32 { left.wrapping_add(right) }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn exposes_api_version() { assert_eq!(mora_core_api_version(), 1); }
    #[test] fn adds_values() { assert_eq!(mora_add_i32(20, 22), 42); }
    #[test] fn addition_wraps_explicitly() { assert_eq!(mora_add_i32(i32::MAX, 1), i32::MIN); }
}
