mod helpers;

use helpers::*;
use freshblu_core::subscription::{CreateSubscriptionParams, SubscriptionType};
use freshblu_server::build_router;
use futures::SinkExt;
use serde_json::json;
use tokio_tungstenite::tungstenite::Message;
use axum::http::{Method, StatusCode};

// ---------------------------------------------------------------------------
// 1. HTTP -> WS delivery: A sends HTTP POST /messages to B, B receives via WS
// ---------------------------------------------------------------------------

#[tokio::test]
async fn http_to_ws_delivery() {
    let (ws_url, state) = setup().await;

    let (uuid_a, token_a) = register_device(&state).await;
    let (uuid_b, token_b) = register_device(&state).await;

    // B connects via WebSocket
    let mut ws_b = connect_and_auth(&ws_url, &uuid_b, &token_b).await;

    // A sends a message to B via HTTP (using oneshot on a router built from same state)
    let app = build_router(state.clone());
    let auth = basic_auth(&uuid_a, &token_a);
    let resp = http_request(
        &app,
        Method::POST,
        "/messages",
        Some(&auth),
        Some(json!({
            "devices": [uuid_b],
            "payload": {"hello": "http"}
        })),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);

    // B should receive the message on its WS stream
    let msg = recv_json(&mut ws_b).await.expect("B should receive a message");
    assert_eq!(msg["event"], "message");
    assert_eq!(msg["payload"]["hello"], "http");
    assert_eq!(msg["fromUuid"], uuid_a);
}

// ---------------------------------------------------------------------------
// 2. HTTP -> WS denied: A sends to private B, B receives nothing
// ---------------------------------------------------------------------------

#[tokio::test]
async fn http_to_ws_denied() {
    let (ws_url, state) = setup().await;

    let (uuid_a, token_a) = register_device(&state).await;
    let (uuid_b, token_b) = register_private_device(&state).await;

    // B connects via WebSocket
    let mut ws_b = connect_and_auth(&ws_url, &uuid_b, &token_b).await;

    // A tries to send a message to private B via HTTP
    let app = build_router(state.clone());
    let auth = basic_auth(&uuid_a, &token_a);
    let resp = http_request(
        &app,
        Method::POST,
        "/messages",
        Some(&auth),
        Some(json!({
            "devices": [uuid_b],
            "payload": {"should": "not arrive"}
        })),
    )
    .await;
    // The HTTP call itself succeeds (server silently skips unauthorized targets)
    assert_eq!(resp.status(), StatusCode::OK);

    // B should NOT receive anything (recv_json times out after 2s)
    let msg = recv_json(&mut ws_b).await;
    assert!(msg.is_none(), "private device B should not receive the message");
}

// ---------------------------------------------------------------------------
// 3. WS -> WS delivery: A sends message to B via WS, B receives it
// ---------------------------------------------------------------------------

#[tokio::test]
async fn ws_to_ws_delivery() {
    let (ws_url, state) = setup().await;

    let (uuid_a, token_a) = register_device(&state).await;
    let (uuid_b, token_b) = register_device(&state).await;

    let mut ws_a = connect_and_auth(&ws_url, &uuid_a, &token_a).await;
    let mut ws_b = connect_and_auth(&ws_url, &uuid_b, &token_b).await;

    // A sends a message to B via its WS connection
    let msg = json!({
        "event": "message",
        "devices": [uuid_b],
        "payload": {"hello": "ws"}
    });
    ws_a.send(Message::Text(msg.to_string())).await.unwrap();

    // B should receive the message
    let received = recv_json(&mut ws_b).await.expect("B should receive a WS message");
    assert_eq!(received["event"], "message");
    assert_eq!(received["payload"]["hello"], "ws");
    assert_eq!(received["fromUuid"], uuid_a);
}

// ---------------------------------------------------------------------------
// 4. Broadcast fanout: E broadcasts, 3 subscribers all receive
// ---------------------------------------------------------------------------

