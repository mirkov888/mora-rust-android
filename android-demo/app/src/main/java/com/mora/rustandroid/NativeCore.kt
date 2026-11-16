package com.mora.rustandroid

/** Kotlin JNI bridge to the Rust native library. */
object NativeCore {
    init {
        System.loadLibrary("mora_rust_android")
    }

    @JvmStatic external fun nativeCoreApiVersion(): Int
    @JvmStatic external fun nativeAdd(left: Int, right: Int): Int
}
