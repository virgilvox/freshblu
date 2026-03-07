use freshblu_core::device::DeviceView;
use freshblu_core::message::Message;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Envelope for delivering events between gateway pods via NATS.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryEnvelope {
    /// The device that should receive this event.
    pub target: Uuid,
    /// The event to deliver.
    pub event: NatsEvent,
    /// The pod that originated this event.
    pub source_pod: String,
}

/// Events transported over NATS between gateway and router.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NatsEvent {
    /// A direct message from one device to another.
    Message { from: Uuid, msg: Message },
    /// A broadcast message.
    Broadcast { from: Uuid, msg: Message },
    /// A device's configuration was updated.
    ConfigUpdate { uuid: Uuid, device: DeviceView },
    /// A device was unregistered.
    Unregister { uuid: Uuid },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn envelope_roundtrip() {
        let uuid = Uuid::new_v4();
        let envelope = DeliveryEnvelope {
            target: uuid,
            event: NatsEvent::Unregister { uuid },
            source_pod: "pod-1".to_string(),
        };
        let json = serde_json::to_string(&envelope).unwrap();
        let decoded: DeliveryEnvelope = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.target, uuid);
        assert_eq!(decoded.source_pod, "pod-1");
    }
}
