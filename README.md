# Mora Rust Android Core

An independent Rust native library intended to be called from Android Kotlin code through JNI.

## Current scope

- Rust native library crate (cdylib + rlib)
- C ABI smoke-test functions
- JNI exports for API version and integer-add smoke tests
- Kotlin bridge and JUnit instrumentation-test example in `android-example/`
- Platform-independent profile-label and input validation helpers
- GitHub Actions CI for tests, linting, and Android native builds

This repository is a native library and integration example, not a complete Android APK or a replacement for an existing Android application.

## Android ABIs

- `arm64-v8a` (primary modern Android target)
- `armeabi-v7a`
- `x86_64`

## Kotlin/JNI integration

1. Build the native library for the ABI(s) supported by the Android app. GitHub Actions uploads one artifact per ABI.
2. Extract the resulting `libmora_rust_android.so` file into the Android app's matching JNI library directory, for example `app/src/main/jniLibs/arm64-v8a/libmora_rust_android.so`.
3. Copy `android-example/NativeCore.kt` into the app's Kotlin source tree. Keep the package name `com.mora.rustandroid`, or rename both the Kotlin package and the JNI export symbols in `src/jni_bridge.rs` together.
4. Call the bridge from Kotlin:

```kotlin
val version = NativeCore.nativeCoreApiVersion()
val sum = NativeCore.nativeAdd(20, 22) // 42
```

5. Add `android-example/NativeCoreSmokeTest.kt` to the app's Android instrumented-test source set and run it on a device/emulator with a matching ABI. This verifies that Android loads the .so and resolves both JNI symbols.

The Rust CI validates the native functions and builds the shared library for all configured ABIs. The Kotlin smoke test is a template for the consuming Android app; it is not run by this Rust-only repository workflow.

## Local Rust development

Install Rust stable, Android SDK/NDK, and Cargo NDK, then run:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo ndk -t arm64-v8a -o ./jniLibs build --release
```

## GitHub Actions

Pushes and pull requests to main run formatting, linting, tests, and native builds for all configured Android ABIs. The resulting .so files are uploaded as workflow artifacts.

## Safety and scope

Current helpers validate app-owned profile labels and input data. They do not spoof hardware identifiers, bypass Android security controls, or modify Play Integrity / hardware attestation.
