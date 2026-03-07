use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::subscription::RouteHop;

/// A message sent between devices
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    /// Target devices. Use ["*"] for broadcast to all subscribers.
    pub devices: Vec<String>,

    /// The UUID of the sending device (set by server, trusted)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_uuid: Option<Uuid>,

    /// Optional topic for filtering
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,

    /// Optional payload
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<Value>,

    /// Route metadata - the path this message took through subscriptions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<MessageMetadata>,

    /// Any extra fields
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MessageMetadata {
    pub route: Vec<RouteHop>,
}

/// Params sent by client to send a message
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageParams {
    pub devices: Vec<String>,
    pub topic: Option<String>,
    pub payload: Option<Value>,
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, Value>,
}

impl SendMessageParams {
    pub fn is_broadcast(&self) -> bool {
        self.devices.iter().any(|d| d == "*")
    }
}

/// Events delivered over WebSocket / MQTT
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "camelCase")]
pub enum DeviceEvent {
    /// A message was received
    Message(Message),
    /// The device's config was updated
    Config { device: crate::device::DeviceView },
    /// A subscription-forwarded event arrived
    Broadcast(Message),
    /// The server says hello, auth complete
    Ready { uuid: Uuid, token: Option<String> },
    /// Auth failed
    NotReady { reason: String },
    /// Another device unregistered
    Unregistered { uuid: Uuid },
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_send_params(devices: Vec<&str>) -> SendMessageParams {
        SendMessageParams {
            devices: devices.into_iter().map(String::from).collect(),
            topic: None,
            payload: None,
            extra: HashMap::new(),
        }
    }

    #[test]
    fn test_is_broadcast() {
        let broadcast = make_send_params(vec!["*"]);
        assert!(broadcast.is_broadcast());

        let mixed = make_send_params(vec!["some-uuid", "*"]);
        assert!(mixed.is_broadcast());

        let specific = make_send_params(vec![
            "550e8400-e29b-41d4-a716-446655440000",
        ]);
        assert!(!specific.is_broadcast());

        let empty = make_send_params(vec![]);
        assert!(!empty.is_broadcast());
    }

    #[test]
    fn test_device_event_serialization() {
        let uuid = Uuid::new_v4();

        // Ready variant
        let ready = DeviceEvent::Ready {
            uuid,
            token: Some("tok123".to_string()),
        };
        let json = serde_json::to_value(&ready).unwrap();
        assert_eq!(json["event"], "ready");
        let deserialized: DeviceEvent = serde_json::from_value(json).unwrap();
        match deserialized {
            DeviceEvent::Ready { uuid: u, token: t } => {
                assert_eq!(u, uuid);
                assert_eq!(t, Some("tok123".to_string()));
            }
            _ => panic!("expected Ready variant"),
        }

        // NotReady variant
        let not_ready = DeviceEvent::NotReady {
            reason: "bad token".to_string(),
        };
        let json = serde_json::to_value(&not_ready).unwrap();
        assert_eq!(json["event"], "notReady");
        let deserialized: DeviceEvent = serde_json::from_value(json).unwrap();
        match deserialized {
            DeviceEvent::NotReady { reason } => assert_eq!(reason, "bad token"),
            _ => panic!("expected NotReady variant"),
        }

        // Unregistered variant
        let unreg = DeviceEvent::Unregistered { uuid };
        let json = serde_json::to_value(&unreg).unwrap();
        assert_eq!(json["event"], "unregistered");
        let roundtripped: DeviceEvent = serde_json::from_value(json).unwrap();
        match roundtripped {
            DeviceEvent::Unregistered { uuid: u } => assert_eq!(u, uuid),
            _ => panic!("expected Unregistered variant"),
        }
    }
}
