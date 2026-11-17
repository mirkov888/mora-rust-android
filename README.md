# Mora Rust Android Core

Independent Rust native library with an Android Kotlin demo. This is not Mora Shield and does not modify any existing app.

## Stage 7
- Native API v6 and capability bitmask `0xff`.
- Rust Event Logger records app-owned profile, configuration, runtime, and system events.
- Each event contains a monotonic event ID, timestamp (milliseconds), category, action, and detail.
- Operations: `list`, `record`, and `clear`.
- Android persists the log in app-private `filesDir/activity_events.json`.
- Log is bounded to 500 records; oldest events are discarded first.
- Input is validated; unknown categories, control characters, invalid timestamps, and malformed stores are rejected.
- This logger records only events explicitly sent by this app. It does not inspect other apps or system-wide activity.

## Stage 6
- Native API v5 and capability bitmask `0x7f`.
- Runtime Manager tracks three app-owned logical services: `profile_store`, `config_store`, and `diagnostics`.
- Operations: `list`, `start`, `stop`, `enable`, and `disable`.
- Runtime state persists in app-private `filesDir/runtime_services.json`.
- Disabled services cannot be started; disabling a running service stops its logical runtime state.
- This is an app-owned state registry, not an Android system-service controller or process launcher.

## Stage 5
- Native API v4 and capability bitmask `0x3f`.
- Rust System Configuration Manager with typed validation and JSON operations (`get`, `set`, `reset`).
- Supported keys: `log_level` (`error`, `warn`, `info`, `debug`), `network_mode` (`auto`, `wifi_only`, `offline`), `background_sync` (boolean), `cache_limit_mb` (64–4096), and `telemetry_enabled` (boolean, defaults off).
- Android persists configuration in app-private `filesDir/system_config.json` and exposes controls in the demo UI.
- Unknown keys, invalid values, malformed JSON, unsupported schema versions, and invalid timestamps are rejected.
- No Android global settings, root operations, hardware identifier changes, or security bypasses.

## Test
Download the `mora-rust-android-demo-apk` artifact from the latest successful Actions run, extract `app-debug.apk`, install it, and test the UI. Restart the app to verify local persistence.

## Local checks
```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

The core handles app-owned data only. It does not spoof hardware identifiers or bypass Android security, Play Integrity, or hardware attestation.
