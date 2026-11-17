# Mora Rust Android Core

Independent Rust native library with an Android Kotlin demo. This is not Mora Shield and does not modify any existing app.

## Stage 10 — Unified Core API
- Native API v9 and capability bitmask `0x7ff`; bit 10 advertises the unified dispatcher.
- `core_api::descriptor()` reports the contract version, capability mask, modules, operations, and app-scoped data boundaries.
- `core_api::dispatch()` accepts one versioned JSON request and routes it to Profile Store, System Config, Runtime Manager, Event Logger, Network Monitor, or Security Auditor.
- All dispatcher responses share `api_version`, `schema_version`, `request_id`, `module`, `operation`, and `ok`; successful data is under `data`, while failures expose a stable `error`.
- Requests are limited to 256 KiB, require schema version 1, reject invalid timestamps and malformed request tokens, and preserve validation errors from each module.
- Android exposes the unified dispatcher through `NativeCore.nativeCoreDispatch()` and the descriptor through `nativeCoreDescriptor()`. Existing per-module JNI methods remain available for backward compatibility.
- This is a common API contract, not a background service or a new Android permission layer.

### Request contract
A request contains `schema_version`, optional `request_id`, `module`, `operation`, `timestamp_ms`, optional `state_json`, and optional object `params`.

Supported module IDs:
- `profile_store`: `list`, `create`, `update`, `delete`; params `id`, `label`.
- `system_config`: `get`, `set`, `reset`; params `key`, `value`.
- `runtime_manager`: `list`, `start`, `stop`, `enable`, `disable`; param `service_id`.
- `event_logger`: `list`, `record`, `clear`; params `category`, `action`, `detail`.
- `network_monitor`: `list`, `summary`, `record`, `clear`; param `snapshot_json`.
- `security_auditor`: `audit`; param `input_json`.

## Stage 9 — Security & Permission Auditor
- Native API v8 and capability bitmask `0x3ff`.
- App-scoped Security & Permission Auditor evaluates the owning app's requested/granted permission states, debuggable flag, backup flag, cleartext traffic policy, and target SDK.
- Produces a bounded JSON report with risk score, risk level, findings, and remediation guidance.
- Android reads only its own package metadata and permission state, then passes the data to Rust; the last report is stored in app-private `filesDir/security_audit.json`.
- The security patch level is included as metadata, not used as a definitive vulnerability verdict.
- No permission is requested or changed automatically. The auditor does not scan other apps, inspect private app data, claim to certify the whole device, or bypass Android security controls.

## Stage 8 — Network Monitor Core
- Native API v7 and capability bitmask `0x1ff`.
- Network Monitor captures app UID RX/TX byte counters, active transport, connection validation, and metered state.
- Stores up to 200 snapshots in app-private `filesDir/network_snapshots.json`, computes deltas, detects counter resets, and supports list, summary, and clear.
- Android `TrafficStats` reports this app's UID only; it does not provide unrestricted traffic monitoring for other apps.

## Stage 7 — Event & Activity Logger
- Native API v6 and capability bitmask `0xff`.
- Rust Event Logger records app-owned profile, configuration, runtime, and system events.
- Each event contains a monotonic event ID, timestamp (milliseconds), category, action, and detail.
- Operations: `list`, `record`, and `clear`.
- Android persists the log in app-private `filesDir/activity_events.json`.
- Log is bounded to 500 records; oldest events are discarded first.
- Input is validated; unknown categories, control characters, invalid timestamps, and malformed stores are rejected.
- This logger records only events explicitly sent by this app. It does not inspect other apps or system-wide activity.

## Stage 6 — Runtime & Service Manager
- Native API v5 and capability bitmask `0x7f`.
- Runtime Manager tracks three app-owned logical services: `profile_store`, `config_store`, and `diagnostics`.
- Operations: `list`, `start`, `stop`, `enable`, and `disable`.
- Runtime state persists in app-private `filesDir/runtime_services.json`.
- Disabled services cannot be started; disabling a running service stops its logical runtime state.
- This is an app-owned state registry, not an Android system-service controller or process launcher.

## Test
Download the `mora-rust-android-demo-apk` artifact from the latest successful Actions run, extract `app-debug.apk`, install it, and test the UI. Restart the app to verify local persistence. In Diagnostics, run all tests; the Unified Core API descriptor and dispatcher are included.

## Local checks
```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

The core handles app-owned data only. It does not spoof hardware identifiers or bypass Android security, Play Integrity, or hardware attestation.
