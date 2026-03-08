use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;

use futures::future::join_all;

use freshblu_core::{
    device::Device,
    forwarder::{ForwarderEntry, ForwarderEvent, MeshbluForwarder, WebhookForwarder},
    message::{DeviceEvent, Message},
    token::GenerateTokenOptions,
};
use freshblu_store::DynStore;
use serde_json::Value;
use tracing::{debug, warn};
use url::Url;
use uuid::Uuid;

use crate::bus::DynBus;
use crate::metrics::{WEBHOOKS_FAILED, WEBHOOKS_SENT};

const MAX_FORWARD_DEPTH: usize = 5;
const MAX_FORWARDERS_PER_EVENT: usize = 10;

pub struct WebhookExecutor {
    client: reqwest::Client,
    store: DynStore,
    bus: DynBus,
    allow_localhost: bool,
}

impl WebhookExecutor {
    pub fn new(store: DynStore, bus: DynBus) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("reqwest client");
        Self {
            client,
            store,
            bus,
            allow_localhost: false,
        }
    }

    /// Create an executor that allows localhost URLs (for testing only).
    #[cfg(test)]
    pub fn new_with_localhost(store: DynStore, bus: DynBus) -> Self {
        let mut this = Self::new(store, bus);
        this.allow_localhost = true;
        this
    }

    /// Allow localhost webhooks (for testing).
    pub fn set_allow_localhost(&mut self, allow: bool) {
        self.allow_localhost = allow;
    }

    /// Fire forwarders for a device event. `forwarded_from` tracks UUIDs to detect loops.
    pub async fn execute(
        self: &Arc<Self>,
        device: &Device,
        event: ForwarderEvent,
        payload: &Value,
        forwarded_from: &[Uuid],
    ) {
        let forwarders = match &device.meshblu.forwarders {
            Some(f) => f.get(event),
            None => return,
        };

        if forwarders.is_empty() {
            return;
        }

        // Cap forwarders per event to prevent abuse
        // Fire webhooks concurrently, meshblu forwarders sequentially (they mutate state)
        let mut webhook_futures = Vec::new();
        let mut meshblu_entries = Vec::new();

        for entry in forwarders.iter().take(MAX_FORWARDERS_PER_EVENT) {
            match entry {
                ForwarderEntry::Webhook(wh) => {
                    webhook_futures.push(self.fire_webhook(device, wh, payload));
                }
                ForwarderEntry::Meshblu(mf) => {
                    meshblu_entries.push(mf);
                }
            }
        }

        // Fire all webhooks concurrently
        join_all(webhook_futures).await;

        // Process meshblu forwarders sequentially (loop detection is order-dependent)
        for mf in meshblu_entries {
            self.fire_meshblu(device, mf, payload, forwarded_from).await;
        }
    }

    async fn fire_webhook(&self, device: &Device, wh: &WebhookForwarder, payload: &Value) {
        if !is_safe_url(&wh.url, self.allow_localhost) {
            warn!("Webhook URL rejected (SSRF protection): {}", wh.url);
            WEBHOOKS_FAILED.inc();
            return;
        }

        let mut req = match wh.method.to_uppercase().as_str() {
            "GET" => self.client.get(&wh.url),
            "PUT" => self.client.put(&wh.url),
            "DELETE" => self.client.delete(&wh.url),
            _ => self.client.post(&wh.url),
        };

        req = req
            .header("X-Meshblu-Uuid", device.uuid.to_string())
            .header("Content-Type", "application/json")
            .json(payload);

        if wh.generate_and_forward_meshblu_credentials {
            let opts = GenerateTokenOptions {
                expires_on: Some(chrono::Utc::now().timestamp() + 300), // 5 min expiry
                tag: Some("webhook-credential".to_string()),
            };
            if let Ok((_, plaintext)) = self.store.generate_token(&device.uuid, opts).await {
                let cred = format!("{}:{}", device.uuid, plaintext);
                let encoded = base64::Engine::encode(
                    &base64::engine::general_purpose::STANDARD,
                    cred.as_bytes(),
                );
                req = req.header("Authorization", format!("Bearer {}", encoded));
            }
        }

        match req.send().await {
            Ok(resp) => {
                WEBHOOKS_SENT.inc();
                debug!("Webhook to {} returned {}", wh.url, resp.status().as_u16());
            }
            Err(e) => {
                WEBHOOKS_FAILED.inc();
                warn!("Webhook to {} failed: {}", wh.url, e);
            }
        }
    }

    async fn fire_meshblu(
        &self,
        device: &Device,
        _mf: &MeshbluForwarder,
        payload: &Value,
        forwarded_from: &[Uuid],
    ) {
        // Loop detection
        if forwarded_from.len() >= MAX_FORWARD_DEPTH {
            warn!(
                "Meshblu forwarder loop depth exceeded for device {}",
                device.uuid
            );
            return;
        }
        if forwarded_from.contains(&device.uuid) {
            warn!(
                "Meshblu forwarder circular loop detected for device {}",
                device.uuid
            );
            return;
        }

        // Re-emit as a message from this device to itself
        let msg = Message {
            devices: vec![device.uuid.to_string()],
            from_uuid: Some(device.uuid),
            topic: Some("forwarder".to_string()),
            payload: Some(payload.clone()),
            metadata: None,
            extra: Default::default(),
        };

        let event = DeviceEvent::Message(msg);
        let _ = self.bus.publish(&device.uuid, event).await;
    }
}

