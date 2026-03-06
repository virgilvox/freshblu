use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A stored token record (bcrypt hashed)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenRecord {
    pub uuid: Uuid,
    /// The bcrypt hash of the token
    pub hash: String,
    /// When this token was created
    pub created_at: DateTime<Utc>,
    /// Optional expiry timestamp (Unix epoch seconds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_on: Option<i64>,
    /// Optional tag / label
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
}

impl TokenRecord {
    pub fn new(uuid: Uuid, hash: String) -> Self {
        Self {
            uuid,
            hash,
            created_at: Utc::now(),
            expires_on: None,
            tag: None,
        }
    }

    pub fn is_expired(&self) -> bool {
        if let Some(exp) = self.expires_on {
            let now = Utc::now().timestamp();
            now > exp
        } else {
            false
        }
    }
}

/// Options for generating a token with expiry/tags
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GenerateTokenOptions {
    pub expires_on: Option<i64>,
    pub tag: Option<String>,
}
