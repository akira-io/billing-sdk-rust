use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("transport error: {0}")]
    Transport(#[from] reqwest::Error),
    #[error("billing api {status}: {code}")]
    Api { status: u16, code: String },
    #[error("decode response: {0}")]
    Decode(#[from] serde_json::Error),
}
