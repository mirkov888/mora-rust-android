//! JNI exports used by the Kotlin integration.
//!
//! String conversion is handled by the jni crate so Java strings are read and
//! returned with checked JNI operations rather than raw pointer assumptions.
#![allow(non_snake_case)]

use jni::objects::{JClass, JString};
use jni::sys::{jboolean, jint, jstring, JNI_FALSE, JNI_TRUE};
use jni::JNIEnv;
use std::ptr;

#[no_mangle]
pub extern "system" fn Java_com_mora_rustandroid_NativeCore_nativeCoreApiVersion(
    _env: JNIEnv<'_>,
    _class: JClass<'_>,
) -> jint {
    crate::mora_core_api_version() as jint
}

#[no_mangle]
pub extern "system" fn Java_com_mora_rustandroid_NativeCore_nativeCoreCapabilities(
    _env: JNIEnv<'_>,
    _class: JClass<'_>,
) -> jint {
    crate::mora_core_capabilities() as jint
}

#[no_mangle]
pub extern "system" fn Java_com_mora_rustandroid_NativeCore_nativeAdd(
    _env: JNIEnv<'_>,
    _class: JClass<'_>,
    left: jint,
    right: jint,
) -> jint {
    crate::mora_add_i32(left, right)
}

/// Returns a normalized label, or null when the input is invalid.
#[no_mangle]
pub extern "system" fn Java_com_mora_rustandroid_NativeCore_nativeNormalizeProfileLabel(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    input: JString<'_>,
) -> jstring {
    let value = match env.get_string(&input) {
        Ok(value) => value.to_string_lossy().into_owned(),
        Err(_) => return ptr::null_mut(),
    };

    match crate::identity::normalize_profile_label(&value) {
        Ok(normalized) => env
            .new_string(normalized)
            .map(|value| value.into_raw())
            .unwrap_or(ptr::null_mut()),
        Err(_) => ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "system" fn Java_com_mora_rustandroid_NativeCore_nativeIsValidProfileId(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    input: JString<'_>,
) -> jboolean {
    let value = match env.get_string(&input) {
        Ok(value) => value.to_string_lossy().into_owned(),
        Err(_) => return JNI_FALSE,
    };

    if crate::profiles::is_valid_profile_id(&value) {
        JNI_TRUE
    } else {
        JNI_FALSE
    }
}

#[no_mangle]
pub extern "system" fn Java_com_mora_rustandroid_NativeCore_nativeIsSafeText(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    input: JString<'_>,
    max_bytes: jint,
) -> jboolean {
    if max_bytes < 0 {
        return JNI_FALSE;
    }

    let value = match env.get_string(&input) {
        Ok(value) => value.to_string_lossy().into_owned(),
        Err(_) => return JNI_FALSE,
    };

    if crate::security::is_safe_text(&value, max_bytes as usize) {
        JNI_TRUE
    } else {
        JNI_FALSE
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn api_version_matches_core() {
        assert_eq!(crate::mora_core_api_version(), 2);
    }

    #[test]
    fn capability_mask_contains_four_features() {
        assert_eq!(crate::mora_core_capabilities(), 0b1111);
    }

    #[test]
    fn native_add_uses_core_function() {
        assert_eq!(crate::mora_add_i32(19, 23), 42);
    }
}
