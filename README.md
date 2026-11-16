# Mora Rust Android Core

An independent Rust library starter intended to be compiled for Android and
later called from Kotlin through JNI or UniFFI.

## Current scope

- Rust native library crate (cdylib + rlib)
- Stable Rust toolchain with formatting and lint components
- Small C ABI smoke-test functions
- Platform-independent profile-label and input validation helpers
- GitHub Actions CI for tests, linting, and Android native builds

This repository is a starter core, not an Android APK and not yet integrated
with any Android application.

## Android ABIs

- arm64-v8a (primary modern Android target)
- armeabi-v7a
- x86_64

## Local development

Install Rust stable, Android SDK/NDK, and Cargo NDK, then run:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo ndk -t arm64-v8a -o ./jniLibs build --release
```

## GitHub Actions

Pushes and pull requests to main run tests and build the native library for
all configured Android ABIs. The resulting .so files are uploaded as workflow
artifacts.

## Next steps

1. Confirm CI builds successfully for all ABIs.
2. Add a Kotlin/JNI or UniFFI integration example.
3. Add Android instrumentation tests in a separate integration project.
4. Only then consider integrating this library into an existing app.

## Safety and scope

Current helpers validate app-owned profile labels and input data. They do
not spoof device identifiers, bypass Android security controls, or modify
Play Integrity / hardware attestation.