#[tokio::test]
async fn broadcast_fanout_3_subscribers() {
    let (ws_url, state) = setup().await;

    // Emitter device
    let (uuid_e, token_e) = register_device(&state).await;

    // 3 subscriber devices
    let (uuid_s1, token_s1) = register_device(&state).await;
    let (uuid_s2, token_s2) = register_device(&state).await;
    let (uuid_s3, token_s3) = register_device(&state).await;

    let emitter_uuid: uuid::Uuid = uuid_e.parse().unwrap();
    let s1_uuid: uuid::Uuid = uuid_s1.parse().unwrap();
    let s2_uuid: uuid::Uuid = uuid_s2.parse().unwrap();
    let s3_uuid: uuid::Uuid = uuid_s3.parse().unwrap();

    // Create broadcast.sent subscriptions: S1, S2, S3 -> E
    for sub_uuid in [s1_uuid, s2_uuid, s3_uuid] {
        let params = CreateSubscriptionParams {
            emitter_uuid,
            subscriber_uuid: sub_uuid,
            subscription_type: SubscriptionType::BroadcastSent,
        };
        state.store.create_subscription(&params).await.unwrap();
    }

    // Connect all 3 subscribers via WS
    let mut ws_s1 = connect_and_auth(&ws_url, &uuid_s1, &token_s1).await;
    let mut ws_s2 = connect_and_auth(&ws_url, &uuid_s2, &token_s2).await;
    let mut ws_s3 = connect_and_auth(&ws_url, &uuid_s3, &token_s3).await;

    // E sends a broadcast via HTTP
    let app = build_router(state.clone());
    let auth = basic_auth(&uuid_e, &token_e);
    let resp = http_request(
        &app,
        Method::POST,
        "/messages",
        Some(&auth),
        Some(json!({
            "devices": ["*"],
            "payload": {"broadcast": true}
        })),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);

    // All 3 subscribers should receive the broadcast
    let m1 = recv_json(&mut ws_s1).await.expect("S1 should receive broadcast");
    assert_eq!(m1["event"], "broadcast");
    assert_eq!(m1["payload"]["broadcast"], true);
    assert_eq!(m1["fromUuid"], uuid_e);

    let m2 = recv_json(&mut ws_s2).await.expect("S2 should receive broadcast");
    assert_eq!(m2["event"], "broadcast");
    assert_eq!(m2["payload"]["broadcast"], true);

    let m3 = recv_json(&mut ws_s3).await.expect("S3 should receive broadcast");
    assert_eq!(m3["event"], "broadcast");
    assert_eq!(m3["payload"]["broadcast"], true);
}

// ---------------------------------------------------------------------------
// 5. message.sent subscriber notified: C subscribes to A's message.sent,
//    A sends to B, C gets notified
// ---------------------------------------------------------------------------

#[tokio::test]
async fn message_sent_subscriber_notified() {
    let (ws_url, state) = setup().await;

    let (uuid_a, token_a) = register_device(&state).await;
    let (uuid_b, _token_b) = register_device(&state).await;
    let (uuid_c, token_c) = register_device(&state).await;

    let a_uuid: uuid::Uuid = uuid_a.parse().unwrap();
    let c_uuid: uuid::Uuid = uuid_c.parse().unwrap();

    // C subscribes to A's message.sent
    let params = CreateSubscriptionParams {
        emitter_uuid: a_uuid,
        subscriber_uuid: c_uuid,
        subscription_type: SubscriptionType::MessageSent,
    };
    state.store.create_subscription(&params).await.unwrap();

    // Connect C via WS
    let mut ws_c = connect_and_auth(&ws_url, &uuid_c, &token_c).await;

    // A sends a message to B via HTTP
    let app = build_router(state.clone());
    let auth = basic_auth(&uuid_a, &token_a);
    let resp = http_request(
        &app,
        Method::POST,
        "/messages",
        Some(&auth),
        Some(json!({
            "devices": [uuid_b],
            "payload": {"data": "from_a"}
        })),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);

    // C should receive the message.sent notification
    let msg = recv_json(&mut ws_c).await.expect("C should be notified of A's message.sent");
    assert_eq!(msg["event"], "message");
    assert_eq!(msg["payload"]["data"], "from_a");
    assert_eq!(msg["fromUuid"], uuid_a);
}

