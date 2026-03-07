mod helpers;

use helpers::*;
use futures::SinkExt;
use serde_json::json;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use uuid::Uuid;

#[tokio::test]
async fn ws_auth_invalid_uuid() {
    let (ws_url, _state) = setup().await;

    let (mut ws, _) = connect_async(&ws_url).await.expect("failed to connect");

    let identity = json!({
        "event": "identity",
        "uuid": "not-a-uuid",
        "token": "anything",
    });
    ws.send(Message::Text(identity.to_string())).await.unwrap();

    let resp = recv_json(&mut ws).await.expect("expected notReady response");
    assert_eq!(resp["event"], "notReady");
    assert!(
        resp["reason"].as_str().is_some(),
        "notReady should include a reason"
    );

    // Connection should still be open -- verify with a ping
    let ping = json!({ "event": "ping" });
    ws.send(Message::Text(ping.to_string())).await.unwrap();
    let pong = recv_json(&mut ws).await.expect("connection should still be alive");
    assert_eq!(pong["event"], "pong");
}

#[tokio::test]
async fn ws_auth_nonexistent_device() {
    let (ws_url, _state) = setup().await;

    let (mut ws, _) = connect_async(&ws_url).await.expect("failed to connect");

    let fake_uuid = Uuid::new_v4().to_string();
    let identity = json!({
        "event": "identity",
        "uuid": fake_uuid,
        "token": "some-token",
    });
    ws.send(Message::Text(identity.to_string())).await.unwrap();

    let resp = recv_json(&mut ws).await.expect("expected notReady response");
    assert_eq!(resp["event"], "notReady");
}

#[tokio::test]
async fn ws_register_creates_device() {
    let (ws_url, state) = setup().await;
    let (uuid, token) = register_device(&state).await;

    let mut ws = connect_and_auth(&ws_url, &uuid, &token).await;

    let register = json!({
        "event": "register",
        "type": "new-device",
    });
    ws.send(Message::Text(register.to_string())).await.unwrap();

    let resp = recv_json(&mut ws).await.expect("expected registered response");
    assert_eq!(resp["event"], "registered");
    assert!(resp["uuid"].is_string(), "registered event should include uuid");
    assert!(resp["token"].is_string(), "registered event should include token");

    // Verify device exists in the store
    let new_uuid: Uuid = resp["uuid"]
        .as_str()
        .unwrap()
        .parse()
        .expect("returned uuid should be valid");
    let device = state.store.get_device(&new_uuid).await.unwrap();
    assert!(device.is_some(), "registered device should exist in store");
}

#[tokio::test]
async fn ws_unregister_self_removes() {
    let (ws_url, state) = setup().await;
    let (uuid, token) = register_device(&state).await;

    let mut ws = connect_and_auth(&ws_url, &uuid, &token).await;

    let unregister = json!({
        "event": "unregister",
        "uuid": uuid,
    });
    ws.send(Message::Text(unregister.to_string())).await.unwrap();

    // Device should be removed from the store
    let device_uuid: Uuid = uuid.parse().unwrap();
    // Give the handler a moment to process
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    let fetched = state.store.get_device(&device_uuid).await.unwrap();
    assert!(fetched.is_none(), "device should be removed after self-unregister");

    // The WS stream should close (next read returns None or Close)
    let next = recv_json(&mut ws).await;
    assert!(next.is_none(), "WS should close after self-unregister");
}

#[tokio::test]
async fn ws_unregister_other_denied() {
    let (ws_url, state) = setup().await;
    let (uuid_a, token_a) = register_device(&state).await;
    let (uuid_b, _token_b) = register_device(&state).await;

    let mut ws_a = connect_and_auth(&ws_url, &uuid_a, &token_a).await;

    // A tries to unregister B
    let unregister = json!({
        "event": "unregister",
        "uuid": uuid_b,
    });
    ws_a.send(Message::Text(unregister.to_string())).await.unwrap();

    // Give the handler a moment
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // B should still exist
    let device_b_uuid: Uuid = uuid_b.parse().unwrap();
    let fetched = state.store.get_device(&device_b_uuid).await.unwrap();
    assert!(
        fetched.is_some(),
        "device B should still exist after A tried to unregister it"
    );

    // A's connection should still be alive
    let ping = json!({ "event": "ping" });
    ws_a.send(Message::Text(ping.to_string())).await.unwrap();
    let pong = recv_json(&mut ws_a).await.expect("A's WS should still be alive");
    assert_eq!(pong["event"], "pong");
}

