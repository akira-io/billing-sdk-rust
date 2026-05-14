# akira-billing

Rust client for the Akira Billing API. Sister crate of
[`billing-sdk-go`](https://github.com/akira-foundation/billing-sdk-go);
both consume the same wire protocol and pass the same fixture vectors.

## Usage

```rust
use akira_billing::{Client, types::{OtpRequestPayload, OtpVerifyPayload}};

let mut client = Client::new(
    "https://billing.akira.foundation",
    "spectra",
    env!("AKIRA_BILLING_SECRET"),
);

let plans = client.plans().await?;

client.request_otp(OtpRequestPayload {
    email: "kid@example.com",
    device_fp: None,
    platform: None,
    app_version: None,
}).await?;

let session = client.verify_otp(OtpVerifyPayload {
    email: "kid@example.com",
    code: "123456",
    device_fp: None,
}).await?;
// session.access_token is now stored on the client.

let trial = client.start_trial(None).await?;
```

## Build-time secret injection

```bash
AKIRA_BILLING_SECRET=$SPECTRA_BILLING_SECRET cargo build --release
```

`env!("AKIRA_BILLING_SECRET")` panics at compile time when missing, so
release builds never ship without a real secret.

## Wire protocol

Signature scheme documented in
[akira-foundation/billing](https://github.com/akira-foundation/billing/blob/main/docs/billing-sdk/protocol.md).