// ---------------------------------------------------------------------------
// 6. message.received subscriber notified: C subscribes to B's message.received,
//    A sends to B, C gets notified
// ---------------------------------------------------------------------------

#[tokio::test]
async fn message_received_subscriber_notified() {
    let (ws_url, state) = setup().await;

    let (uuid_a, token_a) = register_device(&state).await;
    let (uuid_b, _token_b) = register_device(&state).await;
    let (uuid_c, token_c) = register_device(&state).await;

    let b_uuid: uuid::Uuid = uuid_b.parse().unwrap();
    let c_uuid: uuid::Uuid = uuid_c.parse().unwrap();

    // C subscribes to B's message.received
    let params = CreateSubscriptionParams {
        emitter_uuid: b_uuid,
        subscriber_uuid: c_uuid,
        subscription_type: SubscriptionType::MessageReceived,
    };
    state.store.create_subscription(&params).await.unwrap();

    // Connect C via WS
    let mut ws_c = connect_and_auth(&ws_url, &uuid_c, &token_c).await;

    // A sends a message to B via HTTP
    let app = build_router(state.clone());
    let auth = basic_auth(&uuid_a, &token_a);
    let resp = http_request(
        &app,
        Method::POST,
        "/messages",
        Some(&auth),
        Some(json!({
            "devices": [uuid_b],
            "payload": {"data": "for_b"}
        })),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);

    // C should receive the message.received notification
    let msg = recv_json(&mut ws_c).await.expect("C should be notified of B's message.received");
    assert_eq!(msg["event"], "message");
    assert_eq!(msg["payload"]["data"], "for_b");
    assert_eq!(msg["fromUuid"], uuid_a);
}

// ---------------------------------------------------------------------------
// 7. configure.sent on update: C subscribes to A's configure.sent,
//    A is updated via HTTP, C gets notified
// ---------------------------------------------------------------------------

#[tokio::test]
async fn configure_sent_on_update() {
    let (ws_url, state) = setup().await;

    let (uuid_a, token_a) = register_device(&state).await;
    let (uuid_c, token_c) = register_device(&state).await;

    let a_uuid: uuid::Uuid = uuid_a.parse().unwrap();
    let c_uuid: uuid::Uuid = uuid_c.parse().unwrap();

    // C subscribes to A's configure.sent
    let params = CreateSubscriptionParams {
        emitter_uuid: a_uuid,
        subscriber_uuid: c_uuid,
        subscription_type: SubscriptionType::ConfigureSent,
    };
    state.store.create_subscription(&params).await.unwrap();

    // Connect C via WS
    let mut ws_c = connect_and_auth(&ws_url, &uuid_c, &token_c).await;

    // Update A via HTTP PUT /devices/:uuid
    let app = build_router(state.clone());
    let auth = basic_auth(&uuid_a, &token_a);
    let resp = http_request(
        &app,
        Method::PUT,
        &format!("/devices/{}", uuid_a),
        Some(&auth),
        Some(json!({"name": "updated"})),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);

    // C should receive a config event
    let msg = recv_json(&mut ws_c).await.expect("C should be notified of A's configure.sent");
    assert_eq!(msg["event"], "config");
    assert!(msg["device"].is_object(), "config event should contain device data");
    assert_eq!(msg["device"]["uuid"], uuid_a);
}

// ---------------------------------------------------------------------------
// 8. unregister.sent on delete: C subscribes to A's unregister.sent,
//    A is deleted, C gets notified
// ---------------------------------------------------------------------------

#[tokio::test]
async fn unregister_sent_on_delete() {
    let (ws_url, state) = setup().await;

    let (uuid_a, token_a) = register_device(&state).await;
    let (uuid_c, token_c) = register_device(&state).await;

    let a_uuid: uuid::Uuid = uuid_a.parse().unwrap();
    let c_uuid: uuid::Uuid = uuid_c.parse().unwrap();

    // C subscribes to A's unregister.sent
    let params = CreateSubscriptionParams {
        emitter_uuid: a_uuid,
        subscriber_uuid: c_uuid,
        subscription_type: SubscriptionType::UnregisterSent,
    };
    state.store.create_subscription(&params).await.unwrap();

    // Connect C via WS
    let mut ws_c = connect_and_auth(&ws_url, &uuid_c, &token_c).await;

    // Delete A via HTTP DELETE /devices/:uuid
    let app = build_router(state.clone());
    let auth = basic_auth(&uuid_a, &token_a);
    let resp = http_request(
        &app,
        Method::DELETE,
        &format!("/devices/{}", uuid_a),
        Some(&auth),
        None,
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);

    // C should receive an unregistered event
    let msg = recv_json(&mut ws_c).await.expect("C should be notified of A's unregister.sent");
    assert_eq!(msg["event"], "unregistered");
    assert_eq!(msg["uuid"], uuid_a);
}

