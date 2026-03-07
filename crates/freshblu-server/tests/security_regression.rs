//! Security regression tests for FreshBlu server.
//!
//! These tests verify that permission enforcement, token expiration,
//! and the x-meshblu-as header work correctly across HTTP and WS transports.

mod helpers;

use axum::http::{Method, StatusCode};
use freshblu_core::device::WhitelistEntry;
use freshblu_core::permissions::*;
use futures::SinkExt;
use helpers::*;
use serde_json::{json, Value};
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// 1. WS message permission enforced
// ---------------------------------------------------------------------------

/// Device B is private. A sends a message to B via WS and B receives nothing.
/// Then create a new device B2 that explicitly whitelists A in message.from,
/// resend, and B2 receives the message.
#[tokio::test]
async fn ws_message_permission_enforced() {
    let (ws_url, state) = setup().await;

    // Device A: open whitelists (public)
    let (uuid_a, token_a) = register_device(&state).await;

    // Device B: private (empty whitelists) -- nobody can message it
    let (uuid_b, token_b) = register_private_device(&state).await;

    // Connect both via WS
    let mut ws_a = connect_and_auth(&ws_url, &uuid_a, &token_a).await;
    let mut ws_b = connect_and_auth(&ws_url, &uuid_b, &token_b).await;

    // A sends a message targeting B
    let msg = json!({
        "event": "message",
        "devices": [uuid_b],
        "payload": {"test": 1}
    });
    ws_a.send(Message::Text(msg.to_string())).await.unwrap();

    // B should NOT receive anything (permission denied silently)
    let received = recv_json(&mut ws_b).await;
    assert!(
        received.is_none(),
        "private device B should not receive messages from unauthorized A"
    );

    // Now create device B2 that whitelists A in message.from
    let a_uuid_parsed: Uuid = uuid_a.parse().unwrap();
    let mut whitelists = Whitelists::default();
    whitelists.message.from = vec![WhitelistEntry::for_uuid(&a_uuid_parsed)];
    // Also allow self to see messages (sent/received open)
    whitelists.message.sent = vec![WhitelistEntry::wildcard()];
    whitelists.message.received = vec![WhitelistEntry::wildcard()];
    // Open discover so the test doesn't trip on other checks
    whitelists.discover.view = vec![WhitelistEntry::wildcard()];

    let (uuid_b2, token_b2) = register_device_with_whitelists(&state, whitelists).await;
    let mut ws_b2 = connect_and_auth(&ws_url, &uuid_b2, &token_b2).await;

    // A sends message to B2
    let msg2 = json!({
        "event": "message",
        "devices": [uuid_b2],
        "payload": {"test": 2}
    });
    ws_a.send(Message::Text(msg2.to_string())).await.unwrap();

    // B2 should receive the message
    let received = recv_json(&mut ws_b2).await;
    assert!(
        received.is_some(),
        "whitelisted device B2 should receive the message"
    );
    let received = received.unwrap();
    assert_eq!(received["event"], "message");
    assert_eq!(received["payload"]["test"], 2);
}

// ---------------------------------------------------------------------------
// 2. WS update only affects self
// ---------------------------------------------------------------------------

/// A sends an update via WS. Verify A's device has the property, B's does not.
#[tokio::test]
async fn ws_update_only_affects_self() {
    let (ws_url, state) = setup().await;

    let (uuid_a, token_a) = register_device(&state).await;
    let (uuid_b, _token_b) = register_device(&state).await;

    let mut ws_a = connect_and_auth(&ws_url, &uuid_a, &token_a).await;

    // A sends an update setting color=red
    let update = json!({
        "event": "update",
        "color": "red"
    });
    ws_a.send(Message::Text(update.to_string())).await.unwrap();

    // Give the server a moment to process the update
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // Check A's device in the store
    let a_uuid: Uuid = uuid_a.parse().unwrap();
    let device_a = state.store.get_device(&a_uuid).await.unwrap().unwrap();
    assert_eq!(
        device_a.properties.get("color"),
        Some(&json!("red")),
        "device A should have color=red after update"
    );

    // Check B's device in the store
    let b_uuid: Uuid = uuid_b.parse().unwrap();
    let device_b = state.store.get_device(&b_uuid).await.unwrap().unwrap();
    assert!(
        device_b.properties.get("color").is_none(),
        "device B should NOT have a color property"
    );
}

