use std::time::{SystemTime, UNIX_EPOCH};

use reqwest::header::{HeaderMap, HeaderName, HeaderValue, ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use reqwest::Method;
use serde::de::DeserializeOwned;

use crate::error::Error;
use crate::signature::{canonical, new_nonce, sign};
use crate::types::{
    Customer, EntitlementsResponse, GithubAppInfo, GithubInstallationTokenPayload,
    GithubInstallationTokenResponse, GithubUserInstallationsResponse, IssuedDownload, IssuedTrial,
    LicenseActivatePayload, LicenseActivateResponse, LicenseCheckPayload, LicenseCheckResponse,
    LicensePublicKeysResponse, LicenseRefreshPayload, LicenseSyncUsagePayload,
    LicenseSyncUsageResponse, OauthExchangePayload, OauthExchangeResponse, OauthProvidersResponse,
    OtpRequestPayload, OtpVerifyPayload, OtpVerifyResponse, PlansResponse, PortalLink,
    ReleaseManifest, UsagePayload, UsageResponse,
};

const H_PRODUCT: HeaderName = HeaderName::from_static("x-akira-product");
const H_TIMESTAMP: HeaderName = HeaderName::from_static("x-akira-timestamp");
const H_NONCE: HeaderName = HeaderName::from_static("x-akira-nonce");
const H_SIGNATURE: HeaderName = HeaderName::from_static("x-akira-signature");

#[derive(Debug, Clone)]
pub struct Client {
    base_url: String,
    product_slug: String,
    product_secret: String,
    customer_token: Option<String>,
    http: reqwest::Client,
}

impl Client {
    pub fn new(
        base_url: impl Into<String>,
        product_slug: impl Into<String>,
        product_secret: impl Into<String>,
    ) -> Self {
        Self {
            base_url: base_url.into(),
            product_slug: product_slug.into(),
            product_secret: product_secret.into(),
            customer_token: None,
            http: reqwest::Client::new(),
        }
    }

    pub fn set_customer_token(&mut self, token: impl Into<String>) {
        self.customer_token = Some(token.into());
    }

    pub fn clear_customer_token(&mut self) {
        self.customer_token = None;
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn product_slug(&self) -> &str {
        &self.product_slug
    }

    pub fn customer_token(&self) -> Option<&str> {
        self.customer_token.as_deref()
    }

    /// GET /api/v1/downloads/{product}/releases/{channel}/latest
    pub async fn latest_release(&self, channel: &str) -> Result<ReleaseManifest, Error> {
        let path = format!(
            "/api/v1/downloads/{}/releases/{}/latest",
            self.product_slug, channel
        );
        self.do_request::<_, ReleaseManifest>(Method::GET, &path, None::<&()>)
            .await
    }

    /// GET /api/v1/downloads/{product}/{channel}/{platform} (Accept: application/json)
    /// Platform is "os-arch", e.g. "macos-arm64".
    pub async fn issue_download(
        &self,
        channel: &str,
        platform: &str,
    ) -> Result<IssuedDownload, Error> {
        let path = format!(
            "/api/v1/downloads/{}/{}/{}",
            self.product_slug, channel, platform
        );
        self.do_request::<_, IssuedDownload>(Method::GET, &path, None::<&()>)
            .await
    }

    /// Posts the completion beacon for an issued event. The URL is the
    /// absolute beacon_url returned in IssuedDownload, including the sig
    /// query string. No HMAC signing required.
    pub async fn complete_download(&self, beacon_url: &str) -> Result<(), Error> {
        let resp = self
            .http
            .post(beacon_url)
            .header(ACCEPT, "application/json")
            .send()
            .await?;
        let status = resp.status();
        if !status.is_success() {
            let bytes = resp.bytes().await.unwrap_or_default();
            let code = serde_json::from_slice::<serde_json::Value>(&bytes)
                .ok()
                .and_then(|v| v.get("error").and_then(|e| e.as_str()).map(String::from))
                .unwrap_or_else(|| String::from_utf8_lossy(&bytes).into_owned());
            return Err(Error::Api {
                status: status.as_u16(),
                code,
            });
        }
        Ok(())
    }

    pub async fn plans(&self) -> Result<PlansResponse, Error> {
        let path = format!("/api/v1/products/{}/plans", self.product_slug);
        self.do_request::<_, PlansResponse>(Method::GET, &path, None::<&()>)
            .await
    }

    pub async fn start_trial(&self, plan_key: Option<&str>) -> Result<IssuedTrial, Error> {
        let path = format!("/api/v1/me/products/{}/trial", self.product_slug);
        let body = plan_key.map(|p| serde_json::json!({ "plan": p }));
        self.do_request::<_, IssuedTrial>(Method::POST, &path, body.as_ref())
            .await
    }

    pub async fn request_otp(&self, payload: OtpRequestPayload<'_>) -> Result<(), Error> {
        self.do_request::<_, serde_json::Value>(
            Method::POST,
            "/api/auth/customer/otp/request",
            Some(&payload),
        )
        .await
        .map(|_| ())
    }

    pub async fn verify_otp(
        &mut self,
        payload: OtpVerifyPayload<'_>,
    ) -> Result<OtpVerifyResponse, Error> {
        let resp = self
            .do_request::<_, OtpVerifyResponse>(
                Method::POST,
                "/api/auth/customer/otp/verify",
                Some(&payload),
            )
            .await?;
        self.set_customer_token(resp.access_token.clone());

        Ok(resp)
    }

    /// GET /api/me — current authenticated customer.
    pub async fn customer_me(&self) -> Result<Customer, Error> {
        self.do_request::<_, Customer>(Method::GET, "/api/me", None::<&()>)
            .await
    }

    /// POST /api/licenses/check — runtime feature gate check.
    pub async fn license_check(
        &self,
        payload: LicenseCheckPayload<'_>,
    ) -> Result<LicenseCheckResponse, Error> {
        self.do_request::<_, LicenseCheckResponse>(
            Method::POST,
            "/api/licenses/check",
            Some(&payload),
        )
        .await
    }

    /// POST /api/licenses/activate — activate device, returns signed license envelope.
    pub async fn license_activate(
        &self,
        payload: LicenseActivatePayload<'_>,
    ) -> Result<LicenseActivateResponse, Error> {
        self.do_request::<_, LicenseActivateResponse>(
            Method::POST,
            "/api/licenses/activate",
            Some(&payload),
        )
        .await
    }

    /// POST /api/licenses/refresh — refresh signed license envelope.
    pub async fn license_refresh(
        &self,
        payload: LicenseRefreshPayload<'_>,
    ) -> Result<LicenseActivateResponse, Error> {
        self.do_request::<_, LicenseActivateResponse>(
            Method::POST,
            "/api/licenses/refresh",
            Some(&payload),
        )
        .await
    }

    pub async fn license_sync_usage(
        &self,
        payload: LicenseSyncUsagePayload<'_>,
    ) -> Result<LicenseSyncUsageResponse, Error> {
        self.do_request::<_, LicenseSyncUsageResponse>(
            Method::POST,
            "/api/licenses/sync-usage",
            Some(&payload),
        )
        .await
    }

    pub async fn list_oauth_providers(
        &self,
        product: &str,
    ) -> Result<OauthProvidersResponse, Error> {
        let path = format!("/api/v1/products/{}/auth/providers", urlencode(product));
        self.do_request::<_, OauthProvidersResponse>(Method::GET, &path, None::<&()>)
            .await
    }

    pub async fn exchange_oauth_code(
        &mut self,
        payload: OauthExchangePayload<'_>,
    ) -> Result<OauthExchangeResponse, Error> {
        let resp = self
            .do_request::<_, OauthExchangeResponse>(
                Method::POST,
                "/api/auth/oauth/exchange",
                Some(&payload),
            )
            .await?;
        self.set_customer_token(resp.access_token.clone());

        Ok(resp)
    }

    pub async fn github_installation_token(
        &self,
        payload: GithubInstallationTokenPayload,
    ) -> Result<GithubInstallationTokenResponse, Error> {
        self.do_request::<_, GithubInstallationTokenResponse>(
            Method::POST,
            "/api/me/github/installation-token",
            Some(&payload),
        )
        .await
    }

    pub async fn me_github_installations(&self) -> Result<GithubUserInstallationsResponse, Error> {
        self.do_request::<_, GithubUserInstallationsResponse>(
            Method::GET,
            "/api/me/github/installations",
            None::<&()>,
        )
        .await
    }

    /// GET /api/me/entitlements — full customer entitlement + device snapshot.
    pub async fn entitlements(&self) -> Result<EntitlementsResponse, Error> {
        self.do_request::<_, EntitlementsResponse>(Method::GET, "/api/me/entitlements", None::<&()>)
            .await
    }

    /// GET /api/billing/portal — Stripe customer portal short-lived URL.
    pub async fn billing_portal(&self, return_url: &str) -> Result<PortalLink, Error> {
        let path = format!("/api/billing/portal?return_url={}", urlencode(return_url),);
        self.do_request::<_, PortalLink>(Method::GET, &path, None::<&()>)
            .await
    }

    /// POST /api/me/usage — check or increment per-day usage counter.
    pub async fn track_usage(&self, payload: UsagePayload<'_>) -> Result<UsageResponse, Error> {
        self.do_request::<_, UsageResponse>(Method::POST, "/api/me/usage", Some(&payload))
            .await
    }

    /// POST /api/v1/usage/anonymous — anonymous usage tracking (HMAC only, no bearer).
    /// Use when the customer is not yet authenticated; the server applies the
    /// limits defined on the product's anonymous_plan.
    pub async fn track_anonymous_usage(
        &self,
        payload: UsagePayload<'_>,
    ) -> Result<UsageResponse, Error> {
        self.do_request::<_, UsageResponse>(Method::POST, "/api/v1/usage/anonymous", Some(&payload))
            .await
    }

    /// GET /api/v1/github/app — public GitHub App metadata (slug + install URL).
    pub async fn github_app_info(&self) -> Result<GithubAppInfo, Error> {
        let url = format!("{}/api/v1/github/app", self.base_url.trim_end_matches('/'));
        let resp = self
            .http
            .get(&url)
            .header(ACCEPT, "application/json")
            .send()
            .await?;
        let status = resp.status();
        let bytes = resp.bytes().await?;
        if !status.is_success() {
            let code = serde_json::from_slice::<serde_json::Value>(&bytes)
                .ok()
                .and_then(|v| v.get("error").and_then(|e| e.as_str()).map(String::from))
                .unwrap_or_default();
            return Err(Error::Api {
                status: status.as_u16(),
                code,
            });
        }
        serde_json::from_slice::<GithubAppInfo>(&bytes).map_err(|e| Error::Api {
            status: status.as_u16(),
            code: format!("decode response: {e}"),
        })
    }

    /// GET /api/v1/license-keys/public — list registered Ed25519 verification keys.
    /// Public endpoint, no HMAC or bearer required.
    pub async fn public_license_keys(&self) -> Result<LicensePublicKeysResponse, Error> {
        let url = format!(
            "{}/api/v1/license-keys/public",
            self.base_url.trim_end_matches('/')
        );
        let resp = self
            .http
            .get(&url)
            .header(ACCEPT, "application/json")
            .send()
            .await?;
        let status = resp.status();
        let bytes = resp.bytes().await?;
        if !status.is_success() {
            let code = serde_json::from_slice::<serde_json::Value>(&bytes)
                .ok()
                .and_then(|v| v.get("error").and_then(|e| e.as_str()).map(String::from))
                .unwrap_or_default();
            return Err(Error::Api {
                status: status.as_u16(),
                code,
            });
        }
        serde_json::from_slice::<LicensePublicKeysResponse>(&bytes).map_err(|e| Error::Api {
            status: status.as_u16(),
            code: format!("decode response: {e}"),
        })
    }

    async fn do_request<B: serde::Serialize, T: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        body: Option<&B>,
    ) -> Result<T, Error> {
        let body_bytes = match body {
            Some(b) => serde_json::to_vec(b).map_err(|e| Error::Api {
                status: 0,
                code: format!("serialize body: {e}"),
            })?,
            None => Vec::new(),
        };

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let nonce = new_nonce();
        let canonical_str = canonical(
            &self.product_slug,
            timestamp,
            &nonce,
            method.as_str(),
            path,
            &body_bytes,
        );
        let signature = sign(&self.product_secret, &canonical_str);

        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        if !body_bytes.is_empty() {
            headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        }
        headers.insert(H_PRODUCT, header_value(&self.product_slug)?);
        headers.insert(H_TIMESTAMP, header_value(&timestamp.to_string())?);
        headers.insert(H_NONCE, header_value(&nonce)?);
        headers.insert(H_SIGNATURE, header_value(&signature)?);
        if let Some(token) = &self.customer_token {
            headers.insert(AUTHORIZATION, header_value(&format!("Bearer {token}"))?);
        }

        let url = format!("{}{}", self.base_url.trim_end_matches('/'), path);
        let resp = self
            .http
            .request(method, &url)
            .headers(headers)
            .body(body_bytes)
            .send()
            .await?;
        let status = resp.status();
        let bytes = resp.bytes().await?;

        if !status.is_success() {
            let code = match serde_json::from_slice::<serde_json::Value>(&bytes) {
                Ok(v) => v
                    .get("error")
                    .and_then(|e| e.as_str())
                    .unwrap_or("")
                    .to_string(),
                Err(_) => String::from_utf8_lossy(&bytes).into_owned(),
            };
            return Err(Error::Api {
                status: status.as_u16(),
                code,
            });
        }

        serde_json::from_slice::<T>(&bytes).map_err(|e| Error::Api {
            status: status.as_u16(),
            code: format!("decode response: {e}"),
        })
    }
}

fn header_value(s: &str) -> Result<HeaderValue, Error> {
    HeaderValue::from_str(s).map_err(|_| Error::Api {
        status: 0,
        code: "invalid header value".into(),
    })
}

fn urlencode(value: &str) -> String {
    value
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}