// ---------------------------------------------------------------------------
// 9. configure.sent on WS update: C subscribes to A's configure.sent,
//    A updates itself via WS, C gets notified (regression for A1 fix)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn configure_sent_on_ws_update() {
    let (ws_url, state) = setup().await;

    let (uuid_a, token_a) = register_device(&state).await;
    let (uuid_c, token_c) = register_device(&state).await;

    let a_uuid: uuid::Uuid = uuid_a.parse().unwrap();
    let c_uuid: uuid::Uuid = uuid_c.parse().unwrap();

    // C subscribes to A's configure.sent
    let params = CreateSubscriptionParams {
        emitter_uuid: a_uuid,
        subscriber_uuid: c_uuid,
        subscription_type: SubscriptionType::ConfigureSent,
    };
    state.store.create_subscription(&params).await.unwrap();

    // Connect A and C via WS
    let mut ws_a = connect_and_auth(&ws_url, &uuid_a, &token_a).await;
    let mut ws_c = connect_and_auth(&ws_url, &uuid_c, &token_c).await;

    // A updates itself via WS
    let update = json!({
        "event": "update",
        "name": "ws-updated"
    });
    ws_a.send(Message::Text(update.to_string())).await.unwrap();

    // C should receive a config event (fan out from WS Update path)
    let msg = recv_json(&mut ws_c).await.expect("C should be notified of A's configure.sent via WS");
    assert_eq!(msg["event"], "config");
    assert!(msg["device"].is_object());
    assert_eq!(msg["device"]["uuid"], uuid_a);
}

// ---------------------------------------------------------------------------
// 10. Partial delivery: send to mixed open/private targets
// ---------------------------------------------------------------------------

#[tokio::test]
async fn partial_delivery_some_allowed_some_denied() {
    let (ws_url, state) = setup().await;

    let (sender_uuid, sender_token) = register_device(&state).await;

    // T1 and T3: open (anyone can message)
    let (t1_uuid, t1_token) = register_device(&state).await;
    let (t3_uuid, t3_token) = register_device(&state).await;

    // T2: private (locked down)
    let (t2_uuid, t2_token) = register_private_device(&state).await;

    // Connect all targets via WS
    let mut ws_t1 = connect_and_auth(&ws_url, &t1_uuid, &t1_token).await;
    let mut ws_t2 = connect_and_auth(&ws_url, &t2_uuid, &t2_token).await;
    let mut ws_t3 = connect_and_auth(&ws_url, &t3_uuid, &t3_token).await;

    // Sender sends to all three
    let app = build_router(state.clone());
    let auth = basic_auth(&sender_uuid, &sender_token);
    let resp = http_request(
        &app,
        Method::POST,
        "/messages",
        Some(&auth),
        Some(json!({
            "devices": [t1_uuid, t2_uuid, t3_uuid],
            "payload": {"test": "partial"}
        })),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);

    // T1 should receive
    let m1 = recv_json(&mut ws_t1).await.expect("T1 (open) should receive message");
    assert_eq!(m1["event"], "message");
    assert_eq!(m1["payload"]["test"], "partial");

    // T2 should NOT receive
    let m2 = recv_json(&mut ws_t2).await;
    assert!(m2.is_none(), "T2 (private) should not receive message");

    // T3 should receive
    let m3 = recv_json(&mut ws_t3).await.expect("T3 (open) should receive message");
    assert_eq!(m3["event"], "message");
    assert_eq!(m3["payload"]["test"], "partial");
}
