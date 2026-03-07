//! Data-driven permission matrix tests for the Meshblu v2.0 whitelist system.
//!
//! Tests every combination of allowed/denied/self/wildcard for:
//! - discover.view  (GET /devices/:uuid)
//! - configure.update  (PUT /devices/:uuid, DELETE /devices/:uuid)
//! - message.from  (POST /messages)
//! - discover.as / configure.as / message.as  (x-meshblu-as header)
//! - WS subscribe permission checks for all 8 subscription types

mod helpers;
use helpers::*;

use freshblu_core::device::WhitelistEntry;
use freshblu_core::permissions::*;
use freshblu_store::DeviceStore;

use axum::http::{Method, StatusCode};
use futures::SinkExt;
use serde_json::json;
use tokio_tungstenite::tungstenite::Message;
use tower::ServiceExt;

// ---------------------------------------------------------------------------
// Whitelist builder helpers
// ---------------------------------------------------------------------------

fn entry_for(uuid: &str) -> WhitelistEntry {
    WhitelistEntry {
        uuid: uuid.to_string(),
    }
}

fn wildcard_entry() -> WhitelistEntry {
    WhitelistEntry::wildcard()
}

/// Start from fully-open whitelists, then override a specific field.
/// This ensures the device is always discoverable unless we explicitly lock that down.
fn open_with<F>(f: F) -> Whitelists
where
    F: FnOnce(&mut Whitelists),
{
    let mut w = Whitelists::open();
    f(&mut w);
    w
}

/// Completely locked-down whitelists (all lists empty).
fn locked() -> Whitelists {
    Whitelists::default()
}

/// Locked-down whitelists but with discover.view open so the device can be found.
fn locked_but_discoverable() -> Whitelists {
    let mut w = Whitelists::default();
    w.discover.view = vec![wildcard_entry()];
    w
}

// ===========================================================================
// 3.1  HTTP Permission Tests
// ===========================================================================

