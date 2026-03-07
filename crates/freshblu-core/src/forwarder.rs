use serde::{Deserialize, Serialize};

/// The type of forwarder
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ForwarderType {
    Webhook,
    Meshblu,
}

/// Configuration for a webhook forwarder
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookForwarder {
    pub url: String,
    #[serde(default = "default_method")]
    pub method: String,
    #[serde(default)]
    pub sign_request: bool,
    #[serde(default)]
    pub generate_and_forward_meshblu_credentials: bool,
}

fn default_method() -> String {
    "POST".to_string()
}

/// Configuration for a meshblu-to-meshblu forwarder (re-emits as message)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MeshbluForwarder {}

/// A single forwarder entry — either webhook or meshblu
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ForwarderEntry {
    Webhook(WebhookForwarder),
    Meshblu(MeshbluForwarder),
}

/// Event category for forwarders
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ForwarderEvent {
    BroadcastSent,
    BroadcastReceived,
    ConfigureSent,
    ConfigureReceived,
    MessageSent,
    MessageReceived,
    UnregisterSent,
    UnregisterReceived,
}

/// Forwarder configuration on a device — one Vec per event type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Forwarders {
    #[serde(default)]
    pub broadcast: ForwarderPair,
    #[serde(default)]
    pub configure: ForwarderPair,
    #[serde(default)]
    pub message: ForwarderPair,
    #[serde(default)]
    pub unregister: ForwarderPair,
}

/// Sent/received pair for a forwarder category
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ForwarderPair {
    #[serde(default)]
    pub sent: Vec<ForwarderEntry>,
    #[serde(default)]
    pub received: Vec<ForwarderEntry>,
}

impl Forwarders {
    pub fn get(&self, event: ForwarderEvent) -> &[ForwarderEntry] {
        match event {
            ForwarderEvent::BroadcastSent => &self.broadcast.sent,
            ForwarderEvent::BroadcastReceived => &self.broadcast.received,
            ForwarderEvent::ConfigureSent => &self.configure.sent,
            ForwarderEvent::ConfigureReceived => &self.configure.received,
            ForwarderEvent::MessageSent => &self.message.sent,
            ForwarderEvent::MessageReceived => &self.message.received,
            ForwarderEvent::UnregisterSent => &self.unregister.sent,
            ForwarderEvent::UnregisterReceived => &self.unregister.received,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forwarder_entry_webhook_serde() {
        let entry = ForwarderEntry::Webhook(WebhookForwarder {
            url: "https://example.com/hook".to_string(),
            method: "POST".to_string(),
            sign_request: true,
            generate_and_forward_meshblu_credentials: false,
        });
        let json = serde_json::to_value(&entry).unwrap();
        assert_eq!(json["type"], "webhook");
        assert_eq!(json["url"], "https://example.com/hook");
        assert_eq!(json["signRequest"], true);

        let back: ForwarderEntry = serde_json::from_value(json).unwrap();
        assert_eq!(back, entry);
    }

    #[test]
    fn test_forwarder_entry_meshblu_serde() {
        let entry = ForwarderEntry::Meshblu(MeshbluForwarder {});
        let json = serde_json::to_value(&entry).unwrap();
        assert_eq!(json["type"], "meshblu");

        let back: ForwarderEntry = serde_json::from_value(json).unwrap();
        assert_eq!(back, entry);
    }

    #[test]
    fn test_forwarders_full_serde_roundtrip() {
        let fwd = Forwarders {
            message: ForwarderPair {
                sent: vec![ForwarderEntry::Webhook(WebhookForwarder {
                    url: "https://hooks.example.com".to_string(),
                    method: "POST".to_string(),
                    sign_request: false,
                    generate_and_forward_meshblu_credentials: true,
                })],
                received: vec![ForwarderEntry::Meshblu(MeshbluForwarder {})],
            },
            ..Default::default()
        };

        let json = serde_json::to_value(&fwd).unwrap();
        let back: Forwarders = serde_json::from_value(json).unwrap();
        assert_eq!(back, fwd);
    }

    #[test]
    fn test_forwarders_get_accessor() {
        let fwd = Forwarders {
            message: ForwarderPair {
                sent: vec![ForwarderEntry::Meshblu(MeshbluForwarder {})],
                received: vec![],
            },
            ..Default::default()
        };
        assert_eq!(fwd.get(ForwarderEvent::MessageSent).len(), 1);
        assert_eq!(fwd.get(ForwarderEvent::MessageReceived).len(), 0);
        assert_eq!(fwd.get(ForwarderEvent::BroadcastSent).len(), 0);
    }
}
