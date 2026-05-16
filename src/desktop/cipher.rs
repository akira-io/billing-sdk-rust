use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine as _;
use rand::RngCore;

const NONCE_LEN: usize = 12;

#[derive(Debug, thiserror::Error)]
pub enum CipherError {
    #[error("invalid key length")]
    KeyLength,
    #[error("encrypt failed")]
    EncryptFailed,
    #[error("decrypt failed")]
    DecryptFailed,
    #[error("decode base64: {0}")]
    Base64(#[from] base64::DecodeError),
}

#[derive(Clone)]
pub struct TokenCipher {
    key: [u8; 32],
}

impl TokenCipher {
    pub fn new(key: [u8; 32]) -> Self {
        Self { key }
    }

    pub fn encrypt(&self, plaintext: &str) -> Result<String, CipherError> {
        let cipher = Aes256Gcm::new_from_slice(&self.key).map_err(|_| CipherError::KeyLength)?;
        let mut nonce_bytes = [0u8; NONCE_LEN];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|_| CipherError::EncryptFailed)?;
        let mut combined = Vec::with_capacity(NONCE_LEN + ciphertext.len());
        combined.extend_from_slice(&nonce_bytes);
        combined.extend_from_slice(&ciphertext);
        Ok(B64.encode(combined))
    }

    pub fn decrypt(&self, encoded: &str) -> Result<String, CipherError> {
        let cipher = Aes256Gcm::new_from_slice(&self.key).map_err(|_| CipherError::KeyLength)?;
        let bytes = B64.decode(encoded)?;
        if bytes.len() <= NONCE_LEN {
            return Err(CipherError::DecryptFailed);
        }
        let (nonce_bytes, ciphertext) = bytes.split_at(NONCE_LEN);
        let nonce = Nonce::from_slice(nonce_bytes);
        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| CipherError::DecryptFailed)?;
        String::from_utf8(plaintext).map_err(|_| CipherError::DecryptFailed)
    }
}

pub fn generate_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);
    key
}
