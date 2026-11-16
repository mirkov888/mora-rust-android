# Mora Rust Android Core

Independent Rust native library with an Android Kotlin demo. This is not Mora Shield and does not modify any existing app.

## Stage 4
- Native API v3 and capability bitmask.
- Rust profile CRUD engine: create, list, update, delete.
- Validated profile IDs and normalized labels.
- Android persists the JSON document in private app storage at `filesDir/profiles.json`.
- JNI uses checked string conversions.
- CI builds arm64-v8a, armeabi-v7a, and x86_64 APK libraries.

Capability bits: `0x01` arithmetic, `0x02` labels, `0x04` profile IDs, `0x08` safe text, `0x10` profile CRUD.

JNI entry point: `nativeProfileOperation(storeJson, operation, id, label, timestamp)`. Operations are `create`, `list`, `update`, `delete`. Result JSON contains `ok`, `store`, and optional `error`.

## Test
Download the `mora-rust-android-demo-apk` artifact from the latest successful Actions run, extract `app-debug.apk`, then install with `adb install -r app-debug.apk`. Use the profile form to create, view, update, and delete records. Restart the app to verify persistence. Device test target: 12/12.

## Local checks
```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

The profile store handles app-owned data only. It does not spoof hardware identifiers or bypass Android security, Play Integrity, or hardware attestation.
