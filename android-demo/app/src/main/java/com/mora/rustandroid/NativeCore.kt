package com.mora.rustandroid

/** Kotlin JNI bridge to the Rust native library. */
object NativeCore {
    init {
        System.loadLibrary("mora_rust_android")
    }

    @JvmStatic external fun nativeCoreApiVersion(): Int
    @JvmStatic external fun nativeCoreCapabilities(): Int
    @JvmStatic external fun nativeAdd(left: Int, right: Int): Int
    @JvmStatic external fun nativeNormalizeProfileLabel(input: String): String?
    @JvmStatic external fun nativeIsValidProfileId(input: String): Boolean
    @JvmStatic external fun nativeIsSafeText(input: String, maxBytes: Int): Boolean
}
