/// The MessageHub routes events to connected WebSocket/MQTT clients.
/// It holds a map of connected device connections and delivers events.
use std::sync::Arc;

use dashmap::DashMap;
use freshblu_core::message::DeviceEvent;
use tokio::sync::broadcast;
use uuid::Uuid;

const CHANNEL_CAPACITY: usize = 256;

/// A sender handle for a connected device
pub type EventSender = broadcast::Sender<DeviceEvent>;

/// The central message hub
#[derive(Clone)]
pub struct MessageHub {
    /// Map of connected device UUID -> broadcast sender
    connections: Arc<DashMap<Uuid, EventSender>>,
}

impl MessageHub {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            connections: Arc::new(DashMap::new()),
        })
    }

    /// Get a clone of this hub that shares the same connection map.
    /// Used by NatsBus to give its delivery listener access to connected devices.
    pub fn clone_inner(&self) -> Self {
        self.clone()
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
            connections: Arc::new(DashMap::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use freshblu_core::message::DeviceEvent;
    use uuid::Uuid;

    fn test_event() -> DeviceEvent {
        DeviceEvent::Unregistered {
            uuid: Uuid::new_v4(),
        }
    }

    #[test]
    fn connect_and_deliver() {
        let hub = MessageHub::new();
        let uuid = Uuid::new_v4();
        let mut rx = hub.connect(uuid);

        let event = test_event();
        hub.deliver(&uuid, event.clone());

        let received = rx.try_recv().expect("should receive event");
        assert_eq!(received, event);
    }

    #[test]
    fn disconnect_removes() {
        let hub = MessageHub::new();
        let uuid = Uuid::new_v4();
        let _rx = hub.connect(uuid);

        assert!(hub.is_online(&uuid));
        hub.disconnect(&uuid);
        assert!(!hub.is_online(&uuid));
    }

    #[test]
    fn deliver_to_offline_no_panic() {
        let hub = MessageHub::new();
        let uuid = Uuid::new_v4();
        // Delivering to a non-existent UUID should not panic
        hub.deliver(&uuid, test_event());
    }

    #[test]
    fn deliver_many() {
        let hub = MessageHub::new();
        let uuids: Vec<Uuid> = (0..3).map(|_| Uuid::new_v4()).collect();
        let mut receivers: Vec<_> = uuids.iter().map(|u| hub.connect(*u)).collect();

        let event = test_event();
        hub.deliver_many(&uuids, event.clone());

        for rx in receivers.iter_mut() {
            let received = rx.try_recv().expect("should receive event");
            assert_eq!(received, event);
        }
    }

    #[test]
    fn online_count() {
        let hub = MessageHub::new();
        let u1 = Uuid::new_v4();
        let u2 = Uuid::new_v4();

        let _rx1 = hub.connect(u1);
        let _rx2 = hub.connect(u2);
        assert_eq!(hub.online_count(), 2);

        hub.disconnect(&u1);
        assert_eq!(hub.online_count(), 1);
    }

    #[test]
    fn multiple_subscribers_same_device() {
        let hub = MessageHub::new();
        let uuid = Uuid::new_v4();

        let mut rx1 = hub.connect(uuid);
        let mut rx2 = hub.connect(uuid); // second subscriber to the same channel

        let event = test_event();
        hub.deliver(&uuid, event.clone());

        let r1 = rx1.try_recv().expect("rx1 should receive event");
        let r2 = rx2.try_recv().expect("rx2 should receive event");
        assert_eq!(r1, event);
        assert_eq!(r2, event);
    }
}
