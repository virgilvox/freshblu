use std::sync::Arc;

use async_trait::async_trait;
use freshblu_core::message::DeviceEvent;
use tokio::sync::broadcast;
use uuid::Uuid;

/// Abstraction over the message delivery system.
/// `LocalBus` for single-process dev mode, `NatsBus` for multi-pod production.
#[async_trait]
pub trait MessageBus: Send + Sync + 'static {
    /// Publish an event targeting a specific device.
    async fn publish(&self, target: &Uuid, event: DeviceEvent) -> anyhow::Result<()>;

    /// Publish to multiple targets.
    async fn publish_many(&self, targets: &[Uuid], event: DeviceEvent) -> anyhow::Result<()>;

    /// Register device presence on this pod, returns a receiver for events.
    fn connect(&self, uuid: Uuid) -> broadcast::Receiver<DeviceEvent>;

    /// Remove device presence.
    fn disconnect(&self, uuid: &Uuid);

    /// Check if device is online on this pod.
    fn is_online(&self, uuid: &Uuid) -> bool;

    /// Count of online devices on this pod.
    fn online_count(&self) -> usize;
}

pub type DynBus = Arc<dyn MessageBus>;
