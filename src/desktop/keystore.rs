use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine as _;
use std::path::PathBuf;

use super::cipher::{generate_key, CipherError};
use super::keyring::TokenKeyring;

#[derive(Debug, thiserror::Error)]
pub enum KeyStoreError {
    #[error("keyring: {0}")]
    Keyring(String),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("decode base64: {0}")]
    Base64(#[from] base64::DecodeError),
    #[error("key has unexpected length")]
    KeyLength,
    #[error("cipher: {0}")]
    Cipher(#[from] CipherError),
}

/// Storage for a long-lived AES-256 encryption key.
///
/// In release builds the key is stored in the OS keychain. In debug builds the
/// key is also mirrored to a file under the app data directory so unsigned dev
/// rebuilds can still decrypt previously persisted data — the keychain ACL on
/// macOS rejects reads from ad-hoc signatures, so without the fallback the key
/// effectively rotates every time the binary is recompiled.
pub struct KeyStore {
    keyring: TokenKeyring,
    debug_file_path: Option<PathBuf>,
}

impl KeyStore {
    pub fn new(keyring: TokenKeyring) -> Self {
        Self { keyring, debug_file_path: None }
    }

    pub fn with_debug_file(mut self, path: PathBuf) -> Self {
        self.debug_file_path = Some(path);
        self
    }

    pub fn load_or_create(&self) -> Result<[u8; 32], KeyStoreError> {
        #[cfg(debug_assertions)]
        if let Some(path) = &self.debug_file_path {
            if let Ok(encoded) = std::fs::read_to_string(path) {
                if let Ok(key) = decode_key(encoded.trim()) {
                    return Ok(key);
                }
            }
        }

        match self.keyring.get().map_err(KeyStoreError::Keyring)? {
            Some(encoded) => {
                let key = decode_key(&encoded)?;
                #[cfg(debug_assertions)]
                self.write_debug_file(&encoded);
                Ok(key)
            }
            None => {
                let key = generate_key();
                let encoded = B64.encode(key);
                self.keyring
                    .set(&encoded)
                    .map_err(KeyStoreError::Keyring)?;
                #[cfg(debug_assertions)]
                self.write_debug_file(&encoded);
                Ok(key)
            }
        }
    }

    #[cfg(debug_assertions)]
    fn write_debug_file(&self, encoded: &str) {
        if let Some(path) = &self.debug_file_path {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(path, encoded);
        }
    }
}

fn decode_key(encoded: &str) -> Result<[u8; 32], KeyStoreError> {
    let bytes = B64.decode(encoded)?;
    bytes.try_into().map_err(|_| KeyStoreError::KeyLength)
}
