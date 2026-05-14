use hmac::{Hmac, Mac};
use rand::RngCore;
use sha2::{Digest, Sha256};

pub const HEADER_PRODUCT: &str = "X-Akira-Product";
pub const HEADER_TIMESTAMP: &str = "X-Akira-Timestamp";
pub const HEADER_NONCE: &str = "X-Akira-Nonce";
pub const HEADER_SIGNATURE: &str = "X-Akira-Signature";

pub fn canonical(
    product: &str,
    timestamp: i64,
    nonce: &str,
    method: &str,
    path: &str,
    body: &[u8],
) -> String {
    let body_hash = hex::encode(Sha256::digest(body));

    format!(
        "{product}\n{timestamp}\n{nonce}\n{method}\n{path}\n{body_hash}",
        product = product,
        timestamp = timestamp,
        nonce = nonce,
        method = method.to_ascii_uppercase(),
        path = path,
        body_hash = body_hash,
    )
}

pub fn sign(secret: &str, canonical: &str) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
        .expect("HMAC accepts arbitrary key length");
    mac.update(canonical.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

pub fn new_nonce() -> String {
    let mut bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}
