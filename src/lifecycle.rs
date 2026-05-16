use chrono::{DateTime, Duration, Utc};

use crate::types::LicenseSnapshotPayload;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LicenseState {
    None,
    Invalid,
    Active,
    Trialing,
    Grace,
    Expired,
}

impl LicenseState {
    pub fn as_str(&self) -> &'static str {
        match self {
            LicenseState::None => "none",
            LicenseState::Invalid => "invalid",
            LicenseState::Active => "active",
            LicenseState::Trialing => "trialing",
            LicenseState::Grace => "grace",
            LicenseState::Expired => "expired",
        }
    }
}

impl std::fmt::Display for LicenseState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

pub fn compute_state(
    payload: Option<&LicenseSnapshotPayload>,
    grace: Duration,
    now: DateTime<Utc>,
) -> LicenseState {
    let Some(payload) = payload else {
        return LicenseState::None;
    };
    if payload.valid_until.is_empty() {
        return LicenseState::Invalid;
    }
    let Some(expiry) = parse_iso(&payload.valid_until) else {
        return LicenseState::Invalid;
    };

    if now <= expiry {
        if is_trial(payload) {
            return LicenseState::Trialing;
        }
        return LicenseState::Active;
    }

    let cutoff = expiry + grace;
    if now <= cutoff {
        LicenseState::Grace
    } else {
        LicenseState::Expired
    }
}

pub fn trial_days_left(payload: Option<&LicenseSnapshotPayload>, now: DateTime<Utc>) -> i64 {
    let Some(payload) = payload else {
        return 0;
    };
    if !is_trial(payload) {
        return 0;
    }
    let Some(expiry) = parse_iso(&payload.valid_until) else {
        return 0;
    };
    if now >= expiry {
        return 0;
    }
    let delta = expiry - now;
    let secs = delta.num_seconds();
    let day = 86_400i64;
    let mut days = secs / day;
    if secs % day > 0 {
        days += 1;
    }
    days
}

fn is_trial(payload: &LicenseSnapshotPayload) -> bool {
    if let Some(true) = payload.features.get("__trial").copied() {
        return true;
    }
    payload.plan_key.ends_with(":trial")
}

fn parse_iso(s: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|d| d.with_timezone(&Utc))
}
