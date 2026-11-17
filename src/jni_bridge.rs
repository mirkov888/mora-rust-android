//! Checked JNI exports.
#![allow(non_snake_case)]

use jni::objects::{JClass, JString};
use jni::sys::{jboolean, jint, jlong, jstring, JNI_FALSE, JNI_TRUE};
use jni::JNIEnv;
use std::ptr;

fn read(env: &mut JNIEnv<'_>, value: &JString<'_>) -> Option<String> {
    env.get_string(value)
        .ok()
        .map(|s| s.to_string_lossy().into_owned())
}

#[no_mangle]
pub extern "system" fn Java_com_mora_rustandroid_NativeCore_nativeCoreApiVersion(
    _: JNIEnv<'_>,
    _: JClass<'_>,
) -> jint {
    crate::mora_core_api_version() as jint
}

#[no_mangle]
pub extern "system" fn Java_com_mora_rustandroid_NativeCore_nativeCoreCapabilities(
    _: JNIEnv<'_>,
    _: JClass<'_>,
) -> jint {
    crate::mora_core_capabilities() as jint
}

#[no_mangle]
pub extern "system" fn Java_com_mora_rustandroid_NativeCore_nativeCoreDescriptor(
    mut e: JNIEnv<'_>,
    _: JClass<'_>,
) -> jstring {
    e.new_string(crate::core_api::descriptor())
        .map(|s| s.into_raw())
        .unwrap_or(ptr::null_mut())
}

#[no_mangle]
pub extern "system" fn Java_com_mora_rustandroid_NativeCore_nativeCoreDispatch(
    mut e: JNIEnv<'_>,
    _: JClass<'_>,
    request: JString<'_>,
) -> jstring {
    let Some(request) = read(&mut e, &request) else {
        return ptr::null_mut();
    };
    e.new_string(crate::core_api::dispatch(&request))
        .map(|s| s.into_raw())
        .unwrap_or(ptr::null_mut())
}

#[no_mangle]
pub extern "system" fn Java_com_mora_rustandroid_NativeCore_nativeAdd(
    _: JNIEnv<'_>,
    _: JClass<'_>,
    a: jint,
    b: jint,
) -> jint {
    crate::mora_add_i32(a, b)
}

