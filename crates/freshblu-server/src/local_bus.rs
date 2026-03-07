use async_trait::async_trait;
use freshblu_core::message::DeviceEvent;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::bus::MessageBus;
use crate::hub::MessageHub;

/// Single-process message bus backed by the in-memory `MessageHub`.
/// Used when no NATS_URL is configured (dev mode / single-pod deployment).
#[derive(Default)]
pub struct LocalBus {
    hub: MessageHub,
}

impl LocalBus {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl MessageBus for LocalBus {
    async fn publish(&self, target: &Uuid, event: DeviceEvent) -> anyhow::Result<()> {
        self.hub.deliver(target, event);
        Ok(())
    }

    async fn publish_many(&self, targets: &[Uuid], event: DeviceEvent) -> anyhow::Result<()> {
        self.hub.deliver_many(targets, event);
        Ok(())
    }

    fn connect(&self, uuid: Uuid) -> broadcast::Receiver<DeviceEvent> {
        self.hub.connect(uuid)
    }

    fn disconnect(&self, uuid: &Uuid) {
        self.hub.disconnect(uuid);
    }

    fn is_online(&self, uuid: &Uuid) -> bool {
        self.hub.is_online(uuid)
    }

    fn online_count(&self) -> usize {
        self.hub.online_count()
    }
}
