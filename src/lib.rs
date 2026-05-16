//! # akira-billing
//!
//! Rust client for the Akira Billing API. Mirrors the Go SDK; both crates
//! pass the same shared test vectors at `tests/fixtures/signature-vectors.json`.

pub mod client;
pub mod error;
pub mod gate;
pub mod license;
pub mod lifecycle;
pub mod oauth;
pub mod signature;
pub mod types;
pub mod usage;

#[cfg(feature = "loopback")]
pub mod loopback;

#[cfg(feature = "desktop")]
pub mod desktop;

pub use client::Client;
pub use error::Error;
pub use gate::{FeatureAccess, Gate, GateDenied, GateError, GateOptions};
pub use license::{
    can_use_update, compute_remaining, decode_license, is_expired, is_in_grace, period_reset_at,
    verify_license, DecodedLicense, RemainingValue,
};
pub use lifecycle::{compute_state, trial_days_left, LicenseState};
pub use oauth::{
    build_oauth_init_url, generate_oauth_state, generate_pkce_challenge, BuildOauthInitUrl,
    PkceChallenge,
};
pub use signature::{
    canonical, new_nonce, sign, HEADER_NONCE, HEADER_PRODUCT, HEADER_SIGNATURE, HEADER_TIMESTAMP,
};
pub use usage::{MemoryBuffer, TrackerOptions, UsageBuffer, UsageTracker};