#[tokio::test]
async fn ws_malformed_messages_tolerated() {
    let (ws_url, state) = setup().await;
    let (uuid, token) = register_device(&state).await;

    let mut ws = connect_and_auth(&ws_url, &uuid, &token).await;

    // Send garbage JSON
    ws.send(Message::Text("{not valid json!!!".to_string()))
        .await
        .unwrap();

    // Send empty string
    ws.send(Message::Text(String::new())).await.unwrap();

    // Send binary frame
    ws.send(Message::Binary(vec![0xDE, 0xAD, 0xBE, 0xEF]))
        .await
        .unwrap();

    // Give time for server to process
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // Connection should still work
    let ping = json!({ "event": "ping" });
    ws.send(Message::Text(ping.to_string())).await.unwrap();
    let pong = recv_json(&mut ws).await.expect("connection should survive malformed messages");
    assert_eq!(pong["event"], "pong");
}

#[tokio::test]
async fn ws_multiple_connections_same_device() {
    use freshblu_core::message::{DeviceEvent, Message as FreshbluMessage};

    let (ws_url, state) = setup().await;
    let (uuid, token) = register_device(&state).await;

    let mut ws1 = connect_and_auth(&ws_url, &uuid, &token).await;
    let mut ws2 = connect_and_auth(&ws_url, &uuid, &token).await;

    // Publish a message targeting this device
    let device_uuid: Uuid = uuid.parse().unwrap();
    let msg = FreshbluMessage {
        devices: vec![uuid.clone()],
        from_uuid: Some(Uuid::new_v4()),
        topic: None,
        payload: Some(json!({"test": "multi-conn"})),
        metadata: None,
        extra: std::collections::HashMap::new(),
    };
    let _ = state
        .bus
        .publish(&device_uuid, DeviceEvent::Message(msg))
        .await;

    // Both connections should receive the message
    let resp1 = recv_json(&mut ws1).await.expect("ws1 should receive message");
    assert_eq!(resp1["event"], "message");
    assert_eq!(resp1["payload"]["test"], "multi-conn");

    let resp2 = recv_json(&mut ws2).await.expect("ws2 should receive message");
    assert_eq!(resp2["event"], "message");
    assert_eq!(resp2["payload"]["test"], "multi-conn");
}

#[tokio::test]
async fn ws_subscribe_then_receive_broadcast() {
    let (ws_url, state) = setup().await;

    // Device A will subscribe to B's broadcasts
    let (uuid_a, token_a) = register_device(&state).await;
    // Device B needs open whitelists (register_device gives open whitelists)
    let (uuid_b, token_b) = register_device(&state).await;

    let mut ws_a = connect_and_auth(&ws_url, &uuid_a, &token_a).await;

    // A subscribes to B's broadcast.sent
    let subscribe = json!({
        "event": "subscribe",
        "emitterUuid": uuid_b,
        "type": "broadcast.sent",
    });
    ws_a.send(Message::Text(subscribe.to_string())).await.unwrap();

    // Give time for subscription to be processed
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // B broadcasts via a second WS connection
    let mut ws_b = connect_and_auth(&ws_url, &uuid_b, &token_b).await;
    let broadcast = json!({
        "event": "message",
        "devices": ["*"],
        "payload": {"data": "hello-broadcast"},
    });
    ws_b.send(Message::Text(broadcast.to_string())).await.unwrap();

    // A should receive the broadcast
    let resp = recv_json(&mut ws_a).await.expect("A should receive broadcast from B");
    assert_eq!(resp["event"], "broadcast");
    assert_eq!(resp["payload"]["data"], "hello-broadcast");
}

#[tokio::test]
async fn ws_unsubscribe_stops_events() {
    let (ws_url, state) = setup().await;

    let (uuid_a, token_a) = register_device(&state).await;
    let (uuid_b, token_b) = register_device(&state).await;

    let mut ws_a = connect_and_auth(&ws_url, &uuid_a, &token_a).await;

    // A subscribes to B's broadcast.sent
    let subscribe = json!({
        "event": "subscribe",
        "emitterUuid": uuid_b,
        "type": "broadcast.sent",
    });
    ws_a.send(Message::Text(subscribe.to_string())).await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // B broadcasts -- A should receive it
    let mut ws_b = connect_and_auth(&ws_url, &uuid_b, &token_b).await;
    let broadcast1 = json!({
        "event": "message",
        "devices": ["*"],
        "payload": {"seq": 1},
    });
    ws_b.send(Message::Text(broadcast1.to_string())).await.unwrap();

    let resp = recv_json(&mut ws_a).await.expect("A should receive first broadcast");
    assert_eq!(resp["event"], "broadcast");
    assert_eq!(resp["payload"]["seq"], 1);

    // A unsubscribes from B
    let unsubscribe = json!({
        "event": "unsubscribe",
        "emitterUuid": uuid_b,
        "type": "broadcast.sent",
    });
    ws_a.send(Message::Text(unsubscribe.to_string())).await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // B broadcasts again -- A should NOT receive it
    let broadcast2 = json!({
        "event": "message",
        "devices": ["*"],
        "payload": {"seq": 2},
    });
    ws_b.send(Message::Text(broadcast2.to_string())).await.unwrap();

    let resp = recv_json(&mut ws_a).await;
    assert!(
        resp.is_none(),
        "A should NOT receive broadcasts after unsubscribing"
    );
}
