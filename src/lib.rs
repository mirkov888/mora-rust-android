//! Mora Rust Android Core.
pub mod identity;
pub mod jni_bridge;
pub mod profile_store;
pub mod profiles;
pub mod runtime_manager;
pub mod security;
pub mod system_config;

pub const CAP_INTEGER_ARITHMETIC: u32 = 1;
pub const CAP_PROFILE_LABELS: u32 = 2;
pub const CAP_PROFILE_IDS: u32 = 4;
pub const CAP_SAFE_TEXT: u32 = 8;
pub const CAP_PROFILE_STORE: u32 = 16;
pub const CAP_SYSTEM_CONFIG: u32 = 32;
pub const CAP_RUNTIME_MANAGER: u32 = 64;

#[no_mangle]
pub extern "C" fn mora_core_api_version() -> u32 {
    5
}

#[no_mangle]
pub extern "C" fn mora_core_capabilities() -> u32 {
    127
}

#[no_mangle]
pub extern "C" fn mora_add_i32(left: i32, right: i32) -> i32 {
    left.wrapping_add(right)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_five() {
        assert_eq!(mora_core_api_version(), 5);
    }

    #[test]
    fn capabilities_v5() {
        assert_eq!(mora_core_capabilities(), 127);
    }

    #[test]
    fn adds() {
        assert_eq!(mora_add_i32(20, 22), 42);
    }

    #[test]
    fn wraps() {
        assert_eq!(mora_add_i32(i32::MAX, 1), i32::MIN);
    }
}
