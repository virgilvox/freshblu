/// MQTT Protocol Adapter
///
/// Bridges MQTT clients to the FreshBlu messaging system using rumqttd embedded broker.
/// MQTT topic format: <uuid>/<event_type>
/// Authentication: username=uuid, password=token
///
/// Meshblu-compatible: devices can connect over MQTT and receive/send messages.
use freshblu_core::message::{DeviceEvent, SendMessageParams};
use freshblu_core::permissions::PermissionChecker;
use freshblu_store::DynStore;
use rumqttd::{Broker, Config, ConnectionSettings, RouterConfig, ServerSettings};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::bus::DynBus;

pub struct MqttAdapter {
    store: DynStore,
    bus: DynBus,
    port: u16,
}

impl MqttAdapter {
    pub fn new(store: DynStore, bus: DynBus, port: u16) -> Self {
        Self { store, bus, port }
    }

    /// Start the MQTT broker - runs in background task
    pub async fn start(self) -> anyhow::Result<()> {
        info!("MQTT adapter starting on port {}", self.port);

        let store = self.store.clone();
        let auth_handler: rumqttd::AuthHandler = Arc::new(move |_client_id, username, password| {
            let uuid = match Uuid::parse_str(&username) {
                Ok(u) => u,
                Err(_) => return false,
            };
            // Wrap in spawn_blocking so bcrypt doesn't stall the event loop
            let store = store.clone();
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async move {
                matches!(
                    tokio::task::spawn_blocking(move || {
                        let rt2 = tokio::runtime::Handle::current();
                        rt2.block_on(store.authenticate(&uuid, &password))
                    })
                    .await,
                    Ok(Ok(Some(_)))
                )
            })
        });

        let addr: SocketAddr = format!("0.0.0.0:{}", self.port).parse()?;
        let mut server_settings = HashMap::new();
        server_settings.insert(
            "freshblu".to_string(),
            ServerSettings {
                name: "freshblu-mqtt".to_string(),
                listen: addr,
                tls: None,
                next_connection_delay_ms: 0,
                connections: ConnectionSettings {
                    connection_timeout_ms: 5000,
                    max_payload_size: 1_048_576, // 1MB
                    max_inflight_count: 100,
                    auth: None,
                    external_auth: Some(auth_handler),
                    dynamic_filters: true,
                },
            },
        );

        let config = Config {
            id: 0,
            router: RouterConfig {
                max_connections: 10000,
                max_outgoing_packet_count: 200,
                max_segment_size: 100 * 1024,
                max_segment_count: 10,
                ..Default::default()
            },
            v4: Some(server_settings),
            ..Default::default()
        };

        let mut broker = Broker::new(config);

        // Get a programmatic link for bridging MQTT <-> MessageBus
        let (mut link_tx, mut link_rx) = broker
            .link("freshblu-bridge")
            .map_err(|e| anyhow::anyhow!("failed to create broker link: {:?}", e))?;

        // Subscribe to all topics via wildcard
        let _ = link_tx.subscribe("#");

        // Start the broker in a background thread (it blocks)
        std::thread::spawn(move || {
            if let Err(e) = broker.start() {
                error!("MQTT broker error: {:?}", e);
            }
        });

        let store = self.store.clone();
        let bus = self.bus.clone();

        // Bridge task: forward MQTT publishes to MessageBus
        tokio::spawn(async move {
            loop {
                match link_rx.next().await {
                    Ok(Some(notification)) => {
                        if let rumqttd::Notification::Forward(forward) = notification {
                            let topic = forward.publish.topic.clone();
                            let payload = forward.publish.payload.clone();
                            let topic_str = String::from_utf8_lossy(&topic);

                            if let Some((target_uuid, event_type)) = parse_mqtt_topic(&topic_str) {
                                match event_type {
                                    "message" => {
                                        if let Ok(params) =
                                            serde_json::from_slice::<SendMessageParams>(&payload)
                                        {
                                            let msg = freshblu_core::message::Message {
                                                devices: params.devices.clone(),
                                                from_uuid: Some(target_uuid),
                                                topic: params.topic.clone(),
                                                payload: params.payload.clone(),
                                                metadata: None,
                                                extra: params.extra.clone(),
                                            };
                                            for device_id in &params.devices {
                                                if device_id == "*" {
                                                    continue;
                                                }
                                                if let Ok(dest_uuid) = Uuid::parse_str(device_id) {
                                                    // Check can_message_from permission
                                                    let allowed = match store
                                                        .get_device(&dest_uuid)
                                                        .await
                                                    {
                                                        Ok(Some(dest_device)) => {
                                                            let checker = PermissionChecker::new(
                                                                &dest_device.meshblu.whitelists,
                                                                &target_uuid,
                                                                &dest_uuid,
                                                            );
                                                            checker.can_message_from()
                                                        }
                                                        _ => false,
                                                    };
                                                    if allowed {
                                                        let _ = bus
                                                            .publish(
                                                                &dest_uuid,
                                                                DeviceEvent::Message(msg.clone()),
                                                            )
                                                            .await;
                                                    } else {
                                                        warn!("MQTT: dropping unauthorized message from {} to {}", target_uuid, dest_uuid);
                                                    }
                                                }
                                            }
                                            if params.is_broadcast() {
                                                if let Ok(subs) = store.get_subscribers(
                                                    &target_uuid,
                                                    &freshblu_core::subscription::SubscriptionType::BroadcastSent,
                                                ).await {
                                                    for sub_uuid in subs {
                                                        let _ = bus.publish(&sub_uuid, DeviceEvent::Broadcast(msg.clone())).await;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    "broadcast" => {
                                        if let Ok(value) =
                                            serde_json::from_slice::<serde_json::Value>(&payload)
                                        {
                                            let msg = freshblu_core::message::Message {
                                                devices: vec!["*".to_string()],
                                                from_uuid: Some(target_uuid),
                                                topic: None,
                                                payload: Some(value),
                                                metadata: None,
                                                extra: HashMap::new(),
                                            };
                                            if let Ok(subs) = store.get_subscribers(
                                                &target_uuid,
                                                &freshblu_core::subscription::SubscriptionType::BroadcastSent,
                                            ).await {
                                                let event = DeviceEvent::Broadcast(msg);
                                                for sub_uuid in subs {
                                                    let _ = bus.publish(&sub_uuid, event.clone()).await;
                                                }
                                            }
                                        }
                                    }
                                    _ => {
                                        warn!("Unknown MQTT event type: {}", event_type);
                                    }
                                }
                            }
                        }
                    }
                    Ok(None) => continue,
                    Err(e) => {
                        error!("MQTT bridge link error: {:?}", e);
                        break;
                    }
                }
            }
        });

        info!("MQTT broker running at mqtt://0.0.0.0:{}", self.port);
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
