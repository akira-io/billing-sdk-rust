use std::collections::HashMap;
use std::sync::Arc;

use akira_billing::gate::{Gate, GateError, GateOptions};
use akira_billing::lifecycle::LicenseState;
use akira_billing::types::{LicenseSnapshotPayload, SignedLicense, UsageFeatureState, UsagePeriod};
use chrono::{DateTime, Duration, TimeZone, Utc};
use futures::future::BoxFuture;

fn payload_at(now: DateTime<Utc>, valid_until: DateTime<Utc>) -> LicenseSnapshotPayload {
    let _ = now;
    let mut features = HashMap::new();
    features.insert("mock_server".into(), true);
    features.insert("requests_per_day".into(), true);
    features.insert("locked_feature".into(), false);

    let mut usage = HashMap::new();
    usage.insert(
        "mock_server".into(),
        UsageFeatureState::Bool { enabled: true },
    );
    usage.insert(
        "locked_feature".into(),
        UsageFeatureState::Bool { enabled: false },
    );
    usage.insert(
        "requests_per_day".into(),
        UsageFeatureState::Counter {
            allowance: 200,
            period: UsagePeriod::Daily,
            period_start: String::new(),
            period_end: String::new(),
            consumed_at_issue: 50,
        },
    );

    LicenseSnapshotPayload {
        v: None,
        key_id: "k".into(),
        customer_id: "c".into(),
        product_key: "p".into(),
        plan_key: "pro_monthly".into(),
        licensing_mode: None,
        features,
        usage,
        fingerprint_hash: String::new(),
        serial: 0,
        issued_at: String::new(),
        valid_until: valid_until.to_rfc3339(),
        paid_up_until: None,
        fallback_release_date: None,
        updates_window_days: None,
        offline_grace_days: None,
    }
}

fn signed_dummy() -> SignedLicense {
    SignedLicense {
        key_id: String::new(),
        algorithm: String::new(),
        payload: String::new(),
        signature: String::new(),
        valid_until: String::new(),
    }
}

#[tokio::test]
async fn gate_checks() {
    let now = Utc.with_ymd_and_hms(2026, 1, 10, 12, 0, 0).unwrap();
    let payload = payload_at(now, now + Duration::hours(24));
    let payload_arc = Arc::new(payload);

    let p_clone = Arc::clone(&payload_arc);
    let loader = Arc::new(move || {
        let p = (*p_clone).clone();
        Box::pin(async move { Ok(Some((signed_dummy(), p))) }) as BoxFuture<'static, _>
    });

    let local_consumption = Arc::new(|feature: String| {
        Box::pin(async move {
            if feature == "requests_per_day" {
                Ok(25u64)
            } else {
                Ok(0u64)
            }
        }) as BoxFuture<'static, _>
    });

    let now_fn = Arc::new(move || now);

    let gate = Gate::new(GateOptions {
        loader: Some(loader),
        local_consumption: Some(local_consumption),
        grace_window: Duration::days(7),
        now: Some(now_fn),
    });

    let acc = gate.check("mock_server").await.unwrap();
    assert!(acc.allowed && acc.unlimited, "{:?}", acc);

    let acc = gate.check("requests_per_day").await.unwrap();
    assert!(acc.allowed && acc.remaining == 125, "{:?}", acc);

    let acc = gate.check("locked_feature").await.unwrap();
    assert!(
        !acc.allowed && acc.reason == "feature_disabled",
        "{:?}",
        acc
    );

    let err = gate.require("locked_feature").await.unwrap_err();
    match err {
        GateError::Denied(d) => assert_eq!(d.0.reason, "feature_disabled"),
        other => panic!("expected denied, got {other:?}"),
    }
}

#[tokio::test]
async fn gate_expired() {
    let now = Utc.with_ymd_and_hms(2026, 1, 10, 12, 0, 0).unwrap();
    let payload = payload_at(now, now - Duration::days(30));
    let payload_arc = Arc::new(payload);

    let p_clone = Arc::clone(&payload_arc);
    let loader = Arc::new(move || {
        let p = (*p_clone).clone();
        Box::pin(async move { Ok(Some((signed_dummy(), p))) }) as BoxFuture<'static, _>
    });
    let now_fn = Arc::new(move || now);

    let gate = Gate::new(GateOptions {
        loader: Some(loader),
        local_consumption: None,
        grace_window: Duration::days(7),
        now: Some(now_fn),
    });

    let acc = gate.check("mock_server").await.unwrap();
    assert!(!acc.allowed);
    assert_eq!(acc.state, LicenseState::Expired);
}

#[tokio::test]
async fn gate_no_license() {
    let loader = Arc::new(|| Box::pin(async { Ok(None) }) as BoxFuture<'static, _>);
    let gate = Gate::new(GateOptions {
        loader: Some(loader),
        ..Default::default()
    });

    let acc = gate.check("mock_server").await.unwrap();
    assert!(!acc.allowed);
    assert_eq!(acc.reason, "no_license");
}