/// SSRF protection: reject URLs targeting localhost, private IPs, link-local, and metadata endpoints.
fn is_safe_url(url_str: &str, allow_localhost: bool) -> bool {
    let url = match Url::parse(url_str) {
        Ok(u) => u,
        Err(_) => return false,
    };

    // Only allow http/https
    match url.scheme() {
        "http" | "https" => {}
        _ => return false,
    }

    let host = match url.host_str() {
        Some(h) => h,
        None => return false,
    };

    // Allow localhost in test mode
    if allow_localhost
        && (host == "localhost" || host == "127.0.0.1" || host == "::1" || host == "[::1]")
    {
        return true;
    }

    // Block common localhost names
    if host == "localhost"
        || host == "127.0.0.1"
        || host == "::1"
        || host == "[::1]"
        || host == "0.0.0.0"
    {
        return false;
    }

    // Block AWS/GCP/Azure metadata endpoints
    if host == "169.254.169.254" || host == "metadata.google.internal" {
        return false;
    }

    // Parse and check IP addresses for private ranges
    if let Ok(ip) = host.parse::<IpAddr>() {
        return !is_private_ip(ip);
    }

    // Block hosts that look like they resolve to internal services
    if host.ends_with(".internal") || host.ends_with(".local") || host.ends_with(".localhost") {
        return false;
    }

    true
}

fn is_private_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_loopback()
                || v4.is_private()
                || v4.is_link_local()
                || v4.is_broadcast()
                || v4.is_unspecified()
                || v4.octets()[0] == 169 && v4.octets()[1] == 254 // link-local
        }
        IpAddr::V6(v6) => v6.is_loopback() || v6.is_unspecified(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_url_allows_public() {
        assert!(is_safe_url("https://example.com/hook", false));
        assert!(is_safe_url("http://api.external.io/webhook", false));
    }

    #[test]
    fn safe_url_blocks_localhost() {
        assert!(!is_safe_url("http://localhost/hook", false));
        assert!(!is_safe_url("http://127.0.0.1/hook", false));
        assert!(!is_safe_url("http://[::1]/hook", false));
        assert!(!is_safe_url("http://0.0.0.0/hook", false));
    }

    #[test]
    fn safe_url_allows_localhost_when_enabled() {
        assert!(is_safe_url("http://127.0.0.1/hook", true));
        assert!(is_safe_url("http://localhost/hook", true));
    }

    #[test]
    fn safe_url_blocks_private_ips() {
        assert!(!is_safe_url("http://10.0.0.1/hook", false));
        assert!(!is_safe_url("http://192.168.1.1/hook", false));
        assert!(!is_safe_url("http://172.16.0.1/hook", false));
    }

    #[test]
    fn safe_url_blocks_metadata() {
        assert!(!is_safe_url(
            "http://169.254.169.254/latest/meta-data/",
            false
        ));
        assert!(!is_safe_url(
            "http://metadata.google.internal/computeMetadata/",
            false
        ));
    }

    #[test]
    fn safe_url_blocks_non_http() {
        assert!(!is_safe_url("file:///etc/passwd", false));
        assert!(!is_safe_url("ftp://example.com/file", false));
        assert!(!is_safe_url("gopher://evil.com", false));
    }

    #[test]
    fn safe_url_blocks_internal_tlds() {
        assert!(!is_safe_url("http://service.internal/hook", false));
        assert!(!is_safe_url("http://host.local/hook", false));
        assert!(!is_safe_url("http://app.localhost/hook", false));
    }
}
