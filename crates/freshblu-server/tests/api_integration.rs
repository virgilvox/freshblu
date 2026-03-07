use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use freshblu_server::{build_router, AppState, ServerConfig};
use freshblu_store::sqlite::SqliteStore;
use serde_json::{json, Value};
use std::sync::Arc;
use tower::ServiceExt;

async fn setup() -> axum::Router {
    let store: freshblu_store::DynStore =
        Arc::new(SqliteStore::new("sqlite::memory:").await.unwrap());
    let bus: freshblu_server::DynBus = Arc::new(freshblu_server::local_bus::LocalBus::new());
    let state = AppState {
        store,
        bus,
        config: ServerConfig::default(),
    };
    build_router(state)
}

fn basic_auth(uuid: &str, token: &str) -> String {
    use base64::Engine;
    let encoded = base64::engine::general_purpose::STANDARD.encode(format!("{}:{}", uuid, token));
    format!("Basic {}", encoded)
}

async fn register_device(app: &axum::Router) -> (String, String) {
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/devices")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"type":"test"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let v: Value = serde_json::from_slice(&body).unwrap();
    (
        v["uuid"].as_str().unwrap().to_string(),
        v["token"].as_str().unwrap().to_string(),
    )
}

// ---------------------------------------------------------------------------
// Registration & Auth
// ---------------------------------------------------------------------------

#[tokio::test]
async fn register_device_returns_uuid_and_token() {
    let app = setup().await;
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/devices")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"type":"test"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let v: Value = serde_json::from_slice(&body).unwrap();
    assert!(v["uuid"].is_string(), "uuid should be present");
    assert!(v["token"].is_string(), "token should be present");
}

#[tokio::test]
async fn register_device_with_type() {
    let app = setup().await;
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/devices")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"type":"sensor"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let v: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["type"], "sensor");
}

