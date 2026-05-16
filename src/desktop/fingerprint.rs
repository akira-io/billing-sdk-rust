use serde::Serialize;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize)]
pub struct DeviceFingerprint {
    pub fingerprint: String,
    pub platform: String,
    pub app_version: String,
}

/// Stable per-machine fingerprint, derived from the OS machine id, platform
/// and application version. Apps should pass their `CARGO_PKG_VERSION`.
pub fn device_fingerprint(app_version: &str) -> DeviceFingerprint {
    let machine_id = machine_uid::get().unwrap_or_else(|_| "unknown".to_string());
    let platform = std::env::consts::OS.to_string();

    let mut hasher = Sha256::new();
    hasher.update(machine_id.as_bytes());
    hasher.update(b"::");
    hasher.update(platform.as_bytes());
    hasher.update(b"::");
    hasher.update(app_version.as_bytes());

    DeviceFingerprint {
        fingerprint: format!("{:x}", hasher.finalize()),
        platform,
        app_version: app_version.to_string(),
    }
}
