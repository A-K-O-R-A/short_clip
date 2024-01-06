use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Metadata {
    pub version: u8,
    pub created_at: u64,
    pub expires_at: Option<u64>,
    pub author: String,
    pub content_type: String,
}

impl Metadata {
    /// Creates a new Metadata struct
    ///
    /// `content_type` should be a valid mime type
    ///
    /// `ttl` should be supplied in seconds
    pub fn new(author: &str, content_type: &str, ttl: Option<u64>) -> Self {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        Self {
            version: 1,
            created_at: since_the_epoch.as_secs(),
            expires_at: ttl.map(|t| since_the_epoch.as_secs() + t),
            author: author.to_owned(),
            content_type: content_type.to_owned(),
        }
    }

    pub fn from_str(s: &str) -> Result<Metadata, serde_json::Error> {
        serde_json::from_str(s)
    }

    pub fn from_slice(s: &[u8]) -> Result<Metadata, serde_json::Error> {
        serde_json::from_slice(s)
    }

    pub fn to_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}
