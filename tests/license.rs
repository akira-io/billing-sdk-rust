use std::collections::HashMap;

use akira_billing::license::{
    can_use_update, compute_remaining, decode_license, is_expired, is_in_grace, period_reset_at,
    verify_license, RemainingValue,
};
use akira_billing::types::{
    LicenseSnapshotPayload, LicensingMode, SignedLicense, UsageFeatureState, UsagePeriod,
};
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine as _;
use chrono::{DateTime, Utc};
use ed25519_dalek::{Signer, SigningKey};

fn base_payload() -> LicenseSnapshotPayload {
    let mut usage = HashMap::new();
    usage.insert(
        "agent_run".into(),
        UsageFeatureState::Counter {
            allowance: 5,
            period: UsagePeriod::Monthly,
            period_start: "2026-05-01T00:00:00+00:00".into(),
            period_end: "2026-05-31T00:00:00+00:00".into(),
            consumed_at_issue: 2,
        },
    );
    LicenseSnapshotPayload {
        v: Some(2),
        key_id: "k1".into(),
        customer_id: "cust-1".into(),
        product_key: "maintainer".into(),
        plan_key: "free".into(),
        licensing_mode: Some(LicensingMode::OfflineSnapshot),
        features: HashMap::from([("agent_run".to_string(), true)]),
        usage,
        fingerprint_hash: "fp".into(),
        serial: 1,
        issued_at: "2026-05-15T10:00:00+00:00".into(),
        valid_until: "2026-05-29T10:00:00+00:00".into(),
        paid_up_until: Some("2027-05-15T00:00:00+00:00".into()),
        fallback_release_date: Some("2027-05-15T00:00:00+00:00".into()),
        updates_window_days: None,
        offline_grace_days: None,
    }
}

fn make_signed(payload: &LicenseSnapshotPayload, signing: &SigningKey) -> SignedLicense {
    let json = serde_json::to_vec(payload).unwrap();
    let sig = signing.sign(&json);
    SignedLicense {
        key_id: payload.key_id.clone(),
        algorithm: "ed25519".into(),
        payload: B64.encode(&json),
        signature: B64.encode(sig.to_bytes()),
        valid_until: payload.valid_until.clone(),
    }
}

#[test]
fn decode_license_parses_payload() {
    let payload = base_payload();
    let signing = SigningKey::from_bytes(&[7u8; 32]);
    let signed = make_signed(&payload, &signing);
    let decoded = decode_license(&signed).unwrap();
    assert_eq!(decoded.payload.plan_key, "free");
    assert_eq!(decoded.payload.serial, 1);
}

#[test]
fn verify_license_ed25519_roundtrip() {
    let signing = SigningKey::from_bytes(&[42u8; 32]);
    let pubkey_b64 = B64.encode(signing.verifying_key().to_bytes());
    let signed = make_signed(&base_payload(), &signing);
    assert!(verify_license(&signed, &pubkey_b64).unwrap());
}

#[test]
fn verify_license_rejects_wrong_key() {
    let signing = SigningKey::from_bytes(&[42u8; 32]);
    let signed = make_signed(&base_payload(), &signing);
    let wrong = SigningKey::from_bytes(&[1u8; 32]);
    let wrong_b64 = B64.encode(wrong.verifying_key().to_bytes());
    assert!(!verify_license(&signed, &wrong_b64).unwrap());
}

#[test]
fn verify_license_rejects_non_ed25519() {
    let mut signed = make_signed(&base_payload(), &SigningKey::from_bytes(&[1u8; 32]));
    signed.algorithm = "rsa".into();
    assert!(!verify_license(&signed, "AA==").unwrap());
}

#[test]
fn compute_remaining_subtracts_local_consumed() {
    let p = base_payload();
    assert_eq!(
        compute_remaining(&p, "agent_run", 0),
        Some(RemainingValue::Finite(3))
    );
    assert_eq!(
        compute_remaining(&p, "agent_run", 2),
        Some(RemainingValue::Finite(1))
    );
    assert_eq!(
        compute_remaining(&p, "agent_run", 100),
        Some(RemainingValue::Finite(0))
    );
    assert_eq!(compute_remaining(&p, "ghost", 0), None);
}

#[test]
fn compute_remaining_bool_features() {
    let mut p = base_payload();
    p.usage.clear();
    p.usage.insert(
        "white_label".into(),
        UsageFeatureState::Bool { enabled: true },
    );
    assert_eq!(
        compute_remaining(&p, "white_label", 0),
        Some(RemainingValue::Unlimited)
    );

    p.usage.insert(
        "white_label".into(),
        UsageFeatureState::Bool { enabled: false },
    );
    assert_eq!(
        compute_remaining(&p, "white_label", 0),
        Some(RemainingValue::Finite(0))
    );
}

#[test]
fn expiry_and_grace() {
    let p = base_payload();
    let inside: DateTime<Utc> = "2026-05-20T00:00:00Z".parse().unwrap();
    let after: DateTime<Utc> = "2026-06-10T00:00:00Z".parse().unwrap();
    assert!(!is_expired(&p, Some(inside)));
    assert!(is_expired(&p, Some(after)));

    assert!(is_in_grace(
        &p,
        7 * 24 * 3600,
        Some("2026-06-02T00:00:00Z".parse().unwrap())
    ));
    assert!(!is_in_grace(
        &p,
        7 * 24 * 3600,
        Some("2026-06-08T00:00:00Z".parse().unwrap())
    ));
}

#[test]
fn can_use_update_respects_paid_up() {
    let p = base_payload();
    assert!(can_use_update(&p, "2027-01-01T00:00:00Z".parse().unwrap()));
    assert!(!can_use_update(&p, "2028-01-01T00:00:00Z".parse().unwrap()));
}

#[test]
fn can_use_update_extends_with_window() {
    let mut p = base_payload();
    p.updates_window_days = Some(365);
    assert!(can_use_update(&p, "2027-06-01T00:00:00Z".parse().unwrap()));
    assert!(can_use_update(&p, "2028-04-01T00:00:00Z".parse().unwrap()));
    assert!(!can_use_update(&p, "2028-12-01T00:00:00Z".parse().unwrap()));
}

#[test]
fn can_use_update_uses_max_of_paid_up_and_fallback() {
    let mut p = base_payload();
    p.paid_up_until = Some("2026-01-01T00:00:00+00:00".into());
    p.fallback_release_date = Some("2027-12-31T00:00:00+00:00".into());
    assert!(can_use_update(&p, "2027-06-01T00:00:00Z".parse().unwrap()));
}

#[test]
fn period_reset_at_returns_counter_end() {
    let p = base_payload();
    let reset = period_reset_at(&p, "agent_run").unwrap();
    assert_eq!(reset.to_rfc3339(), "2026-05-31T00:00:00+00:00");
}