// ---------------------------------------------------------------------------
// 3. Token expiration enforced on HTTP and WS
// ---------------------------------------------------------------------------

/// Generate a token with expires_on=0 (already expired).
/// HTTP /authenticate with expired token should fail.
/// WS identity with expired token should return notReady.
#[tokio::test]
async fn token_expiration_enforced_http_and_ws() {
    let (ws_url, state) = setup().await;

    // Register a device (gets a valid token)
    let (uuid_str, _valid_token) = register_device(&state).await;
    let device_uuid: Uuid = uuid_str.parse().unwrap();

    // Generate a token that is already expired (expires_on = 0 = epoch)
    use freshblu_core::token::GenerateTokenOptions;
    let opts = GenerateTokenOptions {
        expires_on: Some(0),
        tag: None,
    };
    let (_record, expired_token) = state
        .store
        .generate_token(&device_uuid, opts)
        .await
        .unwrap();

    // HTTP: POST /authenticate with expired token should fail.
    // Build a router that shares the same store so the expired token is visible.
    let app = freshblu_server::build_router(state.clone());
    let resp = http_request(
        &app,
        Method::POST,
        "/authenticate",
        None,
        Some(json!({
            "uuid": uuid_str,
            "token": expired_token
        })),
    )
    .await;
    assert_ne!(
        resp.status(),
        StatusCode::OK,
        "expired token should NOT authenticate successfully via HTTP"
    );

    // WS: identity with expired token should return notReady
    let (mut ws, _) = tokio_tungstenite::connect_async(&ws_url)
        .await
        .expect("failed to connect");

    let identity = json!({
        "event": "identity",
        "uuid": uuid_str,
        "token": expired_token,
    });
    ws.send(Message::Text(identity.to_string())).await.unwrap();

    let resp = recv_json(&mut ws)
        .await
        .expect("expected notReady response");
    assert_eq!(
        resp["event"], "notReady",
        "expired token should yield notReady on WS"
    );
}

// ---------------------------------------------------------------------------
// 4. Message send reports sent:true even when all denied
// ---------------------------------------------------------------------------

/// A messages private B via HTTP. The response says {"sent": true}
/// even though the message was silently denied (documenting current behavior).
#[tokio::test]
async fn message_send_reports_when_all_denied() {
    let (app, state) = setup_router().await;

    // A: open device
    let (uuid_a, token_a) = register_device(&state).await;
    // B: private device
    let (uuid_b, _token_b) = register_private_device(&state).await;

    let auth = basic_auth(&uuid_a, &token_a);
    let resp = http_request(
        &app,
        Method::POST,
        "/messages",
        Some(&auth),
        Some(json!({
            "devices": [uuid_b],
            "payload": {"hello": "world"}
        })),
    )
    .await;

    assert_eq!(resp.status(), StatusCode::OK);
    let body = response_json(resp).await;
    assert_eq!(
        body["sent"], true,
        "current behavior: server returns sent:true even when message was silently denied"
    );
}

// ---------------------------------------------------------------------------
// 5. x-meshblu-as denied on all endpoints for private device
// ---------------------------------------------------------------------------

