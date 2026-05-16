use std::collections::HashMap;

use akira_billing::lifecycle::{compute_state, trial_days_left, LicenseState};
use akira_billing::types::LicenseSnapshotPayload;
use chrono::{Duration, TimeZone, Utc};

fn base_payload() -> LicenseSnapshotPayload {
    LicenseSnapshotPayload {
        v: None,
        key_id: "k".into(),
        customer_id: "c".into(),
        product_key: "p".into(),
        plan_key: "pro_monthly".into(),
        licensing_mode: None,
        features: HashMap::new(),
        usage: HashMap::new(),
        fingerprint_hash: String::new(),
        serial: 0,
        issued_at: String::new(),
        valid_until: String::new(),
        paid_up_until: None,
        fallback_release_date: None,
        updates_window_days: None,
        offline_grace_days: None,
    }
}

#[test]
fn compute_state_cases() {
    let now = Utc.with_ymd_and_hms(2026, 1, 10, 12, 0, 0).unwrap();
    let grace = Duration::days(7);

    assert_eq!(compute_state(None, grace, now), LicenseState::None);

    let empty = base_payload();
    assert_eq!(
        compute_state(Some(&empty), grace, now),
        LicenseState::Invalid
    );

    let mut active = base_payload();
    active.valid_until = (now + Duration::hours(48)).to_rfc3339();
    assert_eq!(
        compute_state(Some(&active), grace, now),
        LicenseState::Active
    );

    let mut trial_plan = base_payload();
    trial_plan.valid_until = (now + Duration::hours(48)).to_rfc3339();
    trial_plan.plan_key = "pro:trial".into();
    assert_eq!(
        compute_state(Some(&trial_plan), grace, now),
        LicenseState::Trialing
    );

    let mut trial_feature = base_payload();
    trial_feature.valid_until = (now + Duration::hours(48)).to_rfc3339();
    trial_feature.features.insert("__trial".into(), true);
    assert_eq!(
        compute_state(Some(&trial_feature), grace, now),
        LicenseState::Trialing
    );

    let mut grace_payload = base_payload();
    grace_payload.valid_until = (now - Duration::hours(24)).to_rfc3339();
    grace_payload.plan_key = "pro".into();
    assert_eq!(
        compute_state(Some(&grace_payload), grace, now),
        LicenseState::Grace
    );

    let mut expired = base_payload();
    expired.valid_until = (now - Duration::days(30)).to_rfc3339();
    expired.plan_key = "pro".into();
    assert_eq!(
        compute_state(Some(&expired), grace, now),
        LicenseState::Expired
    );
}

#[test]
fn trial_days_left_works() {
    let now = Utc.with_ymd_and_hms(2026, 1, 10, 12, 0, 0).unwrap();
    let mut p = base_payload();
    p.valid_until = (now + Duration::hours(72)).to_rfc3339();
    p.plan_key = "pro:trial".into();
    assert_eq!(trial_days_left(Some(&p), now), 3);
    assert_eq!(trial_days_left(None, now), 0);
}

#[test]
fn display_lowercase() {
    assert_eq!(LicenseState::None.to_string(), "none");
    assert_eq!(LicenseState::Trialing.to_string(), "trialing");
    assert_eq!(LicenseState::Expired.to_string(), "expired");
}
