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
    message::{DeviceEvent, SendMessageParams},
    subscription::{CreateSubscriptionParams, SubscriptionType},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::str::FromStr;
use uuid::Uuid;

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
    Register(freshblu_core::device::RegisterParams),
    /// Unregister
    Unregister { uuid: Uuid },
    /// Ping
    Ping,
}

/// Messages the server sends to the client
#[derive(Serialize)]
struct ServerMessage {
    event: String,
    #[serde(flatten)]
    data: Value,
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    let mut authenticated_uuid: Option<Uuid> = None;

    while let Some(msg) = receiver.next().await {
        let text = match msg {
            Ok(Message::Text(t)) => t,
            Ok(Message::Close(_)) => break,
            Ok(Message::Ping(p)) => {
                let _ = sender.send(Message::Pong(p)).await;
                continue;
            }
            _ => continue,
        };

        // Parse the client message
        let client_msg: ClientMessage = match serde_json::from_str(&text) {
            Ok(m) => m,
            Err(_) => {
                // Try to parse as a raw message with event field
                if let Ok(v) = serde_json::from_str::<Value>(&text) {
                    if v.get("event").and_then(|e| e.as_str()) == Some("identity") {
                        // Manual parse for identity
                        let uuid = v.get("uuid").and_then(|u| u.as_str()).unwrap_or("").to_string();
                        let token = v.get("token").and_then(|t| t.as_str()).unwrap_or("").to_string();
                        ClientMessage::Identity { uuid, token }
                    } else {
                        continue;
                    }
                } else {
                    continue;
                }
            }
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
                        authenticated_uuid = Some(uuid);
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

                        // Subscribe to own events in background
                        let hub = state.hub.clone();
                        let store = state.store.clone();
                        let mut rx = hub.connect(uuid);
                        let (tx_ws, mut rx_ws) = tokio::sync::mpsc::channel::<DeviceEvent>(64);

                        // Spawn forwarder: hub events -> WS sender
                        tokio::spawn(async move {
                            while let Ok(event) = rx.recv().await {
                                let json = match serde_json::to_string(&event) {
                                    Ok(j) => j,
                                    Err(_) => continue,
                                };
                                let _ = tx_ws.send(event).await;
                            }
                        });
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

            ClientMessage::Message(params) if authenticated_uuid.is_some() => {
                let actor_uuid = authenticated_uuid.unwrap();
                // Route message through the message system
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
                    if device_id == "*" { continue; }
                    if let Ok(target_uuid) = Uuid::parse_str(device_id) {
                        state.hub.deliver(&target_uuid, DeviceEvent::Message(msg.clone()));
                    }
                }

                if is_broadcast {
                    let subs = state.store
                        .get_subscribers(&actor_uuid, &SubscriptionType::BroadcastSent)
                        .await
                        .unwrap_or_default();
                    for sub_uuid in subs {
                        state.hub.deliver(&sub_uuid, DeviceEvent::Broadcast(msg.clone()));
                    }
                }
            }

            ClientMessage::Whoami if authenticated_uuid.is_some() => {
                let uuid = authenticated_uuid.unwrap();
                if let Ok(Some(device)) = state.store.get_device(&uuid).await {
                    let json = serde_json::json!({
                        "event": "whoami",
                        "device": device.to_view()
                    });
                    let _ = sender.send(Message::Text(json.to_string())).await;
                }
            }

            ClientMessage::Subscribe { emitter_uuid, subscription_type } if authenticated_uuid.is_some() => {
                let subscriber = authenticated_uuid.unwrap();
                if let Ok(sub_type) = SubscriptionType::from_str(&subscription_type) {
                    let params = CreateSubscriptionParams {
                        emitter_uuid,
                        subscriber_uuid: subscriber,
                        subscription_type: sub_type,
                    };
                    let _ = state.store.create_subscription(&params).await;
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
    }

    // Cleanup on disconnect
    if let Some(uuid) = authenticated_uuid {
        let _ = state.store.set_online(&uuid, false).await;
        state.hub.disconnect(&uuid);
    }
}