#[no_mangle]
pub extern "system" fn Java_com_mora_rustandroid_NativeCore_nativeNormalizeProfileLabel(
    mut e: JNIEnv<'_>,
    _: JClass<'_>,
    s: JString<'_>,
) -> jstring {
    let Some(v) = read(&mut e, &s) else {
        return ptr::null_mut();
    };
    match crate::identity::normalize_profile_label(&v) {
        Ok(v) => e
            .new_string(v)
            .map(|s| s.into_raw())
            .unwrap_or(ptr::null_mut()),
        Err(_) => ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "system" fn Java_com_mora_rustandroid_NativeCore_nativeIsValidProfileId(
    mut e: JNIEnv<'_>,
    _: JClass<'_>,
    s: JString<'_>,
) -> jboolean {
    if read(&mut e, &s).is_some_and(|v| crate::profiles::is_valid_profile_id(&v)) {
        JNI_TRUE
    } else {
        JNI_FALSE
    }
}

#[no_mangle]
pub extern "system" fn Java_com_mora_rustandroid_NativeCore_nativeIsSafeText(
    mut e: JNIEnv<'_>,
    _: JClass<'_>,
    s: JString<'_>,
    max: jint,
) -> jboolean {
    if max >= 0 && read(&mut e, &s).is_some_and(|v| crate::security::is_safe_text(&v, max as usize))
    {
        JNI_TRUE
    } else {
        JNI_FALSE
    }
}

#[no_mangle]
pub extern "system" fn Java_com_mora_rustandroid_NativeCore_nativeProfileOperation(
    mut e: JNIEnv<'_>,
    _: JClass<'_>,
    store: JString<'_>,
    op: JString<'_>,
    id: JString<'_>,
    label: JString<'_>,
    time: jlong,
) -> jstring {
    let (Some(store), Some(op), Some(id), Some(label)) = (
        read(&mut e, &store),
        read(&mut e, &op),
        read(&mut e, &id),
        read(&mut e, &label),
    ) else {
        return ptr::null_mut();
    };
    let out = crate::profile_store::operate(&store, &op, &id, &label, time);
    e.new_string(out)
        .map(|s| s.into_raw())
        .unwrap_or(ptr::null_mut())
}

#[no_mangle]
pub extern "system" fn Java_com_mora_rustandroid_NativeCore_nativeSystemConfigOperation(
    mut e: JNIEnv<'_>,
    _: JClass<'_>,
    config: JString<'_>,
    operation: JString<'_>,
    key: JString<'_>,
    value: JString<'_>,
    time: jlong,
) -> jstring {
    let (Some(config), Some(operation), Some(key), Some(value)) = (
        read(&mut e, &config),
        read(&mut e, &operation),
        read(&mut e, &key),
        read(&mut e, &value),
    ) else {
        return ptr::null_mut();
    };
    let out = crate::system_config::operate(&config, &operation, &key, &value, time);
    e.new_string(out)
        .map(|s| s.into_raw())
        .unwrap_or(ptr::null_mut())
}

#[no_mangle]
pub extern "system" fn Java_com_mora_rustandroid_NativeCore_nativeRuntimeOperation(
    mut e: JNIEnv<'_>,
    _: JClass<'_>,
    runtime: JString<'_>,
    operation: JString<'_>,
    id: JString<'_>,
    time: jlong,
) -> jstring {
    let (Some(runtime), Some(operation), Some(id)) = (
        read(&mut e, &runtime),
        read(&mut e, &operation),
        read(&mut e, &id),
    ) else {
        return ptr::null_mut();
    };
    let out = crate::runtime_manager::operate(&runtime, &operation, &id, time);
    e.new_string(out)
        .map(|s| s.into_raw())
        .unwrap_or(ptr::null_mut())
}

#[no_mangle]
pub extern "system" fn Java_com_mora_rustandroid_NativeCore_nativeEventLogOperation(
    mut e: JNIEnv<'_>,
    _: JClass<'_>,
    log: JString<'_>,
    operation: JString<'_>,
    category: JString<'_>,
    action: JString<'_>,
    detail: JString<'_>,
    time: jlong,
) -> jstring {
    let (Some(log), Some(operation), Some(category), Some(action), Some(detail)) = (
        read(&mut e, &log),
        read(&mut e, &operation),
        read(&mut e, &category),
        read(&mut e, &action),
        read(&mut e, &detail),
    ) else {
        return ptr::null_mut();
    };
    let out = crate::event_logger::operate(&log, &operation, &category, &action, &detail, time);
    e.new_string(out)
        .map(|s| s.into_raw())
        .unwrap_or(ptr::null_mut())
}

#[no_mangle]
pub extern "system" fn Java_com_mora_rustandroid_NativeCore_nativeNetworkMonitorOperation(
    mut e: JNIEnv<'_>,
    _: JClass<'_>,
    store: JString<'_>,
    operation: JString<'_>,
    snapshot: JString<'_>,
    time: jlong,
) -> jstring {
    let (Some(store), Some(operation), Some(snapshot)) = (
        read(&mut e, &store),
        read(&mut e, &operation),
        read(&mut e, &snapshot),
    ) else {
        return ptr::null_mut();
    };
    let out = crate::network_monitor::operate(&store, &operation, &snapshot, time);
    e.new_string(out)
        .map(|s| s.into_raw())
        .unwrap_or(ptr::null_mut())
}

#[no_mangle]
pub extern "system" fn Java_com_mora_rustandroid_NativeCore_nativeSecurityAudit(
    mut e: JNIEnv<'_>,
    _: JClass<'_>,
    input: JString<'_>,
    time: jlong,
) -> jstring {
    let Some(input) = read(&mut e, &input) else {
        return ptr::null_mut();
    };
    let out = crate::security_auditor::audit(&input, time);
    e.new_string(out)
        .map(|s| s.into_raw())
        .unwrap_or(ptr::null_mut())
}

#[cfg(test)]
mod tests {
    #[test]
    fn api_v9() {
        assert_eq!(crate::mora_core_api_version(), 9);
    }
}
