use akira_billing::oauth::{
    build_oauth_init_url, generate_oauth_state, generate_pkce_challenge, BuildOauthInitUrl,
};
use base64::engine::general_purpose::URL_SAFE_NO_PAD as B64URL;
use base64::Engine as _;
use sha2::{Digest, Sha256};

#[test]
fn pkce_challenge_is_sha256_of_verifier() {
    let pkce = generate_pkce_challenge();
    assert_eq!(pkce.method, "S256");

    let mut hasher = Sha256::new();
    hasher.update(pkce.verifier.as_bytes());
    let expected = B64URL.encode(hasher.finalize());
    assert_eq!(pkce.challenge, expected);
}

#[test]
fn state_is_url_safe_and_random() {
    let a = generate_oauth_state();
    let b = generate_oauth_state();
    assert_ne!(a, b);
    for c in a.chars() {
        assert!(c.is_ascii_alphanumeric() || c == '-' || c == '_');
    }
}

#[test]
fn init_url_encodes_required_params() {
    let url = build_oauth_init_url(BuildOauthInitUrl {
        base_url: "https://billing.akira.io/",
        provider: "google",
        product: "maintainer",
        redirect_uri: "http://127.0.0.1:53000/cb",
        code_challenge: "abc",
        code_challenge_method: None,
        state: Some("csrf-1"),
    });

    assert!(url.starts_with("https://billing.akira.io/auth/google?"));
    assert!(url.contains("product=maintainer"));
    assert!(url.contains("redirect_uri=http%3A%2F%2F127.0.0.1%3A53000%2Fcb"));
    assert!(url.contains("code_challenge=abc"));
    assert!(url.contains("code_challenge_method=S256"));
    assert!(url.contains("state=csrf-1"));
}

#[test]
fn init_url_omits_state_when_none() {
    let url = build_oauth_init_url(BuildOauthInitUrl {
        base_url: "https://billing.akira.io",
        provider: "github",
        product: "m",
        redirect_uri: "http://127.0.0.1:1/cb",
        code_challenge: "xyz",
        code_challenge_method: None,
        state: None,
    });
    assert!(!url.contains("state="));
}
