//! Stress tests for FreshBlu server.
//! All tests are marked #[ignore] - run with: cargo test -p freshblu-server stress -- --ignored

mod helpers;

use freshblu_core::device::RegisterParams;
use futures::SinkExt;
use helpers::*;
use serde_json::json;
use std::collections::HashSet;
use tokio_tungstenite::tungstenite::Message;

#[tokio::test]
#[ignore]
async fn stress_1000_ws_connections() {
    let (ws_url, state) = setup().await;

    let mut streams = Vec::new();
    let mut uuids = Vec::new();

    for _ in 0..1000 {
        let (uuid, token) = register_device(&state).await;
        let ws = connect_and_auth(&ws_url, &uuid, &token).await;
        uuids.push(uuid);
        streams.push(ws);
    }

    // All 1000 should be online
    assert_eq!(state.bus.online_count(), 1000);

    // Drop all streams
    drop(streams);

    // Wait for disconnect cleanup
    let mut offline = false;
    for _ in 0..50 {
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        if state.bus.online_count() == 0 {
            offline = true;
            break;
        }
    }
    assert!(
        offline,
        "all devices should go offline after dropping WS streams"
    );
}

#[tokio::test]
#[ignore]
async fn stress_rapid_connect_disconnect() {
    let (ws_url, state) = setup().await;

    // Pre-register 100 devices
    let mut devices = Vec::new();
    for _ in 0..100 {
        devices.push(register_device(&state).await);
    }

    for _cycle in 0..50 {
        let mut streams = Vec::new();
        for (uuid, token) in &devices {
            let ws = connect_and_auth(&ws_url, uuid, token).await;
            streams.push(ws);
        }
        // Drop all immediately
        drop(streams);
    }

    // Wait for cleanup
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    assert_eq!(
        state.bus.online_count(),
        0,
        "no leaked hub entries after rapid connect/disconnect"
    );
}

#[tokio::test]
#[ignore]
async fn stress_message_flood() {
    let (ws_url, state) = setup().await;

    let mut devices = Vec::new();
    let mut streams = Vec::new();
    for _ in 0..10 {
        let (uuid, token) = register_device(&state).await;
        let ws = connect_and_auth(&ws_url, &uuid, &token).await;
        devices.push((uuid, token));
        streams.push(ws);
    }

    // Each device sends 1000 messages to the next device
    for i in 0..10 {
        let target_idx = (i + 1) % 10;
        let target_uuid = &devices[target_idx].0;
        for n in 0..1000 {
            let msg = json!({
                "event": "message",
                "devices": [target_uuid],
                "payload": {"n": n}
            });
            let _ = streams[i].send(Message::Text(msg.to_string())).await;
        }
    }

    // Give time for messages to flow
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // No panics = success. Drain any remaining messages.
    for ws in &mut streams {
        while recv_json(ws).await.is_some() {}
    }
}

#[tokio::test]
#[ignore]
async fn stress_concurrent_registration() {
    let (_ws_url, state) = setup().await;

    let mut handles = Vec::new();
    for _ in 0..500 {
        let state = state.clone();
        handles.push(tokio::spawn(async move {
            let params = RegisterParams {
                device_type: Some("stress-test".into()),
                ..Default::default()
            };
            state.store.register(params).await.unwrap()
        }));
    }

    let mut uuids = HashSet::new();
    for handle in handles {
        let (device, _token) = handle.await.unwrap();
        uuids.insert(device.uuid);
    }

    assert_eq!(
        uuids.len(),
        500,
        "all 500 registrations should produce unique UUIDs"
    );
}

#[tokio::test]
#[ignore]
async fn stress_concurrent_auth() {
    let (_ws_url, state) = setup().await;

    // Register 100 devices
    let mut devices = Vec::new();
    for _ in 0..100 {
        devices.push(register_device(&state).await);
    }

    let mut handles = Vec::new();
    let start = std::time::Instant::now();

    // 50 tasks hammering authenticate for 3 seconds
    for i in 0..50 {
        let state = state.clone();
        let (uuid, token) = devices[i % 100].clone();
        handles.push(tokio::spawn(async move {
            let uuid_parsed: uuid::Uuid = uuid.parse().unwrap();
            let mut count = 0u64;
            while start.elapsed() < std::time::Duration::from_secs(3) {
                let result = state.store.authenticate(&uuid_parsed, &token).await;
                assert!(result.is_ok(), "auth should not error");
                assert!(result.unwrap().is_some(), "auth should succeed");
                count += 1;
            }
            count
        }));
    }

    let mut total = 0u64;
    for handle in handles {
        total += handle.await.unwrap();
    }

    assert!(total > 0, "should have completed some authentications");
}
