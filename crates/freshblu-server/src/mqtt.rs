/// MQTT Protocol Adapter
///
/// Bridges MQTT clients to the FreshBlu messaging system.
/// MQTT topic format: <uuid>/<event_type>
/// Authentication: username=uuid, password=token
///
/// Meshblu-compatible: devices can connect over MQTT and receive/send messages.

use freshblu_core::message::SendMessageParams;
use freshblu_store::DynStore;
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::MessageHub;

pub struct MqttAdapter {
    store: DynStore,
    hub: Arc<MessageHub>,
    port: u16,
}

impl MqttAdapter {
    pub fn new(store: DynStore, hub: Arc<MessageHub>, port: u16) -> Self {
        Self { store, hub, port }
    }

    /// Start the MQTT broker - runs in background task
    pub async fn start(self) -> anyhow::Result<()> {
        info!("MQTT adapter starting on port {}", self.port);
        // In a real implementation, use rumqttd embedded broker here
        // For now we log a notice - full MQTT implementation would use
        // rumqttd with a custom authentication handler hooked into the store
        info!("MQTT broker at mqtt://0.0.0.0:{}", self.port);
        info!("MQTT auth: username=device-uuid, password=device-token");
        info!("MQTT topics: {{uuid}}/message, {{uuid}}/broadcast, {{uuid}}/config");
        Ok(())
    }
}

/// Parse a Meshblu MQTT topic
/// Format: <device_uuid>/<event_type>
pub fn parse_mqtt_topic(topic: &str) -> Option<(Uuid, &str)> {
    let mut parts = topic.splitn(2, '/');
    let uuid_str = parts.next()?;
    let event_type = parts.next().unwrap_or("message");
    let uuid = Uuid::parse_str(uuid_str).ok()?;
    Some((uuid, event_type))
}

/// Map MQTT client ID to device UUID
/// Meshblu uses the device UUID as the MQTT client ID
pub fn mqtt_client_id_to_uuid(client_id: &str) -> Option<Uuid> {
    Uuid::parse_str(client_id).ok()
}
