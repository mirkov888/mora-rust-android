package com.mora.rustandroid

/** Minimal Kotlin bridge for the Mora Rust Android native library. */
object NativeCore {
    init {
        System.loadLibrary("mora_rust_android")
    }

    @JvmStatic external fun nativeCoreApiVersion(): Int
    @JvmStatic external fun nativeAdd(left: Int, right: Int): Int
}
