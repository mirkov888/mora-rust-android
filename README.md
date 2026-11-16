# Mora Rust Android Core

An independent Rust native library intended to be called from Android Kotlin code through JNI.

## Current scope — Stage 3

- Rust native library crate (`cdylib` + `rlib`)
- Versioned native API v2 and capability bitmask
- JNI bridge with checked Java/Kotlin string conversion
- Integer arithmetic with explicit wrapping behavior
- Profile-label normalization and validation
- Profile ID validation
- Safe-text validation with caller-provided byte limits
- Android demo app exercising all JNI features
- GitHub Actions CI for Rust tests, linting, Android native builds, and APK packaging

This repository is a native Rust core plus a separate demo Android app. It is not Mora Shield and does not replace or modify any existing application.

## Android ABIs

- `arm64-v8a`
- `armeabi-v7a`
- `x86_64`

## Native API v2

| API | Purpose |
| --- | --- |
| `mora_core_api_version()` | Returns API version 2 |
| `mora_core_capabilities()` | Returns supported-feature bitmask |
| `mora_add_i32(left, right)` | Adds signed 32-bit integers with wrapping semantics |

Capability bits: `0x01` integer arithmetic, `0x02` profile-label helpers, `0x04` profile-ID validation, `0x08` safe-text validation.

## Kotlin JNI methods

- `nativeCoreApiVersion()`
- `nativeCoreCapabilities()`
- `nativeAdd(left, right)`
- `nativeNormalizeProfileLabel(input)` — returns normalized text or null for invalid input
- `nativeIsValidProfileId(input)`
- `nativeIsSafeText(input, maxBytes)`

JNI string conversion uses the `jni` crate and checked JNI calls. Conversion failures return safe failure values.

## Android demo app

The `android-demo/` app provides a **Jalankan Semua Rust Tests** button. The on-device smoke test checks API version, capability flags, integer addition, profile-label normalization, profile-ID validation, and safe-text validation.

On each push to `main`, GitHub Actions runs formatting, Clippy and Rust tests; builds native libraries for all three ABIs; packages them into the demo; then uploads `app-debug.apk` as the `mora-rust-android-demo-apk` artifact.

Open the repository Actions tab, select the latest successful Rust Android CI run, and download `mora-rust-android-demo-apk`. Extract the ZIP to obtain `app-debug.apk`.

Install with ADB:

```bash
adb install -r app-debug.apk
```

Open Mora Rust Core Demo and tap **Jalankan Semua Rust Tests**. A successful result displays API v2 and nine passing checks.

## Local Rust development

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo ndk -t arm64-v8a -o ./jniLibs build --release
```

## Safety and scope

These helpers validate app-owned labels and text. They do not read, spoof, or modify hardware identifiers, bypass Android security controls, or modify Play Integrity / hardware attestation.