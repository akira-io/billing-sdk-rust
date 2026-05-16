use serde::{Deserialize, Serialize};

use crate::license::{decode_license, verify_license, DecodedLicense};
use crate::types::SignedLicense;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotEntry {
    pub signed: SignedLicense,
    pub fetched_at_unix: i64,
}

/// Storage-agnostic helper for the offline license snapshot pattern.
///
/// Consumers persist `SnapshotEntry` somewhere (sqlite, app data file, ...) and
/// pass the bytes back here to verify and decode.
pub struct LicenseSnapshotStore;

impl LicenseSnapshotStore {
    pub fn serialize(entry: &SnapshotEntry) -> Result<String, serde_json::Error> {
        serde_json::to_string(entry)
    }

    pub fn deserialize(raw: &str) -> Result<SnapshotEntry, serde_json::Error> {
        serde_json::from_str(raw)
    }

    pub fn verify_and_decode(
        entry: &SnapshotEntry,
        public_key_b64: &str,
    ) -> Result<DecodedLicense, String> {
        let ok = verify_license(&entry.signed, public_key_b64).map_err(|e| e.to_string())?;
        if !ok {
            return Err("license signature mismatch".to_string());
        }
        decode_license(&entry.signed).map_err(|e| e.to_string())
    }
}
