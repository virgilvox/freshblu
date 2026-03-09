use crate::{Error, FreshBluClient};
use futures::{SinkExt, StreamExt};
use serde_json::Value;
use tokio::sync::{broadcast, mpsc};
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};
use uuid::Uuid;

/// WebSocket event received from the server.
#[derive(Debug, Clone)]
pub struct WsEvent {
    pub event: String,
    pub data: Value,
}

/// WebSocket client for real-time FreshBlu messaging.
pub struct FreshBluWs {
    tx: mpsc::Sender<String>,
    events: broadcast::Sender<WsEvent>,
    _handle: tokio::task::JoinHandle<()>,
}

impl FreshBluWs {
    /// Connect to a FreshBlu WebSocket server and authenticate.
    ///
    /// Returns a connected `FreshBluWs` after the `ready` event is received.
    pub async fn connect(client: &FreshBluClient) -> Result<Self, Error> {
        let (uuid, token) = client.credentials()
            .ok_or_else(|| Error::Other("Credentials required for WebSocket".into()))?;

        let ws_url = client.base_url
            .replace("https://", "wss://")
            .replace("http://", "ws://");
        let ws_url = format!("{}/ws", ws_url);

        let (ws_stream, _) = connect_async(&ws_url).await?;
        let (mut sink, mut stream) = ws_stream.split();

        // Send identity
        let identity = serde_json::json!({
            "event": "identity",
            "uuid": uuid,
            "token": token,
        });
        sink.send(WsMessage::Text(identity.to_string())).await?;

        let (event_tx, _) = broadcast::channel::<WsEvent>(256);
        let (cmd_tx, mut cmd_rx) = mpsc::channel::<String>(64);

        let event_tx_clone = event_tx.clone();

        // Wait for ready
        let (ready_tx, ready_rx) = tokio::sync::oneshot::channel::<Result<(), Error>>();
        let mut ready_tx = Some(ready_tx);

        let handle = tokio::spawn(async move {
            loop {
                tokio::select! {
                    Some(msg) = stream.next() => {
                        match msg {
                            Ok(WsMessage::Text(text)) => {
                                if let Ok(data) = serde_json::from_str::<Value>(&text) {
                                    let event_name = data.get("event")
                                        .and_then(|e| e.as_str())
                                        .unwrap_or("")
                                        .to_string();

                                    if event_name == "ready" {
                                        if let Some(tx) = ready_tx.take() {
                                            let _ = tx.send(Ok(()));
                                        }
                                    } else if event_name == "notReady" {
                                        if let Some(tx) = ready_tx.take() {
                                            let reason = data.get("reason")
                                                .and_then(|r| r.as_str())
                                                .unwrap_or("Authentication failed");
                                            let _ = tx.send(Err(Error::Other(reason.to_string())));
                                        }
                                        break;
                                    }

                                    let _ = event_tx_clone.send(WsEvent {
                                        event: event_name,
                                        data,
                                    });
                                }
                            }
                            Ok(WsMessage::Close(_)) | Err(_) => break,
                            _ => {}
                        }
                    }
                    Some(cmd) = cmd_rx.recv() => {
                        if sink.send(WsMessage::Text(cmd)).await.is_err() {
                            break;
                        }
                    }
                    else => break,
                }
            }
        });

        // Wait for ready or error
        match ready_rx.await {
            Ok(Ok(())) => {}
            Ok(Err(e)) => return Err(e),
            Err(_) => return Err(Error::Other("Connection closed before ready".into())),
        }

        Ok(Self {
            tx: cmd_tx,
            events: event_tx,
            _handle: handle,
        })
    }

    /// Subscribe to all incoming events.
    pub fn subscribe(&self) -> broadcast::Receiver<WsEvent> {
        self.events.subscribe()
    }

    /// Send a raw JSON command over the WebSocket.
    pub async fn send(&self, data: Value) -> Result<(), Error> {
        self.tx.send(data.to_string()).await
            .map_err(|_| Error::Other("WebSocket closed".into()))
    }

    /// Send a message to specific devices.
    pub async fn send_message(&self, devices: &[&str], payload: Value) -> Result<(), Error> {
        self.send(serde_json::json!({
            "event": "message",
            "devices": devices,
            "payload": payload,
        })).await
    }

    /// Broadcast a message.
    pub async fn send_broadcast(&self, payload: Value) -> Result<(), Error> {
        self.send(serde_json::json!({
            "event": "broadcast",
            "payload": payload,
        })).await
    }

    /// Subscribe to events from another device over WebSocket.
    pub async fn subscribe_ws(&self, emitter: &Uuid, sub_type: &str) -> Result<(), Error> {
        self.send(serde_json::json!({
            "event": "subscribe",
            "emitterUuid": emitter,
            "type": sub_type,
        })).await
    }
}
