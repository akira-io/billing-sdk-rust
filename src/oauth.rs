use base64::engine::general_purpose::URL_SAFE_NO_PAD as B64URL;
use base64::Engine as _;
use rand::RngCore;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
pub struct PkceChallenge {
    pub verifier: String,
    pub challenge: String,
    pub method: &'static str,
}

pub fn generate_pkce_challenge() -> PkceChallenge {
    let mut bytes = [0u8; 48];
    rand::thread_rng().fill_bytes(&mut bytes);
    let verifier = B64URL.encode(bytes);

    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let challenge = B64URL.encode(hasher.finalize());

    PkceChallenge {
        verifier,
        challenge,
        method: "S256",
    }
}

pub fn generate_oauth_state() -> String {
    let mut bytes = [0u8; 24];
    rand::thread_rng().fill_bytes(&mut bytes);
    B64URL.encode(bytes)
}

pub struct BuildOauthInitUrl<'a> {
    pub base_url: &'a str,
    pub provider: &'a str,
    pub product: &'a str,
    pub redirect_uri: &'a str,
    pub code_challenge: &'a str,
    pub code_challenge_method: Option<&'a str>,
    pub state: Option<&'a str>,
}

pub fn build_oauth_init_url(opts: BuildOauthInitUrl<'_>) -> String {
    let base = opts.base_url.trim_end_matches('/');
    let method = opts.code_challenge_method.unwrap_or("S256");

    let mut query = format!(
        "product={}&redirect_uri={}&code_challenge={}&code_challenge_method={}",
        urlencode(opts.product),
        urlencode(opts.redirect_uri),
        urlencode(opts.code_challenge),
        urlencode(method),
    );

    if let Some(state) = opts.state {
        query.push_str(&format!("&state={}", urlencode(state)));
    }

    format!("{}/auth/{}?{}", base, urlencode(opts.provider), query)
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