// ---------------------------------------------------------------------------
// discover.view  (GET /devices/:uuid)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn discover_view_allowed() {
    let (app, state) = setup_router().await;
    let (actor_uuid, actor_token) = register_device(&state).await;

    // Target device: only actor in discover.view
    let wl = open_with(|w| {
        w.discover.view = vec![entry_for(&actor_uuid)];
    });
    let (target_uuid, _) = register_device_with_whitelists(&state, wl).await;

    let auth = basic_auth(&actor_uuid, &actor_token);
    let resp = http_request(&app, Method::GET, &format!("/devices/{}", target_uuid), Some(&auth), None).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn discover_view_denied() {
    let (app, state) = setup_router().await;
    let (actor_uuid, actor_token) = register_device(&state).await;

    // Target device: empty discover.view (private)
    let wl = locked();
    let (target_uuid, _) = register_device_with_whitelists(&state, wl).await;

    let auth = basic_auth(&actor_uuid, &actor_token);
    let resp = http_request(&app, Method::GET, &format!("/devices/{}", target_uuid), Some(&auth), None).await;
    // Meshblu returns 404 (not 403) for discover denial
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn discover_view_self() {
    let (app, state) = setup_router().await;
    // Private device queries itself
    let wl = locked();
    let (uuid, token) = register_device_with_whitelists(&state, wl).await;

    let auth = basic_auth(&uuid, &token);
    let resp = http_request(&app, Method::GET, &format!("/devices/{}", uuid), Some(&auth), None).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn discover_view_wildcard() {
    let (app, state) = setup_router().await;
    let (actor_uuid, actor_token) = register_device(&state).await;

    let wl = open_with(|w| {
        w.discover.view = vec![wildcard_entry()];
    });
    let (target_uuid, _) = register_device_with_whitelists(&state, wl).await;

    let auth = basic_auth(&actor_uuid, &actor_token);
    let resp = http_request(&app, Method::GET, &format!("/devices/{}", target_uuid), Some(&auth), None).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn discover_view_empty() {
    let (app, state) = setup_router().await;
    let (actor_uuid, actor_token) = register_device(&state).await;

    // Completely empty discover.view, actor is not self
    let wl = open_with(|w| {
        w.discover.view = vec![];
    });
    let (target_uuid, _) = register_device_with_whitelists(&state, wl).await;

    let auth = basic_auth(&actor_uuid, &actor_token);
    let resp = http_request(&app, Method::GET, &format!("/devices/{}", target_uuid), Some(&auth), None).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

// ---------------------------------------------------------------------------
// configure.update  (PUT /devices/:uuid)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn configure_update_allowed() {
    let (app, state) = setup_router().await;
    let (actor_uuid, actor_token) = register_device(&state).await;

    let wl = open_with(|w| {
        w.configure.update = vec![entry_for(&actor_uuid)];
    });
    let (target_uuid, _) = register_device_with_whitelists(&state, wl).await;

    let auth = basic_auth(&actor_uuid, &actor_token);
    let body = json!({"color": "red"});
    let resp = http_request(&app, Method::PUT, &format!("/devices/{}", target_uuid), Some(&auth), Some(body)).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let target: uuid::Uuid = target_uuid.parse().unwrap();
    let device = state.store.get_device(&target).await.unwrap().unwrap();
    assert_eq!(device.properties.get("color"), Some(&json!("red")),
        "property should be persisted in store after update");
}

#[tokio::test]
async fn configure_update_denied() {
    let (app, state) = setup_router().await;
    let (actor_uuid, actor_token) = register_device(&state).await;

    // Target: discoverable but configure.update is empty
    let wl = locked_but_discoverable();
    let (target_uuid, _) = register_device_with_whitelists(&state, wl).await;

    let auth = basic_auth(&actor_uuid, &actor_token);
    let body = json!({"color": "red"});
    let resp = http_request(&app, Method::PUT, &format!("/devices/{}", target_uuid), Some(&auth), Some(body)).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn configure_update_self() {
    let (app, state) = setup_router().await;

    // Private device updates itself
    let wl = locked();
    let (uuid, token) = register_device_with_whitelists(&state, wl).await;

    let auth = basic_auth(&uuid, &token);
    let body = json!({"color": "green"});
    let resp = http_request(&app, Method::PUT, &format!("/devices/{}", uuid), Some(&auth), Some(body)).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn configure_update_wildcard() {
    let (app, state) = setup_router().await;
    let (actor_uuid, actor_token) = register_device(&state).await;

    let wl = open_with(|w| {
        w.configure.update = vec![wildcard_entry()];
    });
    let (target_uuid, _) = register_device_with_whitelists(&state, wl).await;

    let auth = basic_auth(&actor_uuid, &actor_token);
    let body = json!({"color": "yellow"});
    let resp = http_request(&app, Method::PUT, &format!("/devices/{}", target_uuid), Some(&auth), Some(body)).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let target: uuid::Uuid = target_uuid.parse().unwrap();
    let device = state.store.get_device(&target).await.unwrap().unwrap();
    assert_eq!(device.properties.get("color"), Some(&json!("yellow")),
        "property should be persisted in store after update");
}

#[tokio::test]
async fn configure_update_empty() {
    let (app, state) = setup_router().await;
    let (actor_uuid, actor_token) = register_device(&state).await;

    let wl = locked_but_discoverable();
    let (target_uuid, _) = register_device_with_whitelists(&state, wl).await;

    let auth = basic_auth(&actor_uuid, &actor_token);
    let body = json!({"color": "purple"});
    let resp = http_request(&app, Method::PUT, &format!("/devices/{}", target_uuid), Some(&auth), Some(body)).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

// ---------------------------------------------------------------------------
// configure.update for DELETE  (DELETE /devices/:uuid)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn unregister_allowed() {
    let (app, state) = setup_router().await;
    let (actor_uuid, actor_token) = register_device(&state).await;

    let wl = open_with(|w| {
        w.configure.update = vec![entry_for(&actor_uuid)];
    });
    let (target_uuid, _) = register_device_with_whitelists(&state, wl).await;

    let auth = basic_auth(&actor_uuid, &actor_token);
    let resp = http_request(&app, Method::DELETE, &format!("/devices/{}", target_uuid), Some(&auth), None).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn unregister_denied() {
    let (app, state) = setup_router().await;
    let (actor_uuid, actor_token) = register_device(&state).await;

    let wl = locked_but_discoverable();
    let (target_uuid, _) = register_device_with_whitelists(&state, wl).await;

    let auth = basic_auth(&actor_uuid, &actor_token);
    let resp = http_request(&app, Method::DELETE, &format!("/devices/{}", target_uuid), Some(&auth), None).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

// ---------------------------------------------------------------------------
// message.from  (POST /messages)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn message_from_allowed() {
    let (ws_url, state) = setup().await;
    let (sender_uuid, sender_token) = register_device(&state).await;

    // Target allows sender in message.from
    let wl = open_with(|w| {
        w.message.from = vec![entry_for(&sender_uuid)];
    });
    let (target_uuid, target_token) = register_device_with_whitelists(&state, wl).await;

    // Connect target via WS so we can verify delivery
    let mut ws_target = connect_and_auth(&ws_url, &target_uuid, &target_token).await;

    // Send message via HTTP
    let (app, _) = (freshblu_server::build_router(state.clone()), ());
    let auth = basic_auth(&sender_uuid, &sender_token);
    let body = json!({
        "devices": [target_uuid],
        "payload": {"test": "allowed"}
    });
    let resp = http_request(&app, Method::POST, "/messages", Some(&auth), Some(body)).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Verify message was delivered
    let msg = recv_json(&mut ws_target).await.expect("should receive message");
    assert_eq!(msg["event"], "message");
    assert_eq!(msg["payload"]["test"], "allowed");
}

#[tokio::test]
async fn message_from_denied() {
    let (ws_url, state) = setup().await;
    let (sender_uuid, sender_token) = register_device(&state).await;

    // Target has empty message.from (nobody can send)
    let wl = locked_but_discoverable();
    let (target_uuid, target_token) = register_device_with_whitelists(&state, wl).await;

    // Connect target via WS
    let mut ws_target = connect_and_auth(&ws_url, &target_uuid, &target_token).await;

    let app = freshblu_server::build_router(state.clone());
    let auth = basic_auth(&sender_uuid, &sender_token);
    let body = json!({
        "devices": [target_uuid],
        "payload": {"test": "denied"}
    });
    let resp = http_request(&app, Method::POST, "/messages", Some(&auth), Some(body)).await;
    // API still returns 200 (sent: true) but silently drops the message
    assert_eq!(resp.status(), StatusCode::OK);

    // Target should NOT receive the message
    let msg = recv_json(&mut ws_target).await;
    assert!(msg.is_none(), "message should have been silently dropped");
}

#[tokio::test]
async fn message_from_wildcard() {
    let (ws_url, state) = setup().await;
    let (sender_uuid, sender_token) = register_device(&state).await;

    // Target allows everyone in message.from
    let wl = open_with(|w| {
        w.message.from = vec![wildcard_entry()];
    });
    let (target_uuid, target_token) = register_device_with_whitelists(&state, wl).await;

    let mut ws_target = connect_and_auth(&ws_url, &target_uuid, &target_token).await;

    let app = freshblu_server::build_router(state.clone());
    let auth = basic_auth(&sender_uuid, &sender_token);
    let body = json!({
        "devices": [target_uuid],
        "payload": {"test": "wildcard"}
    });
    let resp = http_request(&app, Method::POST, "/messages", Some(&auth), Some(body)).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let msg = recv_json(&mut ws_target).await.expect("should receive message");
    assert_eq!(msg["event"], "message");
}

#[tokio::test]
async fn message_from_self() {
    let (ws_url, state) = setup().await;

    // Device with locked message.from sends to itself (self always allowed)
    let wl = locked();
    let (uuid, token) = register_device_with_whitelists(&state, wl).await;

    let mut ws = connect_and_auth(&ws_url, &uuid, &token).await;

    let app = freshblu_server::build_router(state.clone());
    let auth = basic_auth(&uuid, &token);
    let body = json!({
        "devices": [uuid],
        "payload": {"test": "self"}
    });
    let resp = http_request(&app, Method::POST, "/messages", Some(&auth), Some(body)).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let msg = recv_json(&mut ws).await.expect("self-message should be delivered");
    assert_eq!(msg["event"], "message");
}

// ---------------------------------------------------------------------------
// x-meshblu-as header: discover.as
// ---------------------------------------------------------------------------

#[tokio::test]
async fn discover_as_allowed() {
    let (app, state) = setup_router().await;
    let (actor_uuid, actor_token) = register_device(&state).await;

    // Device B: has actor A in its discover.as list
    let wl = open_with(|w| {
        w.discover.r#as = vec![entry_for(&actor_uuid)];
    });
    let (b_uuid, _) = register_device_with_whitelists(&state, wl).await;

    // Device X: open, anyone can view
    let (x_uuid, _) = register_device(&state).await;

    // Actor A GETs /devices/X with x-meshblu-as: B
    let auth = basic_auth(&actor_uuid, &actor_token);
    let resp = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/devices/{}", x_uuid))
                .header("authorization", &auth)
                .header("x-meshblu-as", &b_uuid)
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn discover_as_denied() {
    let (app, state) = setup_router().await;
    let (actor_uuid, actor_token) = register_device(&state).await;

    // Device B: empty discover.as (nobody can impersonate)
    let wl = locked_but_discoverable();
    let (b_uuid, _) = register_device_with_whitelists(&state, wl).await;

    let (x_uuid, _) = register_device(&state).await;

    let auth = basic_auth(&actor_uuid, &actor_token);
    let resp = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/devices/{}", x_uuid))
                .header("authorization", &auth)
                .header("x-meshblu-as", &b_uuid)
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn discover_as_wildcard() {
    let (app, state) = setup_router().await;
    let (actor_uuid, actor_token) = register_device(&state).await;

    // Device B: wildcard discover.as
    let wl = open_with(|w| {
        w.discover.r#as = vec![wildcard_entry()];
    });
    let (b_uuid, _) = register_device_with_whitelists(&state, wl).await;

    let (x_uuid, _) = register_device(&state).await;

    let auth = basic_auth(&actor_uuid, &actor_token);
    let resp = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/devices/{}", x_uuid))
                .header("authorization", &auth)
                .header("x-meshblu-as", &b_uuid)
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

// ---------------------------------------------------------------------------
// x-meshblu-as header: configure.as
// ---------------------------------------------------------------------------

#[tokio::test]
async fn configure_as_allowed() {
    let (app, state) = setup_router().await;
    let (actor_uuid, actor_token) = register_device(&state).await;

    // Device B: A is in B's configure.as
    let wl = open_with(|w| {
        w.configure.r#as = vec![entry_for(&actor_uuid)];
    });
    let (b_uuid, _) = register_device_with_whitelists(&state, wl).await;

    // Target device X: B (the impersonated identity) must be in X's configure.update
    let wl_x = open_with(|w| {
        w.configure.update = vec![entry_for(&b_uuid)];
    });
    let (x_uuid, _) = register_device_with_whitelists(&state, wl_x).await;

    let auth = basic_auth(&actor_uuid, &actor_token);
    let resp = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("PUT")
                .uri(format!("/devices/{}", x_uuid))
                .header("authorization", &auth)
                .header("x-meshblu-as", &b_uuid)
                .header("content-type", "application/json")
                .body(axum::body::Body::from(r#"{"color":"blue"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn configure_as_denied() {
    let (app, state) = setup_router().await;
    let (actor_uuid, actor_token) = register_device(&state).await;

    // Device B: empty configure.as
    let wl = locked_but_discoverable();
    let (b_uuid, _) = register_device_with_whitelists(&state, wl).await;

    let (x_uuid, _) = register_device(&state).await;

    let auth = basic_auth(&actor_uuid, &actor_token);
    let resp = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("PUT")
                .uri(format!("/devices/{}", x_uuid))
                .header("authorization", &auth)
                .header("x-meshblu-as", &b_uuid)
                .header("content-type", "application/json")
                .body(axum::body::Body::from(r#"{"color":"blue"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn configure_as_wildcard() {
    let (app, state) = setup_router().await;
    let (actor_uuid, actor_token) = register_device(&state).await;

    // Device B: wildcard configure.as
    let wl = open_with(|w| {
        w.configure.r#as = vec![wildcard_entry()];
    });
    let (b_uuid, _) = register_device_with_whitelists(&state, wl).await;

    // Target X: B in configure.update
    let wl_x = open_with(|w| {
        w.configure.update = vec![entry_for(&b_uuid)];
    });
    let (x_uuid, _) = register_device_with_whitelists(&state, wl_x).await;

    let auth = basic_auth(&actor_uuid, &actor_token);
    let resp = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("PUT")
                .uri(format!("/devices/{}", x_uuid))
                .header("authorization", &auth)
                .header("x-meshblu-as", &b_uuid)
                .header("content-type", "application/json")
                .body(axum::body::Body::from(r#"{"color":"blue"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

// ---------------------------------------------------------------------------
// x-meshblu-as header: message.as
// ---------------------------------------------------------------------------

#[tokio::test]
async fn message_as_allowed() {
    let (app, state) = setup_router().await;
    let (actor_uuid, actor_token) = register_device(&state).await;

    // Device B: A in B's message.as
    let wl = open_with(|w| {
        w.message.r#as = vec![entry_for(&actor_uuid)];
    });
    let (b_uuid, _) = register_device_with_whitelists(&state, wl).await;

    // Target device (open, anyone can message)
    let (x_uuid, _) = register_device(&state).await;

    let auth = basic_auth(&actor_uuid, &actor_token);
    let body = json!({
        "devices": [x_uuid],
        "payload": {"test": "as_allowed"}
    });
    let resp = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/messages")
                .header("authorization", &auth)
                .header("x-meshblu-as", &b_uuid)
                .header("content-type", "application/json")
                .body(axum::body::Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn message_as_denied() {
    let (app, state) = setup_router().await;
    let (actor_uuid, actor_token) = register_device(&state).await;

    // Device B: empty message.as
    let wl = locked_but_discoverable();
    let (b_uuid, _) = register_device_with_whitelists(&state, wl).await;

    let (x_uuid, _) = register_device(&state).await;

    let auth = basic_auth(&actor_uuid, &actor_token);
    let body = json!({
        "devices": [x_uuid],
        "payload": {"test": "as_denied"}
    });
    let resp = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/messages")
                .header("authorization", &auth)
                .header("x-meshblu-as", &b_uuid)
                .header("content-type", "application/json")
                .body(axum::body::Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn message_as_wildcard() {
    let (app, state) = setup_router().await;
    let (actor_uuid, actor_token) = register_device(&state).await;

    // Device B: wildcard message.as
    let wl = open_with(|w| {
        w.message.r#as = vec![wildcard_entry()];
    });
    let (b_uuid, _) = register_device_with_whitelists(&state, wl).await;

    let (x_uuid, _) = register_device(&state).await;

    let auth = basic_auth(&actor_uuid, &actor_token);
    let body = json!({
        "devices": [x_uuid],
        "payload": {"test": "as_wildcard"}
    });
    let resp = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/messages")
                .header("authorization", &auth)
                .header("x-meshblu-as", &b_uuid)
                .header("content-type", "application/json")
                .body(axum::body::Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

// ===========================================================================
// 3.2  WS Subscribe Permission Tests — Denied (private device)
// ===========================================================================

/// Helper: connect via WS, try to subscribe to a private device's events
/// of the given type, and assert an error event is returned.
async fn assert_ws_subscribe_denied(sub_type: &str) {
    let (ws_url, state) = setup().await;

    let (actor_uuid, actor_token) = register_device(&state).await;
    let mut ws = connect_and_auth(&ws_url, &actor_uuid, &actor_token).await;

    // Register a fully-private device
    let (private_uuid, _) = register_private_device(&state).await;

    let subscribe = json!({
        "event": "subscribe",
        "emitterUuid": private_uuid,
        "type": sub_type,
    });
    ws.send(Message::Text(subscribe.to_string())).await.unwrap();

    let resp = recv_json(&mut ws).await.expect("expected error response for denied subscribe");
    assert_eq!(resp["event"], "error", "subscribe to {} on private device should be denied", sub_type);
}

#[tokio::test]
async fn ws_subscribe_broadcast_sent_denied() {
    assert_ws_subscribe_denied("broadcast.sent").await;
}

#[tokio::test]
async fn ws_subscribe_broadcast_received_denied() {
    assert_ws_subscribe_denied("broadcast.received").await;
}

#[tokio::test]
async fn ws_subscribe_message_sent_denied() {
    assert_ws_subscribe_denied("message.sent").await;
}

#[tokio::test]
async fn ws_subscribe_message_received_denied() {
    assert_ws_subscribe_denied("message.received").await;
}

#[tokio::test]
async fn ws_subscribe_configure_sent_denied() {
    assert_ws_subscribe_denied("configure.sent").await;
}

#[tokio::test]
async fn ws_subscribe_configure_received_denied() {
    assert_ws_subscribe_denied("configure.received").await;
}

#[tokio::test]
async fn ws_subscribe_unregister_sent_denied() {
    assert_ws_subscribe_denied("unregister.sent").await;
}

#[tokio::test]
async fn ws_subscribe_unregister_received_denied() {
    assert_ws_subscribe_denied("unregister.received").await;
}

// ---------------------------------------------------------------------------
// WS Subscribe — Allowed (open device)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn ws_subscribe_broadcast_sent_allowed() {
    let (ws_url, state) = setup().await;

    let (actor_uuid, actor_token) = register_device(&state).await;
    let mut ws = connect_and_auth(&ws_url, &actor_uuid, &actor_token).await;

    // Register an open device (default whitelists allow everything)
    let (open_uuid, _) = register_device(&state).await;

    let subscribe = json!({
        "event": "subscribe",
        "emitterUuid": open_uuid,
        "type": "broadcast.sent",
    });
    ws.send(Message::Text(subscribe.to_string())).await.unwrap();

    // Should NOT receive an error (the subscribe succeeds silently).
    // Send a ping to flush the channel and confirm no error arrived.
    let ping = json!({"event": "ping"});
    ws.send(Message::Text(ping.to_string())).await.unwrap();

    let resp = recv_json(&mut ws).await.expect("expected pong (no error before it)");
    assert_eq!(resp["event"], "pong", "should get pong, not an error — subscribe should have succeeded");
}

// ---------------------------------------------------------------------------
// x-meshblu-as header: broadcast.as
// ---------------------------------------------------------------------------

#[tokio::test]
async fn broadcast_as_allowed() {
    let (ws_url, state) = setup().await;
    let (actor_uuid, actor_token) = register_device(&state).await;

    // Device B: A in B's broadcast.as AND message.as
    let wl = open_with(|w| {
        w.broadcast.r#as = vec![entry_for(&actor_uuid)];
        w.message.r#as = vec![entry_for(&actor_uuid)];
    });
    let (b_uuid, _) = register_device_with_whitelists(&state, wl).await;

    // S subscribes to B's broadcast.sent
    let (s_uuid, s_token) = register_device(&state).await;
    let b_parsed: uuid::Uuid = b_uuid.parse().unwrap();
    let s_parsed: uuid::Uuid = s_uuid.parse().unwrap();
    let params = freshblu_core::subscription::CreateSubscriptionParams {
        emitter_uuid: b_parsed,
        subscriber_uuid: s_parsed,
        subscription_type: freshblu_core::subscription::SubscriptionType::BroadcastSent,
    };
    state.store.create_subscription(&params).await.unwrap();

    // S connects via WS
    let mut ws_s = connect_and_auth(&ws_url, &s_uuid, &s_token).await;

    // A broadcasts as B via HTTP
    let app = freshblu_server::build_router(state.clone());
    let auth = basic_auth(&actor_uuid, &actor_token);
    let body = json!({
        "devices": ["*"],
        "payload": {"test": "broadcast_as_allowed"}
    });
    let resp = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/messages")
                .header("authorization", &auth)
                .header("x-meshblu-as", &b_uuid)
                .header("content-type", "application/json")
                .body(axum::body::Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // S should receive the broadcast
    let msg = recv_json(&mut ws_s).await.expect("S should receive broadcast sent as B");
    assert_eq!(msg["event"], "broadcast");
    assert_eq!(msg["payload"]["test"], "broadcast_as_allowed");
}

#[tokio::test]
async fn broadcast_as_denied() {
    let (app, state) = setup_router().await;
    let (actor_uuid, actor_token) = register_device(&state).await;

    // Device B: has message.as but NOT broadcast.as for actor
    let wl = open_with(|w| {
        w.message.r#as = vec![entry_for(&actor_uuid)];
        w.broadcast.r#as = vec![]; // deny broadcast.as
    });
    let (b_uuid, _) = register_device_with_whitelists(&state, wl).await;

    let auth = basic_auth(&actor_uuid, &actor_token);
    let body = json!({
        "devices": ["*"],
        "payload": {"test": "broadcast_as_denied"}
    });
    let resp = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/messages")
                .header("authorization", &auth)
                .header("x-meshblu-as", &b_uuid)
                .header("content-type", "application/json")
                .body(axum::body::Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn broadcast_as_wildcard() {
    let (ws_url, state) = setup().await;
    let (actor_uuid, actor_token) = register_device(&state).await;

    // Device B: wildcard broadcast.as AND message.as
    let wl = open_with(|w| {
        w.broadcast.r#as = vec![wildcard_entry()];
        w.message.r#as = vec![wildcard_entry()];
    });
    let (b_uuid, _) = register_device_with_whitelists(&state, wl).await;

    // S subscribes to B's broadcast.sent
    let (s_uuid, s_token) = register_device(&state).await;
    let b_parsed: uuid::Uuid = b_uuid.parse().unwrap();
    let s_parsed: uuid::Uuid = s_uuid.parse().unwrap();
    let params = freshblu_core::subscription::CreateSubscriptionParams {
        emitter_uuid: b_parsed,
        subscriber_uuid: s_parsed,
        subscription_type: freshblu_core::subscription::SubscriptionType::BroadcastSent,
    };
    state.store.create_subscription(&params).await.unwrap();

    let mut ws_s = connect_and_auth(&ws_url, &s_uuid, &s_token).await;

    let app = freshblu_server::build_router(state.clone());
    let auth = basic_auth(&actor_uuid, &actor_token);
    let body = json!({
        "devices": ["*"],
        "payload": {"test": "broadcast_as_wildcard"}
    });
    let resp = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/messages")
                .header("authorization", &auth)
                .header("x-meshblu-as", &b_uuid)
                .header("content-type", "application/json")
                .body(axum::body::Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let msg = recv_json(&mut ws_s).await.expect("S should receive broadcast sent as B (wildcard)");
    assert_eq!(msg["event"], "broadcast");
    assert_eq!(msg["payload"]["test"], "broadcast_as_wildcard");
}
