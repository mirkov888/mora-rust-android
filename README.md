# Mora Rust Android Core

An independent Rust native library intended to be called from Android Kotlin code through JNI.

## Current scope

- Rust native library crate (cdylib + rlib)
- C ABI smoke-test functions
- JNI exports for API version and integer-add smoke tests
- Kotlin bridge and JUnit instrumentation-test example in `android-example/`
- A minimal installable Android demo app in `android-demo/`
- Platform-independent profile-label and input validation helpers
- GitHub Actions CI for tests, linting, Android native builds, and demo APK packaging

This repository is a native Rust core plus a separate demo Android app. It is not Mora Shield and does not replace or modify any existing application.

## Android ABIs

- `arm64-v8a` (primary modern Android target)
- `armeabi-v7a`
- `x86_64`

## Android demo app

The `android-demo/` directory contains a minimal Kotlin Android application. It loads `libmora_rust_android.so` through JNI and provides a **Jalankan Rust JNI Test** button.

The test checks:
- Rust native API version returns `1`
- Rust integer addition returns `20 + 22 = 42`

### Build output

On each push to `main`, GitHub Actions:
1. Runs Rust formatting, Clippy, and unit tests.
2. Builds the Rust shared library for all three Android ABIs.
3. Packages the libraries into the Android demo app.
4. Builds and uploads `app-debug.apk` as the `mora-rust-android-demo-apk` workflow artifact.

Open the repository's **Actions** tab, select the latest successful **Rust Android CI** run, then download the `mora-rust-android-demo-apk` artifact. Extract the ZIP to obtain `app-debug.apk`.

Install on a connected Android device with ADB:

```bash
adb install -r app-debug.apk
```

Open **Mora Rust Core Demo** and tap **Jalankan Rust JNI Test**. A successful result displays API v1 and `20 + 22 = 42`.

## Kotlin/JNI integration in another app

1. Build the native library for the ABI(s) supported by the Android app.
2. Place `libmora_rust_android.so` in the matching `app/src/main/jniLibs/<abi>/` directory.
3. Copy `android-example/NativeCore.kt` into the app's Kotlin source tree. Keep package name `com.mora.rustandroid`, or rename both the Kotlin package and JNI export symbols in `src/jni_bridge.rs` together.
4. Call the bridge from Kotlin:

```kotlin
val version = NativeCore.nativeCoreApiVersion()
val sum = NativeCore.nativeAdd(20, 22) // 42
```

5. Add `android-example/NativeCoreSmokeTest.kt` to the app's Android instrumented-test source set and run it on a device/emulator with a matching ABI.

## Local Rust development

Install Rust stable, Android SDK/NDK, and Cargo NDK, then run:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo ndk -t arm64-v8a -o ./jniLibs build --release
```

## Safety and scope

Current helpers validate app-owned profile labels and input data. They do not spoof hardware identifiers, bypass Android security controls, or modify Play Integrity / hardware attestation.