#[tokio::test]
async fn whoami_returns_device() {
    let app = setup().await;
    let (uuid, token) = register_device(&app).await;

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/whoami")
                .header("authorization", basic_auth(&uuid, &token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let v: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["uuid"].as_str().unwrap(), uuid);
}

#[tokio::test]
async fn whoami_unauthorized() {
    let app = setup().await;
    let resp = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/whoami")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn whoami_wrong_token() {
    let app = setup().await;
    let (uuid, _token) = register_device(&app).await;

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/whoami")
                .header("authorization", basic_auth(&uuid, "wrong-token"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn authenticate_endpoint() {
    let app = setup().await;
    let (uuid, token) = register_device(&app).await;

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/authenticate")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({"uuid": uuid, "token": token})).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
}

// ---------------------------------------------------------------------------
// Device CRUD
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_device_by_uuid() {
    let app = setup().await;
    let (uuid, token) = register_device(&app).await;

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/devices/{}", uuid))
                .header("authorization", basic_auth(&uuid, &token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let v: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["uuid"].as_str().unwrap(), uuid);
}

#[tokio::test]
async fn get_device_not_found() {
    let app = setup().await;
    let (uuid, token) = register_device(&app).await;
    let random_uuid = uuid::Uuid::new_v4();

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/devices/{}", random_uuid))
                .header("authorization", basic_auth(&uuid, &token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn update_device() {
    let app = setup().await;
    let (uuid, token) = register_device(&app).await;

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/devices/{}", uuid))
                .header("authorization", basic_auth(&uuid, &token))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"color":"blue"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let v: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["color"], "blue");
}

#[tokio::test]
async fn unregister_device() {
    let app = setup().await;
    let (uuid, token) = register_device(&app).await;

    // DELETE the device
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/devices/{}", uuid))
                .header("authorization", basic_auth(&uuid, &token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    // Register a new device so we have valid auth to query with
    let (uuid2, token2) = register_device(&app).await;

    // GET the deleted device should return 404
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/devices/{}", uuid))
                .header("authorization", basic_auth(&uuid2, &token2))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

// ---------------------------------------------------------------------------
// Messaging
// ---------------------------------------------------------------------------

#[tokio::test]
async fn send_message_to_device() {
    let app = setup().await;
    let (uuid_a, token_a) = register_device(&app).await;
    let (uuid_b, _token_b) = register_device(&app).await;

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/messages")
                .header("authorization", basic_auth(&uuid_a, &token_a))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "devices": [uuid_b],
                        "payload": {"hello": "world"}
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
}

// ---------------------------------------------------------------------------
// Subscriptions
// ---------------------------------------------------------------------------

#[tokio::test]
async fn create_subscription() {
    let app = setup().await;
    let (uuid_a, token_a) = register_device(&app).await;
    let (uuid_b, _token_b) = register_device(&app).await;

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/devices/{}/subscriptions", uuid_a))
                .header("authorization", basic_auth(&uuid_a, &token_a))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "emitterUuid": uuid_b,
                        "subscriberUuid": uuid_a,
                        "type": "broadcast-sent"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn list_subscriptions() {
    let app = setup().await;
    let (uuid_a, token_a) = register_device(&app).await;
    let (uuid_b, _token_b) = register_device(&app).await;

    // Create a subscription first
    let _resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/devices/{}/subscriptions", uuid_a))
                .header("authorization", basic_auth(&uuid_a, &token_a))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "emitterUuid": uuid_b,
                        "subscriberUuid": uuid_a,
                        "type": "broadcast-sent"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // List subscriptions
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/devices/{}/subscriptions", uuid_a))
                .header("authorization", basic_auth(&uuid_a, &token_a))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let v: Value = serde_json::from_slice(&body).unwrap();
    let subs = v.as_array().expect("should be an array");
    assert!(!subs.is_empty(), "subscriptions list should not be empty");
    assert_eq!(subs[0]["emitterUuid"].as_str().unwrap(), uuid_b);
}

#[tokio::test]
async fn delete_subscription() {
    let app = setup().await;
    let (uuid_a, token_a) = register_device(&app).await;
    let (uuid_b, _token_b) = register_device(&app).await;

    // Create a subscription
    let _resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/devices/{}/subscriptions", uuid_a))
                .header("authorization", basic_auth(&uuid_a, &token_a))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "emitterUuid": uuid_b,
                        "subscriberUuid": uuid_a,
                        "type": "broadcast-sent"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Delete the subscription
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!(
                    "/devices/{}/subscriptions/{}/broadcast-sent",
                    uuid_a, uuid_b
                ))
                .header("authorization", basic_auth(&uuid_a, &token_a))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
}

// ---------------------------------------------------------------------------
// Tokens
// ---------------------------------------------------------------------------

#[tokio::test]
async fn generate_additional_token() {
    let app = setup().await;
    let (uuid, token) = register_device(&app).await;

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/devices/{}/tokens", uuid))
                .header("authorization", basic_auth(&uuid, &token))
                .header("content-type", "application/json")
                .body(Body::from("null"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let v: Value = serde_json::from_slice(&body).unwrap();
    assert!(v["token"].is_string(), "new token should be returned");
    assert_ne!(
        v["token"].as_str().unwrap(),
        token,
        "new token should differ from original"
    );
}

#[tokio::test]
async fn revoke_token() {
    let app = setup().await;
    let (uuid, token) = register_device(&app).await;

    // Generate a new token
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/devices/{}/tokens", uuid))
                .header("authorization", basic_auth(&uuid, &token))
                .header("content-type", "application/json")
                .body(Body::from("null"))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let v: Value = serde_json::from_slice(&body).unwrap();
    let new_token = v["token"].as_str().unwrap();

    // Revoke the new token
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/devices/{}/tokens/{}", uuid, new_token))
                .header("authorization", basic_auth(&uuid, &token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
}

// ---------------------------------------------------------------------------
// Status
// ---------------------------------------------------------------------------

#[tokio::test]
async fn status_endpoint() {
    let app = setup().await;
    let resp = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let v: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["meshblu"], true);
}

#[tokio::test]
async fn v2_routes_work() {
    let app = setup().await;
    let (uuid, token) = register_device(&app).await;

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v2/whoami")
                .header("authorization", basic_auth(&uuid, &token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
}

// ---------------------------------------------------------------------------
// Security: Permission checks
// ---------------------------------------------------------------------------

async fn setup_with_config(config: ServerConfig) -> axum::Router {
    let store: freshblu_store::DynStore =
        Arc::new(SqliteStore::new("sqlite::memory:").await.unwrap());
    let bus: freshblu_server::DynBus = Arc::new(freshblu_server::local_bus::LocalBus::new());
    let state = AppState { store, bus, config };
    build_router(state)
}

/// Register a device with private (locked-down) whitelists
async fn register_private_device(app: &axum::Router) -> (String, String) {
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/devices")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"type":"private","meshblu":{"whitelists":{"discover":{"view":[],"as":[]},"configure":{"update":[],"sent":[],"received":[],"as":[]},"message":{"from":[],"sent":[],"received":[],"as":[]},"broadcast":{"sent":[],"received":[],"as":[]}}}}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let v: Value = serde_json::from_slice(&body).unwrap();
    (
        v["uuid"].as_str().unwrap().to_string(),
        v["token"].as_str().unwrap().to_string(),
    )
}

#[tokio::test]
async fn subscribe_permission_denied() {
    let app = setup().await;
    let (uuid_a, token_a) = register_device(&app).await;
    let (uuid_b, _token_b) = register_private_device(&app).await;

    // Device A tries to subscribe to private device B's broadcasts
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/devices/{}/subscriptions", uuid_a))
                .header("authorization", basic_auth(&uuid_a, &token_a))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_string(&json!({
                        "emitterUuid": uuid_b,
                        "subscriberUuid": uuid_a,
                        "type": "broadcast-sent"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn as_header_permission_denied() {
    let app = setup().await;
    let (uuid_a, token_a) = register_device(&app).await;
    let (uuid_b, _token_b) = register_private_device(&app).await;

    // Device A tries to act as private device B (no as permission)
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/devices/{}", uuid_a))
                .header("authorization", basic_auth(&uuid_a, &token_a))
                .header("x-meshblu-as", &uuid_b)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn register_closed_registration() {
    let mut config = ServerConfig::default();
    config.open_registration = false;
    let app = setup_with_config(config).await;

    // Try to register without auth — should be denied
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/devices")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"type":"test"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}
