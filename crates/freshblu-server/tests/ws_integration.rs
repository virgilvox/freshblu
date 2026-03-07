use freshblu_server::{build_router, AppState, ServerConfig};
use freshblu_store::sqlite::SqliteStore;
use std::sync::Arc;

use futures::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio_tungstenite::{connect_async, tungstenite::Message};

async fn setup() -> (String, AppState) {
    let store: freshblu_store::DynStore =
        Arc::new(SqliteStore::new("sqlite::memory:").await.unwrap());
    let bus: freshblu_server::DynBus = Arc::new(freshblu_server::local_bus::LocalBus::new());
    let state = AppState {
        store,
        bus,
        config: ServerConfig::default(),
    };
    let app = build_router(state.clone());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let ws_url = format!("ws://127.0.0.1:{}/ws", addr.port());
    (ws_url, state)
}

async fn register_device(state: &AppState) -> (String, String) {
    use freshblu_core::device::RegisterParams;
    let params = RegisterParams {
        device_type: Some("test".into()),
        ..Default::default()
    };
    let (device, token) = state.store.register(params).await.unwrap();
    (device.uuid.to_string(), token)
}

async fn recv_json(
    ws: &mut (impl StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin),
) -> Option<Value> {
    match tokio::time::timeout(std::time::Duration::from_secs(2), ws.next()).await {
        Ok(Some(Ok(Message::Text(text)))) => serde_json::from_str(&text).ok(),
        _ => None,
    }
}

/// Helper: connect and authenticate a device, returning the WS stream split.
async fn connect_and_auth(
    ws_url: &str,
    uuid: &str,
    token: &str,
) -> tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>> {
    let (mut ws, _) = connect_async(ws_url).await.expect("failed to connect");

    let identity = json!({
        "event": "identity",
        "uuid": uuid,
        "token": token,
    });
    ws.send(Message::Text(identity.to_string())).await.unwrap();

    // Consume the "ready" response
    let resp = recv_json(&mut ws).await.expect("expected ready response");
    assert_eq!(resp["event"], "ready");

    ws
}

#[tokio::test]
async fn ws_identity_and_ready() {
    let (ws_url, state) = setup().await;
    let (uuid, token) = register_device(&state).await;

    let (mut ws, _) = connect_async(&ws_url).await.expect("failed to connect");

    let identity = json!({
        "event": "identity",
        "uuid": uuid,
        "token": token,
    });
    ws.send(Message::Text(identity.to_string())).await.unwrap();

    let resp = recv_json(&mut ws).await.expect("expected ready response");
    assert_eq!(resp["event"], "ready");
    assert_eq!(resp["uuid"], uuid);
}

#[tokio::test]
async fn ws_identity_wrong_token() {
    let (ws_url, state) = setup().await;
    let (uuid, _token) = register_device(&state).await;

    let (mut ws, _) = connect_async(&ws_url).await.expect("failed to connect");

    let identity = json!({
        "event": "identity",
        "uuid": uuid,
        "token": "completely-wrong-token",
    });
    ws.send(Message::Text(identity.to_string())).await.unwrap();

    let resp = recv_json(&mut ws)
        .await
        .expect("expected notReady response");
    assert_eq!(resp["event"], "notReady");
}

#[tokio::test]
async fn ws_whoami() {
    let (ws_url, state) = setup().await;
    let (uuid, token) = register_device(&state).await;

    let mut ws = connect_and_auth(&ws_url, &uuid, &token).await;

    let whoami = json!({ "event": "whoami" });
    ws.send(Message::Text(whoami.to_string())).await.unwrap();

    let resp = recv_json(&mut ws).await.expect("expected whoami response");
    assert_eq!(resp["event"], "whoami");
    assert!(
        resp["device"].is_object(),
        "whoami should contain device data"
    );
    assert_eq!(resp["device"]["uuid"], uuid);
}

#[tokio::test]
async fn ws_ping_pong() {
    let (ws_url, state) = setup().await;
    let (uuid, token) = register_device(&state).await;

    let mut ws = connect_and_auth(&ws_url, &uuid, &token).await;

    let ping = json!({ "event": "ping" });
    ws.send(Message::Text(ping.to_string())).await.unwrap();

    let resp = recv_json(&mut ws).await.expect("expected pong response");
    assert_eq!(resp["event"], "pong");
}

