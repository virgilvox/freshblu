/// WebSocket handler for real-time device connections.
///
/// Protocol: After connecting, client sends an "identity" message with uuid + token.
/// Server responds with "ready" event. Then normal message exchange begins.
///
/// Compatible with Meshblu Socket.io event names so existing JS clients work.
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use futures::{SinkExt, StreamExt};
use freshblu_core::{
    device::RegisterParams,
    message::{DeviceEvent, SendMessageParams},
    subscription::{CreateSubscriptionParams, SubscriptionType},
};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::str::FromStr;
use uuid::Uuid;

use freshblu_core::permissions::PermissionChecker;

use crate::metrics::WS_CONNECTIONS;
use crate::AppState;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Messages the client can send over WebSocket
#[derive(Debug, Deserialize)]
#[serde(tag = "event", rename_all = "camelCase")]
enum ClientMessage {
    /// Authenticate: { event: "identity", uuid, token }
    Identity { uuid: String, token: String },
    /// Send a message
    Message(SendMessageParams),
    /// Update this device
    Update(HashMap<String, Value>),
    /// Subscribe to events
    Subscribe {
        #[serde(rename = "emitterUuid")]
        emitter_uuid: Uuid,
        #[serde(rename = "type")]
        subscription_type: String,
    },
    /// Unsubscribe
    Unsubscribe {
        #[serde(rename = "emitterUuid")]
        emitter_uuid: Uuid,
        #[serde(rename = "type")]
        subscription_type: Option<String>,
    },
    /// Whoami
    Whoami,
    /// Register a new device
    Register(RegisterParams),
    /// Unregister
    Unregister { uuid: Uuid },
    /// Ping
    Ping,
}