/// Device B is private (empty as-whitelists). For each major endpoint,
/// sending with x-meshblu-as: B should return 403.
#[tokio::test]
async fn x_meshblu_as_denied_all_endpoints() {
    let (app, state) = setup_router().await;

    // A: open device (the authenticated caller)
    let (uuid_a, token_a) = register_device(&state).await;
    // B: private device (empty whitelists, no as-permissions)
    let (uuid_b, _token_b) = register_private_device(&state).await;

    let auth = basic_auth(&uuid_a, &token_a);

    // Helper to make a request with x-meshblu-as header
    let request_with_as = |method: Method, uri: String, body: Option<Value>| {
        let auth = auth.clone();
        let uuid_b = uuid_b.clone();
        let app = app.clone();
        async move {
            use axum::body::Body;
            use axum::http::Request;
            use tower::ServiceExt;

            let mut builder = Request::builder()
                .method(method)
                .uri(&uri)
                .header("authorization", &auth)
                .header("x-meshblu-as", &uuid_b);

            if body.is_some() {
                builder = builder.header("content-type", "application/json");
            }

            let req_body = match body {
                Some(v) => Body::from(serde_json::to_string(&v).unwrap()),
                None => Body::empty(),
            };

            app.oneshot(builder.body(req_body).unwrap()).await.unwrap()
        }
    };

    // GET /devices/B
    let resp = request_with_as(Method::GET, format!("/devices/{}", uuid_b), None).await;
    assert_eq!(
        resp.status(),
        StatusCode::FORBIDDEN,
        "GET /devices/B with x-meshblu-as should be 403"
    );

    // PUT /devices/B
    let resp = request_with_as(
        Method::PUT,
        format!("/devices/{}", uuid_b),
        Some(json!({"color": "blue"})),
    )
    .await;
    assert_eq!(
        resp.status(),
        StatusCode::FORBIDDEN,
        "PUT /devices/B with x-meshblu-as should be 403"
    );

    // DELETE /devices/B
    let resp = request_with_as(Method::DELETE, format!("/devices/{}", uuid_b), None).await;
    assert_eq!(
        resp.status(),
        StatusCode::FORBIDDEN,
        "DELETE /devices/B with x-meshblu-as should be 403"
    );

    // POST /messages with x-meshblu-as: B
    let resp = request_with_as(
        Method::POST,
        "/messages".to_string(),
        Some(json!({
            "devices": [uuid_a],
            "payload": {"test": true}
        })),
    )
    .await;
    assert_eq!(
        resp.status(),
        StatusCode::FORBIDDEN,
        "POST /messages with x-meshblu-as should be 403"
    );

    // POST /devices/search with x-meshblu-as: B
    let resp = request_with_as(Method::POST, "/devices/search".to_string(), Some(json!({}))).await;
    assert_eq!(
        resp.status(),
        StatusCode::FORBIDDEN,
        "POST /devices/search with x-meshblu-as should be 403"
    );

    // POST /devices/B/subscriptions with x-meshblu-as: B
    let resp = request_with_as(
        Method::POST,
        format!("/devices/{}/subscriptions", uuid_b),
        Some(json!({
            "emitterUuid": uuid_a,
            "subscriberUuid": uuid_b,
            "type": "broadcast-sent"
        })),
    )
    .await;
    assert_eq!(
        resp.status(),
        StatusCode::FORBIDDEN,
        "POST /devices/B/subscriptions with x-meshblu-as should be 403"
    );
}

// ---------------------------------------------------------------------------
// 6. NATS bus ready/notReady (covered by inline tests)
// ---------------------------------------------------------------------------

// Test 6 (nats_bus_ready_notready_not_unregister) is intentionally omitted here.
// The NatsBus connect/disconnect and event routing logic is already covered by
// unit tests inside crates/freshblu-server/src/nats_bus.rs.

// ---------------------------------------------------------------------------
// 7. Metrics increment on WS connect
// ---------------------------------------------------------------------------

/// Connect a device via WS, verify the freshblu_ws_connections gauge is > 0.
/// Disconnect, wait, verify it goes back to 0.
#[tokio::test]
async fn metrics_increment_on_ws_connect() {
    use freshblu_server::metrics::WS_CONNECTIONS;

    let (ws_url, state) = setup().await;
    let (uuid, token) = register_device(&state).await;

    let baseline = WS_CONNECTIONS.get();

    // Connect via WS
    let ws = connect_and_auth(&ws_url, &uuid, &token).await;

    // The gauge should have incremented
    let after_connect = WS_CONNECTIONS.get();
    assert!(
        after_connect > baseline,
        "WS_CONNECTIONS should increase after connect (was {}, now {})",
        baseline,
        after_connect
    );

    // Drop the WS connection
    drop(ws);

    // Wait for the server to notice the disconnect and decrement
    let mut went_back = false;
    for _ in 0..20 {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        if WS_CONNECTIONS.get() <= baseline {
            went_back = true;
            break;
        }
    }
    assert!(
        went_back,
        "WS_CONNECTIONS should return to baseline ({}) after disconnect, got {}",
        baseline,
        WS_CONNECTIONS.get()
    );
}
