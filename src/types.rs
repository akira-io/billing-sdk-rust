use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct PlansResponse {
    pub product: String,
    pub name: String,
    pub description: Option<String>,
    pub landing_url: Option<String>,
    pub beta_ends_at: Option<String>,
    pub beta_active: bool,
    pub plans: Vec<ApiPlan>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiPlan {
    pub id: String,
    pub key: String,
    pub name: String,
    pub description: Option<String>,
    pub amount: Option<i64>,
    pub currency: Option<String>,
    pub billing_interval: Option<String>,
    pub trial_period_days: u32,
    #[serde(default)]
    pub is_coming_soon: bool,
    pub features: Vec<ApiPlanFeature>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiPlanFeature {
    pub key: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReleaseAsset {
    pub os: String,
    pub arch: String,
    pub format: String,
    pub object_key: String,
    pub size_bytes: i64,
    pub sha256: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReleaseManifest {
    pub version: String,
    pub channel: String,
    pub released_at: String,
    pub notes_url: Option<String>,
    pub assets: Vec<ReleaseAsset>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssuedDownload {
    pub event_id: String,
    pub product: String,
    pub version: String,
    pub channel: String,
    pub os: String,
    pub arch: String,
    pub format: String,
    pub size_bytes: i64,
    pub sha256: String,
    pub signed_url: String,
    pub expires_at: String,
    pub beacon_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IssuedTrial {
    pub product: String,
    pub plan: Option<String>,
    pub source: String,
    pub starts_at: String,
    pub ends_at: String,
    pub trial_period_days: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OtpRequestPayload<'a> {
    pub email: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_fp: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_version: Option<&'a str>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OtpVerifyPayload<'a> {
    pub email: &'a str,
    pub code: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_fp: Option<&'a str>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OtpVerifyResponse {
    pub access_token: String,
    pub customer: OtpCustomer,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OtpCustomer {
    pub id: String,
    pub email: String,
}
