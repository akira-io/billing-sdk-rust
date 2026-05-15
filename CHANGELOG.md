# Changelog

All notable changes to `akira-billing` are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the crate adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.4] — 2026-05-15

### Changed

- Republish of 0.1.3 contents under a fresh version. The v0.1.3 tag
  was never uploaded to crates.io because the crate had no publish
  workflow at the time; this release exists to drive the new
  Trusted Publishing pipeline end-to-end. No code or API changes.

[0.1.4]: https://github.com/akira-io/billing-sdk-rust/releases/tag/v0.1.4

## [0.1.3] — 2026-05-15

### Added

- New `license` module with helpers for `offline_snapshot` products:
  `decode_license`, `verify_license` (Ed25519 via `ed25519-dalek`),
  `compute_remaining`, `is_expired`, `is_in_grace`, `can_use_update`,
  `period_reset_at`. `RemainingValue` enum distinguishes finite
  counters from unlimited features.
- `Client::license_sync_usage` POST `/api/licenses/sync-usage` to
  apply local usage deltas and receive a re-signed snapshot.
- `UsagePayload.count: Option<u32>` for variable-count realtime
  tracking (e.g. AI token usage).
- Types: `LicensingMode`, `UsagePeriod`, `UsageFeatureState`,
  `LicenseSnapshotPayload`, `LicenseSyncUsagePayload`,
  `LicenseSyncUsageResponse`.

### Dependencies

- Added `base64 = 0.22`, `chrono = 0.4` (with `serde`),
  `ed25519-dalek = 2`.

## [0.1.2] — 2026-05-15

### Added

- `UsagePayload` carries optional `platform`, `device_type`, and
  `app_version` so the server can record device metadata alongside
  the usage counter. Authenticated and anonymous endpoints both
  accept the new fields.

[0.1.2]: https://github.com/akira-io/billing-sdk-rust/releases/tag/v0.1.2

## [0.1.1] — 2026-05-15

### Added

- `track_anonymous_usage(payload)` — `POST /api/v1/usage/anonymous`. HMAC-only
  endpoint (no bearer) for metering devices that have not yet authenticated.
  The server applies the limits defined on the product's `anonymous_plan`.

[0.1.1]: https://github.com/akira-io/billing-sdk-rust/releases/tag/v0.1.1

## [0.1.0] — 2026-05-15

First public release. Async Rust client for the Akira Billing API. Mirrors the
Go and JS SDKs and shares the same HMAC wire protocol.

### Client surface

- OTP login: `request_otp`, `verify_otp` (auto-stores the bearer).
- Customer profile: `customer_me`.
- License lifecycle: `license_check`, `license_activate`, `license_refresh`.
  Activation and refresh return `SignedLicense` (key_id, algorithm, base64
  payload, base64 signature, valid_until) so clients can verify the envelope
  offline with the matching Ed25519 public key.
- Entitlements snapshot: `entitlements`.
- Stripe billing portal short-lived URL: `billing_portal(return_url)`.
- Usage tracking: `track_usage` with `check` / `increment` actions.
- Trials: `start_trial(plan_key)`.
- Plans listing: `plans()`.
- Downloads: `latest_release(channel)`, `issue_download(channel, platform)`,
  `complete_download(beacon_url)`.
- Unsigned key set fetch: `public_license_keys()` for build-time embedding of
  the Ed25519 verification keys.

### Tooling

- HMAC signing helpers exported under `signature::{canonical, sign, new_nonce}`
  so callers can sign requests for endpoints the SDK has not yet typed.
- `Error::Api { status, code }` carries the server error payload.
- Async via `reqwest` + `tokio`. TLS via `rustls`. Zero unsafe code.
- Shared signature test vectors against the Go SDK ensure wire-level parity.

[0.1.0]: https://github.com/akira-io/billing-sdk-rust/releases/tag/v0.1.0
