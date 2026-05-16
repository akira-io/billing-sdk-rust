# Changelog

All notable changes to `akira-billing` are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the crate adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] — 2026-05-16

### Added

- `Gate` struct (`gate.rs`) bundling verify + lifecycle state +
  `compute_remaining` behind a single `check(feature)` call. `require(feature)`
  returns `GateError::Denied(GateDenied)` on failure. Configurable via
  `GateOptions { loader, local_consumption, grace_window, now }`.
- `FeatureAccess` with `allowed`, `has_feature`, `unlimited`, `remaining`,
  `reason`, `plan`, `state`. Pattern match `GateError::Denied(_)` for
  type-safe denial handling.
- `LicenseState` enum (`None|Invalid|Active|Trialing|Grace|Expired`) plus
  `compute_state(payload, grace, now)` and `trial_days_left(payload, now)`.
  Trial detected via feature `__trial=true` or plan key suffix `:trial`.
- `UsageTracker` (`usage.rs`) with `UsageBuffer` async trait (`add`, `drain`,
  `restore`) for buffered counter sync. `track`, `flush`, `start`, `stop`;
  default 5min interval. `MemoryBuffer` reference impl. Requires the tracker
  to be wrapped in `Arc<UsageTracker>` when using `start`.

### Changed

- `Cargo.toml`: added `async-trait`, `futures`; expanded `tokio` features
  (`rt-multi-thread`, `sync`, `time`).

## [0.1.8] — 2026-05-16

### Added

- `Client::github_installation_token(payload)` posts
  `/api/me/github/installation-token` and returns the minted GitHub
  App installation token.
- Types: `GithubInstallationTokenPayload`,
  `GithubInstallationTokenResponse`, `OauthExchangeEntitlement`.

### Changed

- `OauthExchangeCustomer` now exposes `email` and `name`.
- `OauthExchangeResponse` carries `entitlement` plus a
  `requires_plan_selection` flag so clients can branch into a
  plan-picker UI when no free plan auto-grant happened.

[0.1.8]: https://github.com/akira-io/billing-sdk-rust/releases/tag/v0.1.8

## [0.1.7] — 2026-05-15

### Added

- New `oauth` module with `generate_pkce_challenge`,
  `generate_oauth_state`, and `build_oauth_init_url` helpers for the
  Authorization Code + PKCE flow brokered by billing.
- `Client::list_oauth_providers(product)` returns the enabled
  providers + scopes for a product.
- `Client::exchange_oauth_code(payload)` redeems a one-time code for
  a customer access token and stores it on the client.
- Types: `OauthProvider`, `OauthProviderInfo`, `OauthProvidersResponse`,
  `OauthExchangePayload`, `OauthExchangeResponse`, `PkceChallenge`,
  `BuildOauthInitUrl`.

[0.1.7]: https://github.com/akira-io/billing-sdk-rust/releases/tag/v0.1.7

## [0.1.6] — 2026-05-15

### Added

- `LicenseSnapshotPayload.updates_window_days` and
  `LicenseSnapshotPayload.offline_grace_days` carried through from the
  plan-level overrides on the billing server.

### Changed

- `can_use_update` now uses the maximum of `paid_up_until` and
  `fallback_release_date`, extended by `updates_window_days`, before
  comparing against the release date. Snapshots without any of the
  three fields still allow all releases.

[0.1.6]: https://github.com/akira-io/billing-sdk-rust/releases/tag/v0.1.6

## [0.1.5] — 2026-05-15

### Changed

- Version bump to align all three SDKs (JS, Rust, Go) on the same
  release number. No code or API changes.

[0.1.5]: https://github.com/akira-io/billing-sdk-rust/releases/tag/v0.1.5

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
