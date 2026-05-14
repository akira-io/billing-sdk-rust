//! # akira-billing
//!
//! Rust client for the Akira Billing API. Mirrors the Go SDK; both crates
//! pass the same shared test vectors at `tests/fixtures/signature-vectors.json`.

pub mod client;
pub mod error;
pub mod signature;
pub mod types;

pub use client::Client;
pub use error::Error;
pub use signature::{canonical, new_nonce, sign, HEADER_NONCE, HEADER_PRODUCT, HEADER_SIGNATURE, HEADER_TIMESTAMP};