#[tokio::test]
async fn ws_receive_message() {
    use freshblu_core::message::{DeviceEvent, Message as FreshbluMessage};

    let (ws_url, state) = setup().await;

    // Device A connects via WS
    let (uuid_a, token_a) = register_device(&state).await;
    let mut ws_a = connect_and_auth(&ws_url, &uuid_a, &token_a).await;

    // Device B is registered (no WS connection needed)
    let (uuid_b, _token_b) = register_device(&state).await;

    // Deliver a message from B to A through the bus
    let device_a_uuid: uuid::Uuid = uuid_a.parse().unwrap();
    let sender_uuid: uuid::Uuid = uuid_b.parse().unwrap();

    let msg = FreshbluMessage {
        devices: vec![uuid_a.clone()],
        from_uuid: Some(sender_uuid),
        topic: None,
        payload: Some(json!({"hello": "world"})),
        metadata: None,
        extra: std::collections::HashMap::new(),
    };
    let _ = state
        .bus
        .publish(&device_a_uuid, DeviceEvent::Message(msg))
        .await;

    // Device A should receive the message on its WS
    let resp = recv_json(&mut ws_a).await.expect("expected message event");
    assert_eq!(resp["event"], "message");
    assert_eq!(resp["payload"]["hello"], "world");
    assert_eq!(resp["fromUuid"], uuid_b);
}

#[tokio::test]
async fn ws_receive_config_update() {
    use freshblu_core::message::DeviceEvent;

    let (ws_url, state) = setup().await;
    let (uuid, token) = register_device(&state).await;

    let mut ws = connect_and_auth(&ws_url, &uuid, &token).await;

    // Update device properties via the store
    let device_uuid: uuid::Uuid = uuid.parse().unwrap();
    let mut props = std::collections::HashMap::new();
    props.insert("name".to_string(), json!("updated-device"));
    let updated = state
        .store
        .update_device(&device_uuid, props)
        .await
        .unwrap();

    // Deliver config event through bus
    let config_event = DeviceEvent::Config {
        device: Box::new(updated.to_view()),
    };
    let _ = state.bus.publish(&device_uuid, config_event).await;

    // Verify WS receives the config update
    let resp = recv_json(&mut ws).await.expect("expected config event");
    assert_eq!(resp["event"], "config");
    assert!(resp["device"].is_object());
    assert_eq!(resp["device"]["uuid"], uuid);
}

#[tokio::test]
async fn ws_disconnect_sets_offline() {
    let (ws_url, state) = setup().await;
    let (uuid, token) = register_device(&state).await;

    let ws = connect_and_auth(&ws_url, &uuid, &token).await;

    let device_uuid: uuid::Uuid = uuid.parse().unwrap();

    // After auth, the bus should show the device as online
    assert!(
        state.bus.is_online(&device_uuid),
        "device should be online in bus after WS auth"
    );

    // Close the WebSocket connection
    drop(ws);

    // Poll until the bus shows the device as offline
    let mut went_offline = false;
    for _ in 0..20 {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        if !state.bus.is_online(&device_uuid) {
            went_offline = true;
            break;
        }
    }
    assert!(
        went_offline,
        "device should be offline in bus after WS disconnect"
    );

    // Also verify the store reflects offline status
    let device = state.store.get_device(&device_uuid).await.unwrap().unwrap();
    assert!(
        !device.online,
        "device should be offline in store after WS disconnect"
    );
}

#[tokio::test]
async fn ws_subscribe_permission_denied() {
    use freshblu_core::permissions::Whitelists;

    let (ws_url, state) = setup().await;

    // Device A connects via WS
    let (uuid_a, token_a) = register_device(&state).await;
    let mut ws_a = connect_and_auth(&ws_url, &uuid_a, &token_a).await;

    // Device B is registered with empty (private) whitelists
    let params_b = freshblu_core::device::RegisterParams {
        device_type: Some("private".into()),
        meshblu: Some(freshblu_core::device::WhitelistOverride {
            whitelists: Some(Whitelists::default()),
        }),
        ..Default::default()
    };
    let (device_b, _token_b) = state.store.register(params_b).await.unwrap();

    // Device A tries to subscribe to private device B's broadcasts
    let subscribe = json!({
        "event": "subscribe",
        "emitterUuid": device_b.uuid.to_string(),
        "type": "broadcast.sent",
    });
    ws_a.send(Message::Text(subscribe.to_string()))
        .await
        .unwrap();

    // Should receive an error event
    let resp = recv_json(&mut ws_a).await.expect("expected error response");
    assert_eq!(resp["event"], "error");
}