fn parse_client_message(text: &str) -> Option<ClientMessage> {
    if let Ok(m) = serde_json::from_str(text) {
        return Some(m);
    }
    // Try to parse as a raw message with event field
    let v: Value = serde_json::from_str(text).ok()?;
    let event = v.get("event")?.as_str()?;
    match event {
        "identity" => {
            let uuid = v.get("uuid")?.as_str()?.to_string();
            let token = v.get("token")?.as_str()?.to_string();
            Some(ClientMessage::Identity { uuid, token })
        }
        "ping" => Some(ClientMessage::Ping),
        "whoami" => Some(ClientMessage::Whoami),
        _ => None,
    }
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    // Phase 1: Wait for identity message
    let (device_uuid, hub_rx) = loop {
        let msg = match receiver.next().await {
            Some(Ok(Message::Text(t))) => t,
            Some(Ok(Message::Ping(p))) => {
                let _ = sender.send(Message::Pong(p)).await;
                continue;
            }
            Some(Ok(Message::Close(_))) | None => return,
            _ => continue,
        };

        let client_msg = match parse_client_message(&msg) {
            Some(m) => m,
            None => continue,
        };

        match client_msg {
            ClientMessage::Identity { uuid: uuid_str, token } => {
                let uuid = match Uuid::parse_str(&uuid_str) {
                    Ok(u) => u,
                    Err(_) => {
                        let _ = sender
                            .send(Message::Text(
                                serde_json::json!({
                                    "event": "notReady",
                                    "reason": "invalid uuid"
                                })
                                .to_string(),
                            ))
                            .await;
                        continue;
                    }
                };

                match state.store.authenticate(&uuid, &token).await {
                    Ok(Some(device)) => {
                        let _ = state.store.set_online(&uuid, true).await;
                        let ready = serde_json::json!({
                            "event": "ready",
                            "uuid": uuid,
                            "fromUuid": uuid,
                            "meshblu": device.meshblu,
                        });
                        let _ = sender
                            .send(Message::Text(ready.to_string()))
                            .await;
                        WS_CONNECTIONS.inc();
                        let rx = state.bus.connect(uuid);
                        break (uuid, rx);
                    }
                    _ => {
                        let _ = sender
                            .send(Message::Text(
                                serde_json::json!({
                                    "event": "notReady",
                                    "reason": "unauthorized"
                                })
                                .to_string(),
                            ))
                            .await;
                    }
                }
            }
            ClientMessage::Ping => {
                let _ = sender
                    .send(Message::Text(
                        serde_json::json!({ "event": "pong" }).to_string(),
                    ))
                    .await;
            }
            _ => {}
        }
    };

    // Phase 2: Bidirectional communication via tokio::select!
    let mut hub_rx = hub_rx;
    loop {
        tokio::select! {
            msg = receiver.next() => {
                let text = match msg {
                    Some(Ok(Message::Text(t))) => t,
                    Some(Ok(Message::Ping(p))) => {
                        let _ = sender.send(Message::Pong(p)).await;
                        continue;
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => continue,
                };

                let client_msg = match parse_client_message(&text) {
                    Some(m) => m,
                    None => continue,
                };

                match client_msg {
                    ClientMessage::Message(params) => {
                        handle_ws_message(&state, device_uuid, params).await;
                    }

                    ClientMessage::Whoami => {
                        if let Ok(Some(device)) = state.store.get_device(&device_uuid).await {
                            let json = serde_json::json!({
                                "event": "whoami",
                                "device": device.to_view()
                            });
                            let _ = sender.send(Message::Text(json.to_string())).await;
                        }
                    }

                    ClientMessage::Subscribe { emitter_uuid, subscription_type } => {
                        if let Ok(sub_type) = SubscriptionType::from_str(&subscription_type) {
                            // Check permission on the emitter device
                            let allowed = match state.store.get_device(&emitter_uuid).await {
                                Ok(Some(emitter_device)) => {
                                    let checker = PermissionChecker::new(
                                        &emitter_device.meshblu.whitelists,
                                        &device_uuid,
                                        &emitter_uuid,
                                    );
                                    match sub_type {
                                        SubscriptionType::BroadcastSent => checker.can_broadcast_sent(),
                                        SubscriptionType::BroadcastReceived => checker.can_broadcast_received(),
                                        SubscriptionType::MessageSent => checker.can_message_sent(),
                                        SubscriptionType::MessageReceived => checker.can_message_received(),
                                        SubscriptionType::ConfigureSent => checker.can_configure_sent(),
                                        SubscriptionType::ConfigureReceived => checker.can_configure_received(),
                                        SubscriptionType::UnregisterSent | SubscriptionType::UnregisterReceived => {
                                            checker.can_discover_view()
                                        }
                                    }
                                }
                                _ => false,
                            };

                            if allowed {
                                let params = CreateSubscriptionParams {
                                    emitter_uuid,
                                    subscriber_uuid: device_uuid,
                                    subscription_type: sub_type,
                                };
                                let _ = state.store.create_subscription(&params).await;
                            } else {
                                let err = serde_json::json!({
                                    "event": "error",
                                    "message": "forbidden: insufficient permission to subscribe"
                                });
                                let _ = sender.send(Message::Text(err.to_string())).await;
                            }
                        }
                    }

                    ClientMessage::Unsubscribe { emitter_uuid, subscription_type } => {
                        let sub_type = subscription_type
                            .and_then(|s| SubscriptionType::from_str(&s).ok());
                        let _ = state.store.delete_subscription(
                            &device_uuid,
                            Some(&emitter_uuid),
                            sub_type.as_ref(),
                        ).await;
                    }

                    ClientMessage::Update(properties) => {
                        if let Ok(updated) = state.store.update_device(&device_uuid, properties).await {
                            let view = updated.to_view();
                            let config_event = DeviceEvent::Config { device: view };
                            // Fan out to configure.sent subscribers
                            let subscribers = state.store
                                .get_subscribers(&device_uuid, &SubscriptionType::ConfigureSent)
                                .await.unwrap_or_default();
                            for sub_uuid in subscribers {
                                let _ = state.bus.publish(&sub_uuid, config_event.clone()).await;
                            }
                            let _ = state.bus.publish(&device_uuid, config_event).await;
                        }
                    }

                    ClientMessage::Register(params) => {
                        match state.store.register(params).await {
                            Ok((device, token)) => {
                                let json = serde_json::json!({
                                    "event": "registered",
                                    "uuid": device.uuid,
                                    "token": token,
                                });
                                let _ = sender.send(Message::Text(json.to_string())).await;
                            }
                            Err(e) => {
                                let json = serde_json::json!({
                                    "event": "error",
                                    "message": e.to_string(),
                                });
                                let _ = sender.send(Message::Text(json.to_string())).await;
                            }
                        }
                    }

                    ClientMessage::Unregister { uuid } => {
                        // Only allow unregistering if it's the connected device or has permission
                        if uuid == device_uuid {
                            let _ = state.store.unregister(&uuid).await;
                            state.bus.disconnect(&uuid);
                            break;
                        }
                    }

                    ClientMessage::Ping => {
                        let _ = sender
                            .send(Message::Text(
                                serde_json::json!({ "event": "pong" }).to_string(),
                            ))
                            .await;
                    }

                    ClientMessage::Identity { .. } => {
                        // Already authenticated, ignore
                    }
                }
            }

            event = hub_rx.recv() => {
                match event {
                    Ok(device_event) => {
                        if let Ok(json) = serde_json::to_string(&device_event) {
                            if sender.send(Message::Text(json)).await.is_err() {
                                break;
                            }
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                        // Slow consumer, skip missed messages
                        continue;
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        break;
                    }
                }
            }
        }
    }

    // Cleanup on disconnect
    WS_CONNECTIONS.dec();
    let _ = state.store.set_online(&device_uuid, false).await;
    state.bus.disconnect(&device_uuid);
}

async fn handle_ws_message(state: &AppState, actor_uuid: Uuid, params: SendMessageParams) {
    let msg = freshblu_core::message::Message {
        devices: params.devices.clone(),
        from_uuid: Some(actor_uuid),
        topic: params.topic.clone(),
        payload: params.payload.clone(),
        metadata: None,
        extra: params.extra.clone(),
    };

    let is_broadcast = params.is_broadcast();

    for device_id in &params.devices {
        if device_id == "*" {
            continue;
        }
        if let Ok(target_uuid) = Uuid::parse_str(device_id) {
            // Check can_message_from permission on target device
            let allowed = match state.store.get_device(&target_uuid).await {
                Ok(Some(target_device)) => {
                    let checker = PermissionChecker::new(
                        &target_device.meshblu.whitelists,
                        &actor_uuid,
                        &target_uuid,
                    );
                    checker.can_message_from()
                }
                _ => false,
            };
            if allowed {
                let _ = state.bus.publish(&target_uuid, DeviceEvent::Message(msg.clone())).await;
            }
        }
    }

    if is_broadcast {
        let subs = state
            .store
            .get_subscribers(&actor_uuid, &SubscriptionType::BroadcastSent)
            .await
            .unwrap_or_default();
        for sub_uuid in subs {
            let _ = state.bus.publish(&sub_uuid, DeviceEvent::Broadcast(msg.clone())).await;
        }
    }
}
