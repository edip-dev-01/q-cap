use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};

/// Minimal manifest for the MVP.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct QcapManifest {
    pub schema_version: String,      // e.g. "0.1.0"
    pub merkle_root: String,         // "blake3:<hex>"
    pub issuer: Option<String>,      // key fingerprint or issuer id
    pub created_at: String,          // RFC3339
    pub metadata: Value,             // free-form (title, description, tags, etc.)
}

impl QcapManifest {
    /// Construct a new manifest.
    pub fn new(
        schema_version: impl Into<String>,
        merkle_root: impl Into<String>,
        issuer: Option<String>,
        created_at: impl Into<String>,
        metadata: Value,
    ) -> Self {
        Self {
            schema_version: schema_version.into(),
            merkle_root: merkle_root.into(),
            issuer,
            created_at: created_at.into(),
            metadata,
        }
    }

    /// Create with current timestamp in RFC3339.
    pub fn new_with_now(
        schema_version: impl Into<String>,
        merkle_root: impl Into<String>,
        issuer: Option<String>,
        metadata: Value,
    ) -> Self {
        let ts = rfc3339_now();
        Self::new(schema_version, merkle_root, issuer, ts, metadata)
    }

    /// Serialize to pretty JSON bytes.
    pub fn to_json_bytes(&self) -> serde_json::Result<Vec<u8>> {
        serde_json::to_vec_pretty(self)
    }

    /// Deserialize from JSON bytes.
    pub fn from_json_bytes(bytes: &[u8]) -> serde_json::Result<Self> {
        serde_json::from_slice(bytes)
    }
}

fn rfc3339_now() -> String {
    // Use chrono if added later; for now, synthesize from SystemTime as an ISO-like string.
    // Fallback to unix timestamp seconds to avoid extra deps.
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    // Represent as seconds since epoch; acceptable for MVP, though not strict RFC3339.
    // Example: "unix-seconds:1733420000"
    format!("unix-seconds:{}", now.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_round_trip_json() {
        let m = QcapManifest::new_with_now(
            "0.1.0",
            "blake3:deadbeef",
            Some("issuer:abc".into()),
            serde_json::json!({
                "title": "Test Capsule",
                "tags": ["demo", "mvp"],
            }),
        );
        let bytes = m.to_json_bytes().expect("serialize");
        let m2 = QcapManifest::from_json_bytes(&bytes).expect("deserialize");
        assert_eq!(m.schema_version, m2.schema_version);
        assert_eq!(m.merkle_root, m2.merkle_root);
        assert_eq!(m.issuer, m2.issuer);
        assert_eq!(m.metadata, m2.metadata);
        assert!(m2.created_at.starts_with("unix-seconds:"));
    }
}
