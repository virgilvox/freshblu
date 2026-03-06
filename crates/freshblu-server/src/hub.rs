/// The MessageHub routes events to connected WebSocket/MQTT clients.
/// It holds a map of connected device connections and delivers events.

use std::collections::HashMap;
use std::sync::Arc;

use dashmap::DashMap;
use freshblu_core::message::DeviceEvent;
use tokio::sync::broadcast;
use uuid::Uuid;

const CHANNEL_CAPACITY: usize = 256;

/// A sender handle for a connected device
pub type EventSender = broadcast::Sender<DeviceEvent>;

/// The central message hub
pub struct MessageHub {
    /// Map of connected device UUID -> broadcast sender
    connections: DashMap<Uuid, EventSender>,
}

impl MessageHub {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            connections: DashMap::new(),
        })
    }

    /// Register a new connection, returns a receiver
    pub fn connect(&self, uuid: Uuid) -> broadcast::Receiver<DeviceEvent> {
        // If already connected, just subscribe to existing channel
        if let Some(sender) = self.connections.get(&uuid) {
            return sender.subscribe();
        }
        let (tx, rx) = broadcast::channel(CHANNEL_CAPACITY);
        self.connections.insert(uuid, tx);
        rx
    }

    /// Disconnect a device
    pub fn disconnect(&self, uuid: &Uuid) {
        self.connections.remove(uuid);
    }

    /// Deliver an event to a specific device
    pub fn deliver(&self, uuid: &Uuid, event: DeviceEvent) {
        if let Some(sender) = self.connections.get(uuid) {
            let _ = sender.send(event);
        }
    }

    /// Deliver an event to multiple devices
    pub fn deliver_many(&self, uuids: &[Uuid], event: DeviceEvent) {
        for uuid in uuids {
            self.deliver(uuid, event.clone());
        }
    }

    /// Check if a device is currently online (has an active connection)
    pub fn is_online(&self, uuid: &Uuid) -> bool {
        self.connections.contains_key(uuid)
    }

    /// Get count of online devices
    pub fn online_count(&self) -> usize {
        self.connections.len()
    }

    /// Get all online device UUIDs
    pub fn online_devices(&self) -> Vec<Uuid> {
        self.connections.iter().map(|e| *e.key()).collect()
    }
}

impl Default for MessageHub {
    fn default() -> Self {
        Self {
            connections: DashMap::new(),
        }
    }
}
