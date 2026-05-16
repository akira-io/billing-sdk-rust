//! Helpers for `offline_snapshot` licensing mode: decode + verify Ed25519
//! signed snapshots and reason about usage state, expiry, and update windows.
//!
//! Ed25519 verification uses [`ed25519-dalek`]. Server signs the canonical
//! JSON payload of [`LicenseSnapshotPayload`] with the configured private
//! key; clients verify with the public key fetched from
//! `/api/v1/license-keys/public` (or embedded at build time).

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine as _;
use chrono::{DateTime, Utc};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};

use crate::error::Error;
use crate::types::{LicenseSnapshotPayload, SignedLicense, UsageFeatureState};

#[derive(Debug, Clone)]
pub struct DecodedLicense {
    pub raw: SignedLicense,
    pub payload: LicenseSnapshotPayload,
}

pub fn decode_license(signed: &SignedLicense) -> Result<DecodedLicense, Error> {
    let payload_bytes = B64
        .decode(signed.payload.as_bytes())
        .map_err(|e| Error::Api {
            status: 0,
            code: format!("decode payload b64: {e}"),
        })?;
    let payload: LicenseSnapshotPayload =
        serde_json::from_slice(&payload_bytes).map_err(|e| Error::Api {
            status: 0,
            code: format!("parse payload: {e}"),
        })?;
    Ok(DecodedLicense {
        raw: signed.clone(),
        payload,
    })
}

pub fn verify_license(signed: &SignedLicense, public_key_b64: &str) -> Result<bool, Error> {
    if signed.algorithm != "ed25519" {
        return Ok(false);
    }

    let payload_bytes = B64
        .decode(signed.payload.as_bytes())
        .map_err(|e| Error::Api {
            status: 0,
            code: format!("decode payload b64: {e}"),
        })?;
    let sig_bytes = B64
        .decode(signed.signature.as_bytes())
        .map_err(|e| Error::Api {
            status: 0,
            code: format!("decode signature b64: {e}"),
        })?;
    let pk_bytes = B64
        .decode(public_key_b64.as_bytes())
        .map_err(|e| Error::Api {
            status: 0,
            code: format!("decode public key b64: {e}"),
        })?;

    let pk_arr: [u8; 32] = pk_bytes.as_slice().try_into().map_err(|_| Error::Api {
        status: 0,
        code: "public key must be 32 bytes".into(),
    })?;
    let sig_arr: [u8; 64] = sig_bytes.as_slice().try_into().map_err(|_| Error::Api {
        status: 0,
        code: "signature must be 64 bytes".into(),
    })?;

    let key = VerifyingKey::from_bytes(&pk_arr).map_err(|e| Error::Api {
        status: 0,
        code: format!("public key: {e}"),
    })?;
    let signature = Signature::from_bytes(&sig_arr);

    Ok(key.verify(&payload_bytes, &signature).is_ok())
}

pub fn compute_remaining(
    payload: &LicenseSnapshotPayload,
    feature: &str,
    consumed_local: u64,
) -> Option<RemainingValue> {
    let state = payload.usage.get(feature)?;
    Some(match state {
        UsageFeatureState::Bool { enabled } => {
            if *enabled {
                RemainingValue::Unlimited
            } else {
                RemainingValue::Finite(0)
            }
        }
        UsageFeatureState::Counter {
            allowance,
            consumed_at_issue,
            ..
        } => {
            let total_consumed = consumed_at_issue.saturating_add(consumed_local);
            RemainingValue::Finite(allowance.saturating_sub(total_consumed))
        }
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemainingValue {
    Finite(u64),
    Unlimited,
}

impl RemainingValue {
    pub fn is_zero(&self) -> bool {
        matches!(self, RemainingValue::Finite(0))
    }
}

pub fn is_expired(payload: &LicenseSnapshotPayload, now: Option<DateTime<Utc>>) -> bool {
    let now = now.unwrap_or_else(Utc::now);
    parse_iso(&payload.valid_until)
        .map(|expiry| expiry < now)
        .unwrap_or(true)
}

pub fn is_in_grace(
    payload: &LicenseSnapshotPayload,
    grace_seconds: u64,
    now: Option<DateTime<Utc>>,
) -> bool {
    let now = now.unwrap_or_else(Utc::now);
    parse_iso(&payload.valid_until)
        .map(|expiry| {
            let cutoff = expiry + chrono::Duration::seconds(grace_seconds as i64);
            now <= cutoff
        })
        .unwrap_or(false)
}

pub fn can_use_update(payload: &LicenseSnapshotPayload, release_date: DateTime<Utc>) -> bool {
    let paid_up = payload.paid_up_until.as_deref().and_then(parse_iso);
    let fallback = payload.fallback_release_date.as_deref().and_then(parse_iso);

    let effective = match (paid_up, fallback) {
        (None, None) => return true,
        (Some(a), None) => a,
        (None, Some(b)) => b,
        (Some(a), Some(b)) => a.max(b),
    };

    let window_days = payload.updates_window_days.unwrap_or(0) as i64;
    let cutoff = effective + chrono::Duration::days(window_days);
    release_date <= cutoff
}

pub fn period_reset_at(payload: &LicenseSnapshotPayload, feature: &str) -> Option<DateTime<Utc>> {
    match payload.usage.get(feature)? {
        UsageFeatureState::Counter { period_end, .. } => parse_iso(period_end),
        _ => None,
    }
}

pub fn deltas() -> HashMap<String, u64> {
    HashMap::new()
}

fn parse_iso(s: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|d| d.with_timezone(&Utc))
}

#[allow(dead_code)]
fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
