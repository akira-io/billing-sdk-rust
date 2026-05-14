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
    pub stripe_price_id: Option<String>,
    pub features: Vec<ApiPlanFeature>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiPlanFeature {
    pub key: String,
    pub name: String,
    pub description: Option<String>,
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
