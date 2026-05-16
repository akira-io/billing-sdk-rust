use akira_billing::{canonical, sign};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Vector {
    name: String,
    product: String,
    timestamp: i64,
    nonce: String,
    method: String,
    path: String,
    body: String,
    secret: String,
    canonical: String,
    signature: String,
}

fn load_vectors() -> Vec<Vector> {
    let raw = std::fs::read_to_string("tests/fixtures/signature-vectors.json")
        .expect("read signature-vectors.json");
    serde_json::from_str(&raw).expect("decode vectors")
}

#[test]
fn canonical_matches_fixtures() {
    for v in load_vectors() {
        let got = canonical(
            &v.product,
            v.timestamp,
            &v.nonce,
            &v.method,
            &v.path,
            v.body.as_bytes(),
        );
        assert_eq!(got, v.canonical, "canonical mismatch in {}", v.name);
    }
}

#[test]
fn signature_matches_fixtures() {
    for v in load_vectors() {
        let canon = canonical(
            &v.product,
            v.timestamp,
            &v.nonce,
            &v.method,
            &v.path,
            v.body.as_bytes(),
        );
        let got = sign(&v.secret, &canon);
        assert_eq!(got, v.signature, "signature mismatch in {}", v.name);
    }
}

#[test]
fn nonce_is_32_hex_chars() {
    let nonce = akira_billing::new_nonce();
    assert_eq!(nonce.len(), 32);
    assert!(nonce
        .chars()
        .all(|c| c.is_ascii_hexdigit() && (c.is_ascii_digit() || c.is_ascii_lowercase())));
}
