# akira-billing

Rust client for the [Akira Billing API](https://github.com/akira-foundation/billing).
Sister crate of [`billing-sdk-go`](https://github.com/akira-io/billing-sdk-go);
both consume the same wire protocol and pass the same fixture vectors.

Handles request signing, OTP login, full license lifecycle (check / activate
/ refresh), entitlements, customer profile, billing portal, per-day usage
tracking, downloads, trial start, and plans listing. Async via `reqwest` +
`tokio`.

## Install

```toml
[dependencies]
akira-billing = { git = "https://github.com/akira-io/billing-sdk-rust", tag = "v0.1.0" }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

## Quick start

```rust
use akira_billing::{
    Client,
    types::{OtpRequestPayload, OtpVerifyPayload},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = Client::new(
        "https://billing.akira.foundation",
        "spectra",
        // Injected at build time. See "Build-time secret injection" below.
        env!("AKIRA_BILLING_SECRET"),
    );

    // 1. Public plans
    let plans = client.plans().await?;
    println!("Beta active: {} · {} plans", plans.beta_active, plans.plans.len());

    // 2. OTP login
    client
        .request_otp(OtpRequestPayload {
            email: "kid@example.com",
            device_fp: Some("deadbeef"),
            platform: Some("macos"),
            app_version: Some("0.1.0"),
        })
        .await?;

    let session = client
        .verify_otp(OtpVerifyPayload {
            email: "kid@example.com",
            code: "123456",
            device_fp: Some("deadbeef"),
        })
        .await?;
    println!("Signed in as {}", session.customer.email);
    // session.access_token is now stored on the client; subsequent calls auto-sign + auth.

    // 3. Start trial (None = first eligible plan)
    let trial = client.start_trial(None).await?;
    println!("Trial ends {}", trial.ends_at);

    Ok(())
}
```

## Configuration

```rust
let mut client = Client::new(base_url, product_slug, product_secret);
client.set_customer_token("existing-bearer");  // restore from keychain
```

| Field            | Type     | Notes                                                 |
| ---------------- | -------- | ----------------------------------------------------- |
| `base_url`       | `String` | Billing endpoint root, no trailing slash              |
| `product_slug`   | `String` | Matches `products.key` on the backend                 |
| `product_secret` | `String` | Per-product HMAC secret, set at build time            |
| `customer_token` | `Option<String>` | Sanctum bearer, populated after `verify_otp`  |

The crate is `Clone`, so you can hand it across tasks; the HTTP client
underneath pools connections via `reqwest`.

## Endpoints

| Method                              | Backend route                                  | Auth          |
| ----------------------------------- | ---------------------------------------------- | ------------- |
| `plans()`                           | `GET  /api/v1/products/{key}/plans`            | HMAC only     |
| `request_otp(payload)`              | `POST /api/auth/customer/otp/request`          | HMAC only     |
| `verify_otp(payload)`               | `POST /api/auth/customer/otp/verify`           | HMAC only     |
| `start_trial(plan_key)`             | `POST /api/v1/me/products/{key}/trial`         | HMAC + bearer |
| `customer_me()`                     | `GET  /api/me`                                 | HMAC + bearer |
| `license_check(payload)`            | `POST /api/licenses/check`                     | HMAC + bearer |
| `license_activate(payload)`         | `POST /api/licenses/activate`                  | HMAC + bearer |
| `license_refresh(payload)`          | `POST /api/licenses/refresh`                   | HMAC + bearer |
| `license_sync_usage(payload)`       | `POST /api/licenses/sync-usage` (offline mode) | HMAC + bearer |
| `entitlements()`                    | `GET  /api/me/entitlements`                    | HMAC + bearer |
| `billing_portal(return_url)`        | `GET  /api/billing/portal`                     | HMAC + bearer |
| `track_usage(payload)`              | `POST /api/me/usage` (variable `count`)        | HMAC + bearer |
| `latest_release(channel)`           | `GET  /api/v1/downloads/{product}/releases/{channel}/latest` | HMAC only |
| `issue_download(channel, platform)` | `GET  /api/v1/downloads/{product}/{channel}/{platform}`      | HMAC only |
| `complete_download(beacon_url)`     | `POST` to the absolute beacon URL              | unsigned beacon |
| `public_license_keys()`             | `GET  /api/v1/license-keys/public`             | unsigned      |

For routes the SDK has not yet typed drop down to
`signature::{canonical, sign, new_nonce}` and build the request manually; the
helpers are public.

## Error handling

```rust
use akira_billing::Error;

match client.plans().await {
    Ok(plans) => { /* ... */ }
    Err(Error::Api { status: 404, code }) if code == "unknown_product" => { /* slug typo */ }
    Err(Error::Api { code, .. }) if code == "trial_already_used"
        || code == "already_has_entitlement" => { /* business rule */ }
    Err(Error::Api { code, .. }) if code == "bad_signature"
        || code == "missing_signature_headers"
        || code == "timestamp_skew" => { /* wire-level: rotate or sync clock */ }
    Err(Error::Transport(e)) => eprintln!("network error: {e}"),
    Err(e) => return Err(e.into()),
}
```

## Licensing modes

Products are tagged server-side with a `licensing_mode`:

| Mode | When to use | Client flow |
|---|---|---|
| `offline_snapshot` | Desktop apps. Long-lived entitlement, infrequent sync. | Refresh signed snapshot, decrement local counter, sync deltas periodically. |
| `online_realtime` | Pay-per-unit (AI tokens, API calls). | Pre-check budget + post-commit actual `count`. |

### Offline snapshot helpers

`license` module exports `decode_license`, `verify_license` (Ed25519 via
`ed25519-dalek`), `compute_remaining`, `is_expired`, `is_in_grace`,
`can_use_update`, `period_reset_at`.

```rust
use akira_billing::{Client, license, RemainingValue};
use akira_billing::types::{LicenseRefreshPayload, LicenseSyncUsagePayload};
use std::collections::HashMap;

let resp = client.license_refresh(LicenseRefreshPayload {
    product: "maintainer",
    fingerprint: &fp,
}).await?;

let decoded = license::decode_license(&resp.license)?;

let keys = client.public_license_keys().await?;
let ok = license::verify_license(&resp.license, &keys.keys[0].public_key_base64)?;
if !ok { panic!("forged license") }

match license::compute_remaining(&decoded.payload, "agent_run", local_consumed) {
    Some(RemainingValue::Finite(0)) => return Err("limit reached".into()),
    Some(RemainingValue::Finite(n)) => println!("{n} remaining"),
    Some(RemainingValue::Unlimited)  => {},
    None => return Err("unknown feature".into()),
}

let mut deltas = HashMap::new();
deltas.insert("agent_run".to_string(), 3u64);
let next = client.license_sync_usage(LicenseSyncUsagePayload {
    product: "maintainer",
    fingerprint: &fp,
    serial: decoded.payload.serial,
    deltas,
}).await?;
```

### Online realtime (variable `count`)

```rust
use akira_billing::types::UsagePayload;

let pre = client.track_usage(UsagePayload {
    product: "aisite",
    feature: "llm_tokens",
    date: "2026-05-15",
    device_fp: &fp,
    action: "check",
    count: Some(4000),
    platform: None,
    device_type: None,
    app_version: None,
}).await?;
if !pre.allowed { return Err("budget exhausted".into()); }

// run LLM, get actual tokens
let actual = 1247u32;
client.track_usage(UsagePayload {
    action: "increment",
    count: Some(actual),
    // ... same fields
    product: "aisite", feature: "llm_tokens", date: "2026-05-15", device_fp: &fp,
    platform: None, device_type: None, app_version: None,
}).await?;
```

## Build-time secret injection

```bash
AKIRA_BILLING_SECRET=$SPECTRA_BILLING_SECRET cargo build --release
```

`env!("AKIRA_BILLING_SECRET")` is a compile-time macro: a release build
without the secret fails to compile, so production binaries never ship
without one.

For local development, use `option_env!` and a fallback:

```rust
const PRODUCT_SECRET: &str = match option_env!("AKIRA_BILLING_SECRET") {
    Some(s) => s,
    None => "dev-only-secret",
};
```

## Cargo features

| Feature  | Default | Effect                                        |
| -------- | ------- | --------------------------------------------- |
| `tokio`  | on      | Brings `tokio` for the async runtime          |

`reqwest` is wired with `rustls-tls` by default; no system OpenSSL needed.

## Wire protocol

Signing scheme: HMAC-SHA256 over a canonical string that includes product
slug, unix timestamp, nonce, HTTP method, request path, and a hash of the
body.

Full spec: [docs/protocol.md](docs/protocol.md).

The fixture vectors in `tests/fixtures/signature-vectors.json` are shared
with the backend and the Go SDK. Run the test suite to confirm parity:

```bash
cargo test
```

## License

MIT OR Apache-2.0.
