use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use futures::future::BoxFuture;
use tokio::sync::Mutex;

use crate::error::Error;
use crate::license::{compute_remaining, RemainingValue};
use crate::lifecycle::{compute_state, LicenseState};
use crate::types::{LicenseSnapshotPayload, SignedLicense};

#[derive(Debug, Clone)]
pub struct FeatureAccess {
    pub feature: String,
    pub allowed: bool,
    pub has_feature: bool,
    pub unlimited: bool,
    pub remaining: u64,
    pub reason: String,
    pub plan: String,
    pub state: LicenseState,
}

impl FeatureAccess {
    fn new(feature: &str) -> Self {
        Self {
            feature: feature.to_string(),
            allowed: false,
            has_feature: false,
            unlimited: false,
            remaining: 0,
            reason: String::new(),
            plan: String::new(),
            state: LicenseState::None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GateDenied(pub FeatureAccess);

impl std::fmt::Display for GateDenied {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "billing: feature {:?} denied ({})",
            self.0.feature, self.0.reason
        )
    }
}

impl std::error::Error for GateDenied {}

#[derive(Debug)]
pub enum GateError {
    Denied(GateDenied),
    Other(Error),
}

impl std::fmt::Display for GateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GateError::Denied(d) => d.fmt(f),
            GateError::Other(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for GateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            GateError::Denied(d) => Some(d),
            GateError::Other(e) => Some(e),
        }
    }
}

impl From<Error> for GateError {
    fn from(value: Error) -> Self {
        GateError::Other(value)
    }
}

pub type LicenseLoader = Arc<
    dyn Fn() -> BoxFuture<'static, Result<Option<(SignedLicense, LicenseSnapshotPayload)>, Error>>
        + Send
        + Sync,
>;

pub type LocalConsumption =
    Arc<dyn Fn(String) -> BoxFuture<'static, Result<u64, Error>> + Send + Sync>;

pub type NowFn = Arc<dyn Fn() -> DateTime<Utc> + Send + Sync>;

pub struct GateOptions {
    pub loader: Option<LicenseLoader>,
    pub local_consumption: Option<LocalConsumption>,
    pub grace_window: Duration,
    pub now: Option<NowFn>,
}

impl Default for GateOptions {
    fn default() -> Self {
        Self {
            loader: None,
            local_consumption: None,
            grace_window: Duration::zero(),
            now: None,
        }
    }
}

pub struct Gate {
    loader: Option<LicenseLoader>,
    local_consumption: LocalConsumption,
    grace_window: Duration,
    now: NowFn,
    mu: Mutex<()>,
}

impl Gate {
    pub fn new(opts: GateOptions) -> Self {
        let local_consumption: LocalConsumption = opts.local_consumption.unwrap_or_else(|| {
            Arc::new(|_feature: String| Box::pin(async { Ok(0u64) }) as BoxFuture<'static, _>)
        });
        let now: NowFn = opts.now.unwrap_or_else(|| Arc::new(Utc::now));
        Self {
            loader: opts.loader,
            local_consumption,
            grace_window: opts.grace_window,
            now,
            mu: Mutex::new(()),
        }
    }

    pub async fn check(&self, feature: &str) -> Result<FeatureAccess, Error> {
        let mut access = FeatureAccess::new(feature);

        let Some(loader) = self.loader.clone() else {
            access.reason = "no_loader".into();
            return Ok(access);
        };

        let _guard = self.mu.lock().await;

        let loaded = match loader().await {
            Ok(v) => v,
            Err(err) => {
                access.reason = "verify_failed".into();
                return Err(err);
            }
        };

        let Some((_signed, payload)) = loaded else {
            access.reason = "no_license".into();
            return Ok(access);
        };

        access.plan = payload.plan_key.clone();
        let now = (self.now)();
        access.state = compute_state(Some(&payload), self.grace_window, now);

        match access.state {
            LicenseState::Expired | LicenseState::Invalid => {
                access.reason = format!("license_{}", access.state);
                return Ok(access);
            }
            _ => {}
        }

        if let Some(&enabled) = payload.features.get(feature) {
            access.has_feature = enabled;
            if !enabled {
                access.reason = "feature_disabled".into();
                return Ok(access);
            }
        }

        let consumed = match (self.local_consumption)(feature.to_string()).await {
            Ok(v) => v,
            Err(err) => {
                access.reason = "local_consumption_failed".into();
                return Err(err);
            }
        };

        match compute_remaining(&payload, feature, consumed) {
            None => {
                if access.has_feature {
                    access.allowed = true;
                    access.unlimited = true;
                    return Ok(access);
                }
                access.reason = "feature_missing".into();
                Ok(access)
            }
            Some(RemainingValue::Unlimited) => {
                access.unlimited = true;
                access.allowed = true;
                access.has_feature = true;
                Ok(access)
            }
            Some(RemainingValue::Finite(remaining)) => {
                access.remaining = remaining;
                if remaining > 0 {
                    access.allowed = true;
                    access.has_feature = true;
                } else {
                    access.reason = "limit_reached".into();
                }
                Ok(access)
            }
        }
    }

    pub async fn require(&self, feature: &str) -> Result<FeatureAccess, GateError> {
        let access = self.check(feature).await?;
        if !access.allowed {
            return Err(GateError::Denied(GateDenied(access)));
        }
        Ok(access)
    }
}
