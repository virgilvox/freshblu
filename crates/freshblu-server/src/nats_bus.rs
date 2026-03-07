use async_trait::async_trait;
use freshblu_core::message::DeviceEvent;
use freshblu_proto::{DeliveryEnvelope, NatsEvent};
use tokio::sync::broadcast;
use tracing::{error, info};
use uuid::Uuid;

use crate::bus::MessageBus;
use crate::hub::MessageHub;

/// Multi-pod message bus backed by NATS.
/// Local delivery uses the embedded `MessageHub`. Cross-pod delivery publishes
/// events to NATS subjects; the router resolves subscriptions and fans out
/// `DeliveryEnvelope`s to the target pod's delivery subject.
pub struct NatsBus {
    client: async_nats::Client,
    local: MessageHub,
    pod_id: String,
}

impl NatsBus {
    pub async fn new(nats_url: &str, pod_id: String) -> anyhow::Result<Self> {
        let client = async_nats::connect(nats_url).await?;
        let bus = Self {
            client,
            local: MessageHub::default(),
            pod_id,
        };
        bus.start_delivery_listener().await?;
        Ok(bus)
    }

    /// Subscribe to `freshblu.delivery.{pod_id}` and route incoming envelopes
    /// to locally connected devices.
    async fn start_delivery_listener(&self) -> anyhow::Result<()> {
        let subject = freshblu_proto::delivery(&self.pod_id);
        let mut sub = self.client.subscribe(subject.clone()).await?;
        let local = self.local.clone_inner();
        info!("NatsBus: listening on {}", subject);

        tokio::spawn(async move {
            while let Some(msg) = sub.next().await {
                match serde_json::from_slice::<DeliveryEnvelope>(&msg.payload) {
                    Ok(envelope) => {
                        let event = match envelope.event {
                            NatsEvent::Message { msg, .. } => DeviceEvent::Message(msg),
                            NatsEvent::Broadcast { msg, .. } => DeviceEvent::Broadcast(msg),
                            NatsEvent::ConfigUpdate { device, .. } => {
                                DeviceEvent::Config { device }
                            }
                            NatsEvent::Unregister { uuid } => DeviceEvent::Unregistered { uuid },
                        };
                        local.deliver(&envelope.target, event);
                    }
                    Err(e) => {
                        error!("NatsBus: failed to deserialize delivery envelope: {}", e);
                    }
                }
            }
        });
        Ok(())
    }
}

use futures::StreamExt;

#[async_trait]
impl MessageBus for NatsBus {
    async fn publish(&self, target: &Uuid, event: DeviceEvent) -> anyhow::Result<()> {
        // Try local delivery first
        if self.local.is_online(target) {
            self.local.deliver(target, event);
            return Ok(());
        }

        // Publish to NATS for cross-pod routing
        let nats_event = match device_event_to_nats(target, &event) {
            Some(e) => e,
            None => return Ok(()), // Local-only event, skip NATS
        };
        let subject = freshblu_proto::device_inbox(target);
        let payload = serde_json::to_vec(&nats_event)?;
        self.client.publish(subject, payload.into()).await?;
        Ok(())
    }

    async fn publish_many(&self, targets: &[Uuid], event: DeviceEvent) -> anyhow::Result<()> {
        for target in targets {
            self.publish(target, event.clone()).await?;
        }
        Ok(())
    }

    fn connect(&self, uuid: Uuid) -> broadcast::Receiver<DeviceEvent> {
        self.local.connect(uuid)
    }

    fn disconnect(&self, uuid: &Uuid) {
        self.local.disconnect(uuid);
    }

    fn is_online(&self, uuid: &Uuid) -> bool {
        self.local.is_online(uuid)
    }

    fn online_count(&self) -> usize {
        self.local.online_count()
    }
}

fn device_event_to_nats(target: &Uuid, event: &DeviceEvent) -> Option<NatsEvent> {
    match event {
        DeviceEvent::Message(msg) => Some(NatsEvent::Message {
            from: msg.from_uuid.unwrap_or(*target),
            msg: msg.clone(),
        }),
        DeviceEvent::Broadcast(msg) => Some(NatsEvent::Broadcast {
            from: msg.from_uuid.unwrap_or(*target),
            msg: msg.clone(),
        }),
        DeviceEvent::Config { device } => Some(NatsEvent::ConfigUpdate {
            uuid: *target,
            device: device.clone(),
        }),
        DeviceEvent::Unregistered { uuid } => Some(NatsEvent::Unregister { uuid: *uuid }),
        // Ready/NotReady are local-only events, should not be published to NATS
        DeviceEvent::Ready { .. } | DeviceEvent::NotReady { .. } => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use freshblu_core::message::Message;
    use freshblu_core::permissions::Whitelists;
    use std::collections::HashMap;

    fn make_msg() -> Message {
        Message {
            devices: vec!["test".into()],
            from_uuid: Some(Uuid::new_v4()),
            topic: None,
            payload: None,
            metadata: None,
            extra: HashMap::new(),
        }
    }

    #[test]
    fn device_event_to_nats_all_variants() {
        let target = Uuid::new_v4();
        let msg = make_msg();

        // Message -> Some(Message)
        let result = device_event_to_nats(&target, &DeviceEvent::Message(msg.clone()));
        assert!(matches!(result, Some(NatsEvent::Message { .. })));

        // Broadcast -> Some(Broadcast)
        let result = device_event_to_nats(&target, &DeviceEvent::Broadcast(msg.clone()));
        assert!(matches!(result, Some(NatsEvent::Broadcast { .. })));

        // Config -> Some(ConfigUpdate)
        let view = freshblu_core::device::DeviceView {
            uuid: target,
            online: true,
            device_type: None,
            meshblu: freshblu_core::device::MeshbluMeta::new(Whitelists::default()),
            properties: HashMap::new(),
        };
        let result = device_event_to_nats(
            &target,
            &DeviceEvent::Config {
                device: Box::new(view),
            },
        );
        assert!(matches!(result, Some(NatsEvent::ConfigUpdate { .. })));

        // Unregistered -> Some(Unregister)
        let result = device_event_to_nats(&target, &DeviceEvent::Unregistered { uuid: target });
        assert!(matches!(result, Some(NatsEvent::Unregister { .. })));
    }

    #[test]
    fn device_event_to_nats_ready_skipped() {
        let target = Uuid::new_v4();

        // Ready should return None, NOT Unregister
        let result = device_event_to_nats(
            &target,
            &DeviceEvent::Ready {
                uuid: target,
                token: None,
            },
        );
        assert!(result.is_none(), "Ready should not produce a NatsEvent");

        // NotReady should return None
        let result = device_event_to_nats(
            &target,
            &DeviceEvent::NotReady {
                reason: "test".into(),
            },
        );
        assert!(result.is_none(), "NotReady should not produce a NatsEvent");
    }
}
