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

#[derive(Debug, Clone, Deserialize)]
pub struct Customer {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub trial_ends_at: Option<String>,
    pub plan: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LicenseCheckPayload<'a> {
    pub product: &'a str,
    pub feature: &'a str,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LicenseCheckResponse {
    pub allowed: bool,
    pub product: String,
    pub plan: Option<String>,
    pub feature: String,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LicenseActivatePayload<'a> {
    pub product: &'a str,
    pub device_type: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_name: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_version: Option<&'a str>,
    pub fingerprint: &'a str,
}

#[derive(Debug, Clone, Serialize)]
pub struct LicenseRefreshPayload<'a> {
    pub product: &'a str,
    pub fingerprint: &'a str,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SignedLicense {
    pub key_id: String,
    pub algorithm: String,
    pub payload: String,
    pub signature: String,
    pub valid_until: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ActivatedDevice {
    pub id: String,
    #[serde(rename = "type")]
    pub device_type: String,
    pub slots_used: u32,
    pub slots_limit: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LicenseActivateResponse {
    pub allowed: bool,
    pub product: String,
    pub plan: String,
    #[serde(default)]
    pub features: std::collections::HashMap<String, bool>,
    pub device: ActivatedDevice,
    pub license: SignedLicense,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EntitlementCustomer {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EntitlementsResponse {
    pub customer: EntitlementCustomer,
    pub entitlements: serde_json::Value,
    pub devices: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PortalLink {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LicensePublicKey {
    pub key_id: String,
    pub algorithm: String,
    pub public_key_base64: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LicensePublicKeysResponse {
    pub keys: Vec<LicensePublicKey>,
    pub active_key_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UsagePayload<'a> {
    pub product: &'a str,
    pub feature: &'a str,
    pub date: &'a str,
    pub device_fp: &'a str,
    pub action: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_type: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_version: Option<&'a str>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UsageResponse {
    pub count: u32,
    pub limit: Option<u32>,
    #[serde(default)]
    pub period: Option<String>,
    pub allowed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LicensingMode {
    OfflineSnapshot,
    OnlineRealtime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UsagePeriod {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UsageFeatureState {
    Bool {
        enabled: bool,
    },
    Counter {
        allowance: u64,
        period: UsagePeriod,
        period_start: String,
        period_end: String,
        consumed_at_issue: u64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseSnapshotPayload {
    #[serde(default)]
    pub v: Option<u32>,
    pub key_id: String,
    pub customer_id: String,
    pub product_key: String,
    pub plan_key: String,
    #[serde(default)]
    pub licensing_mode: Option<LicensingMode>,
    #[serde(default)]
    pub features: std::collections::HashMap<String, bool>,
    #[serde(default)]
    pub usage: std::collections::HashMap<String, UsageFeatureState>,
    pub fingerprint_hash: String,
    #[serde(default)]
    pub serial: u64,
    pub issued_at: String,
    pub valid_until: String,
    #[serde(default)]
    pub paid_up_until: Option<String>,
    #[serde(default)]
    pub fallback_release_date: Option<String>,
    #[serde(default)]
    pub updates_window_days: Option<u32>,
    #[serde(default)]
    pub offline_grace_days: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LicenseSyncUsagePayload<'a> {
    pub product: &'a str,
    pub fingerprint: &'a str,
    pub serial: u64,
    pub deltas: std::collections::HashMap<String, u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LicenseSyncUsageResponse {
    pub license: SignedLicense,
    pub applied: std::collections::HashMap<String, u64>,
    pub serial: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OauthProvider {
    Google,
    GitHub,
    Apple,
    Microsoft,
    GitLab,
    Bitbucket,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OauthProviderInfo {
    pub provider: OauthProvider,
    pub label: String,
    pub scopes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OauthProvidersResponse {
    pub product: String,
    pub providers: Vec<OauthProviderInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OauthExchangePayload<'a> {
    pub code: &'a str,
    pub code_verifier: &'a str,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OauthExchangeCustomer {
    pub id: String,
    pub email: String,
    #[serde(default)]
    pub name: Option<String>,
    pub product_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OauthExchangeEntitlement {
    #[serde(default)]
    pub plan_key: Option<String>,
    pub source: String,
    #[serde(default)]
    pub ends_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OauthExchangeResponse {
    pub access_token: String,
    pub token_type: String,
    pub customer: OauthExchangeCustomer,
    #[serde(default)]
    pub entitlement: Option<OauthExchangeEntitlement>,
    #[serde(default)]
    pub requires_plan_selection: bool,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GithubInstallationTokenPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub installation_id: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GithubInstallationTokenResponse {
    pub token: String,
    pub expires_at: String,
    pub installation_id: i64,
    pub account_login: String,
    pub account_type: String,
}
