//! # freshblu-client
//!
//! Rust client SDK for the FreshBlu IoT messaging platform.
//!
//! ```rust,no_run
//! use freshblu_client::FreshBluClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), freshblu_client::Error> {
//!     let mut client = FreshBluClient::new("https://api.freshblu.org");
//!
//!     // Register a device
//!     let device = client.register(serde_json::json!({"type": "sensor"})).await?;
//!     client.set_credentials(device.uuid, device.token.clone());
//!
//!     // Send a message
//!     client.message(&["target-uuid"], serde_json::json!({"temp": 22.5})).await?;
//!
//!     Ok(())
//! }
//! ```

#[cfg(feature = "http")]
mod http;
#[cfg(feature = "ws")]
mod ws;

pub use freshblu_core::device::{DeviceView, RegisterResponse};
pub use freshblu_core::message::Message;
pub use freshblu_core::subscription::{Subscription, SubscriptionType};
pub use freshblu_core::token::TokenRecord;

use base64::Engine;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("HTTP error ({status}): {message}")]
    Http { status: u16, message: String },

    #[cfg(feature = "http")]
    #[error("Request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[cfg(feature = "ws")]
    #[error("WebSocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("{0}")]
    Other(String),
}

/// Server status response.
#[derive(Debug, Deserialize)]
pub struct StatusResponse {
    pub meshblu: bool,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub connections: Option<u64>,
}

/// Token generation result.
#[derive(Debug, Deserialize)]
pub struct GenerateTokenResponse {
    pub uuid: String,
    pub token: String,
    #[serde(default)]
    pub tag: Option<String>,
}

/// HTTP client for FreshBlu.
pub struct FreshBluClient {
    base_url: String,
    uuid: Option<Uuid>,
    token: Option<String>,
    #[cfg(feature = "http")]
    http: reqwest::Client,
}

impl FreshBluClient {
    /// Create a new client pointing at the given server URL.
    pub fn new(base_url: &str) -> Self {
        let base_url = base_url.trim_end_matches('/').to_string();
        Self {
            base_url,
            uuid: None,
            token: None,
            #[cfg(feature = "http")]
            http: reqwest::Client::new(),
        }
    }

    /// Set authentication credentials.
    pub fn set_credentials(&mut self, uuid: Uuid, token: String) {
        self.uuid = Some(uuid);
        self.token = Some(token);
    }

    /// Get the current credentials.
    pub fn credentials(&self) -> Option<(Uuid, &str)> {
        match (&self.uuid, &self.token) {
            (Some(u), Some(t)) => Some((*u, t.as_str())),
            _ => None,
        }
    }

    fn auth_header(&self) -> Option<String> {
        let (uuid, token) = self.credentials()?;
        let creds = base64::engine::general_purpose::STANDARD
            .encode(format!("{}:{}", uuid, token));
        Some(format!("Basic {}", creds))
    }
}

#[cfg(feature = "ws")]
pub use ws::FreshBluWs;
