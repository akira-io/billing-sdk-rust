use serde::Serialize;

use crate::client::Client;
use crate::types::{Customer, LicenseCheckPayload};

#[derive(Debug, Clone, Serialize)]
pub struct AuthSnapshot {
    pub authenticated: bool,
    pub licensed: bool,
    pub customer: Option<Customer>,
    pub features: Vec<String>,
}

impl AuthSnapshot {
    pub fn guest() -> Self {
        Self { authenticated: false, licensed: false, customer: None, features: Vec::new() }
    }

    pub fn has_feature(&self, key: &str) -> bool {
        self.features.iter().any(|f| f == key)
    }
}

pub struct RefreshOptions<'a> {
    pub product: &'a str,
    pub fallback_feature: &'a str,
}

pub async fn refresh_auth(
    client: &Client,
    opts: RefreshOptions<'_>,
) -> Result<AuthSnapshot, crate::Error> {
    if client.customer_token().is_none() {
        return Ok(AuthSnapshot::guest());
    }

    let customer = match client.customer_me().await {
        Ok(c) => c,
        Err(crate::Error::Api { status, .. }) if status == 401 => {
            return Ok(AuthSnapshot::guest());
        }
        Err(e) => return Err(e),
    };

    let features = client
        .customer_features(opts.product)
        .await
        .map(|r| r.features)
        .unwrap_or_default();

    let licensed = if !features.is_empty() {
        true
    } else {
        client
            .license_check(LicenseCheckPayload { product: opts.product, feature: opts.fallback_feature })
            .await
            .map(|r| r.allowed)
            .unwrap_or(false)
    };

    Ok(AuthSnapshot {
        authenticated: true,
        licensed,
        customer: Some(customer),
        features,
    })
}
