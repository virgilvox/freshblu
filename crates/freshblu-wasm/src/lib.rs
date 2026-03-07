/// FreshBlu WASM Client
///
/// Compiles to WebAssembly, works in browser and Node.js.
/// Provides the same API as the original Meshblu JS clients.

use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    fn error(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

/// Configuration for the FreshBlu client
#[wasm_bindgen]
#[derive(Clone)]
pub struct FreshBluConfig {
    hostname: String,
    port: u16,
    secure: bool,
    uuid: Option<String>,
    token: Option<String>,
}

#[wasm_bindgen]
impl FreshBluConfig {
    #[wasm_bindgen(constructor)]
    pub fn new(hostname: String, port: u16) -> Self {
        Self {
            hostname,
            port,
            secure: false,
            uuid: None,
            token: None,
        }
    }

    pub fn set_uuid(&mut self, uuid: String) {
        self.uuid = Some(uuid);
    }

    pub fn set_token(&mut self, token: String) {
        self.token = Some(token);
    }

    pub fn set_secure(&mut self, secure: bool) {
        self.secure = secure;
    }

    pub fn http_base_url(&self) -> String {
        let scheme = if self.secure { "https" } else { "http" };
        format!("{}://{}:{}", scheme, self.hostname, self.port)
    }

    pub fn ws_url(&self) -> String {
        let scheme = if self.secure { "wss" } else { "ws" };
        format!("{}://{}:{}/ws", scheme, self.hostname, self.port)
    }
}

/// HTTP-based FreshBlu client (works in browser and Node.js)
#[wasm_bindgen]
pub struct FreshBluHttp {
    config: FreshBluConfig,
}

#[wasm_bindgen]
impl FreshBluHttp {
    #[wasm_bindgen(constructor)]
    pub fn new(config: FreshBluConfig) -> Self {
        Self { config }
    }

    fn auth_header(&self) -> Option<String> {
        let uuid = self.config.uuid.as_ref()?;
        let token = self.config.token.as_ref()?;
        let creds = format!("{}:{}", uuid, token);
        Some(format!("Basic {}", base64_encode(creds.as_bytes())))
    }

    fn base_url(&self) -> String {
        self.config.http_base_url()
    }

    /// Register a new device. Returns JSON string with uuid and token.
    #[wasm_bindgen]
    pub async fn register(&self, properties: JsValue) -> Result<JsValue, JsValue> {
        let props: Value = serde_wasm_bindgen::from_value(properties)
            .unwrap_or(Value::Object(Default::default()));

        let resp = gloo_net::http::Request::post(&format!("{}/devices", self.base_url()))
            .json(&props)
            .map_err(|e| JsValue::from_str(&e.to_string()))?
            .send()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let result: Value = resp
            .json()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Get authenticated device info
    #[wasm_bindgen]
    pub async fn whoami(&self) -> Result<JsValue, JsValue> {
        let auth = self
            .auth_header()
            .ok_or_else(|| JsValue::from_str("No credentials"))?;

        let resp = gloo_net::http::Request::get(&format!("{}/whoami", self.base_url()))
            .header("Authorization", &auth)
            .send()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let result: Value = resp
            .json()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Get a device by UUID
    #[wasm_bindgen]
    pub async fn get_device(&self, uuid: String) -> Result<JsValue, JsValue> {
        let auth = self
            .auth_header()
            .ok_or_else(|| JsValue::from_str("No credentials"))?;

        let resp = gloo_net::http::Request::get(&format!("{}/devices/{}", self.base_url(), uuid))
            .header("Authorization", &auth)
            .send()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let result: Value = resp
            .json()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Update a device
    #[wasm_bindgen]
    pub async fn update_device(&self, uuid: String, properties: JsValue) -> Result<JsValue, JsValue> {
        let auth = self
            .auth_header()
            .ok_or_else(|| JsValue::from_str("No credentials"))?;
        let props: Value = serde_wasm_bindgen::from_value(properties)
            .unwrap_or(Value::Object(Default::default()));

        let resp = gloo_net::http::Request::put(&format!("{}/devices/{}", self.base_url(), uuid))
            .header("Authorization", &auth)
            .json(&props)
            .map_err(|e| JsValue::from_str(&e.to_string()))?
            .send()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let result: Value = resp
            .json()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Send a message
    #[wasm_bindgen]
    pub async fn message(&self, msg: JsValue) -> Result<JsValue, JsValue> {
        let auth = self
            .auth_header()
            .ok_or_else(|| JsValue::from_str("No credentials"))?;
        let msg_val: Value = serde_wasm_bindgen::from_value(msg)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let resp = gloo_net::http::Request::post(&format!("{}/messages", self.base_url()))
            .header("Authorization", &auth)
            .json(&msg_val)
            .map_err(|e| JsValue::from_str(&e.to_string()))?
            .send()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let result: Value = resp
            .json()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Search devices
    #[wasm_bindgen]
    pub async fn search(&self, query: JsValue) -> Result<JsValue, JsValue> {
        let auth = self
            .auth_header()
            .ok_or_else(|| JsValue::from_str("No credentials"))?;
        let query_val: Value = serde_wasm_bindgen::from_value(query)
            .unwrap_or(Value::Object(Default::default()));

        let resp = gloo_net::http::Request::post(&format!("{}/devices/search", self.base_url()))
            .header("Authorization", &auth)
            .json(&query_val)
            .map_err(|e| JsValue::from_str(&e.to_string()))?
            .send()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let result: Value = resp
            .json()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Create a subscription
    #[wasm_bindgen]
    pub async fn subscribe(
        &self,
        subscriber_uuid: String,
        emitter_uuid: String,
        subscription_type: String,
    ) -> Result<JsValue, JsValue> {
        let auth = self
            .auth_header()
            .ok_or_else(|| JsValue::from_str("No credentials"))?;

        let body = serde_json::json!({
            "emitterUuid": emitter_uuid,
            "subscriberUuid": subscriber_uuid,
            "type": subscription_type
        });

        let resp = gloo_net::http::Request::post(&format!(
            "{}/devices/{}/subscriptions",
            self.base_url(),
            subscriber_uuid
        ))
        .header("Authorization", &auth)
        .json(&body)
        .map_err(|e| JsValue::from_str(&e.to_string()))?
        .send()
        .await
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let result: Value = resp
            .json()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Get server status
    #[wasm_bindgen]
    pub async fn status(&self) -> Result<JsValue, JsValue> {
        let resp = gloo_net::http::Request::get(&format!("{}/status", self.base_url()))
            .send()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let result: Value = resp
            .json()
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

fn base64_encode(input: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = if chunk.len() > 1 { chunk[1] as usize } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as usize } else { 0 };
        out.push(CHARS[(b0 >> 2)] as char);
        out.push(CHARS[((b0 & 3) << 4) | (b1 >> 4)] as char);
        out.push(if chunk.len() > 1 { CHARS[((b1 & 15) << 2) | (b2 >> 6)] as char } else { '=' });
        out.push(if chunk.len() > 2 { CHARS[b2 & 63] as char } else { '=' });
    }
    out
}

#[wasm_bindgen(start)]
pub fn main() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
