//! JNI exports used by the Kotlin integration example.
//!
//! JNI references are opaque pointers because these functions do not inspect
//! the Java environment or manipulate Java objects. The argument and return
//! ABI matches JNI pointer and 32-bit integer types.
use core::ffi::c_void;

/// JNI entry point for NativeCore.nativeCoreApiVersion().
#[no_mangle]
pub extern "system" fn Java_com_mora_rustandroid_NativeCore_nativeCoreApiVersion(_env: *mut c_void, _class: *mut c_void) -> i32 {
    crate::mora_core_api_version() as i32
}

/// JNI entry point for NativeCore.nativeAdd(Int, Int).
#[no_mangle]
pub extern "system" fn Java_com_mora_rustandroid_NativeCore_nativeAdd(_env: *mut c_void, _class: *mut c_void, left: i32, right: i32) -> i32 {
    crate::mora_add_i32(left, right)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn jni_api_version_matches_core() {
        assert_eq!(Java_com_mora_rustandroid_NativeCore_nativeCoreApiVersion(core::ptr::null_mut(), core::ptr::null_mut()), 1);
    }
    #[test]
    fn jni_add_calls_core_function() {
        assert_eq!(Java_com_mora_rustandroid_NativeCore_nativeAdd(core::ptr::null_mut(), core::ptr::null_mut(), 19, 23), 42);
    }
}
