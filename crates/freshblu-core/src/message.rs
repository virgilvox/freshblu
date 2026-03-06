use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::subscription::RouteHop;

/// A message sent between devices
#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
