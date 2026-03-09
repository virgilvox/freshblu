//! Integration tests for the CLI embedded server.
//!
//! These tests spin up the real server (same code path as `freshblu server`)
//! and hit it with HTTP requests to verify end-to-end behavior.
//!
//! Requires: `cargo test -p freshblu-cli --features server`

#![cfg(feature = "server")]

use std::sync::Arc;

use freshblu_server::{build_router, AppState, DynBus, RateLimiter, ServerConfig, WebhookExecutor};
use freshblu_store::{sqlite::SqliteStore, DynStore};
use serde_json::{json, Value};

/// Spin up an in-memory server on a random port, return base URL + port.
async fn start_server() -> String {
    let store: DynStore = Arc::new(SqliteStore::new("sqlite::memory:").await.unwrap());
    let bus: DynBus = Arc::new(freshblu_server::local_bus::LocalBus::new());
    let config = ServerConfig::default();
    let rate_limiter = RateLimiter::new(config.rate_limit, config.rate_window);
    let webhook_executor = Arc::new(WebhookExecutor::new(store.clone(), bus.clone()));

    let state = AppState {
        store,
        bus,
        config,
        rate_limiter,
        webhook_executor,
    };

    let router = build_router(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });

    format!("http://127.0.0.1:{}", addr.port())
}

fn basic_auth(uuid: &str, token: &str) -> String {
    use base64::Engine;
    let encoded =
        base64::engine::general_purpose::STANDARD.encode(format!("{}:{}", uuid, token));
    format!("Basic {}", encoded)
}

#[tokio::test]
async fn status_returns_meshblu_true() {
    let base = start_server().await;
    let client = reqwest::Client::new();

    let resp: Value = client
        .get(format!("{}/status", base))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(resp["meshblu"], true);
}

#[tokio::test]
async fn register_returns_uuid_and_token() {
    let base = start_server().await;
    let client = reqwest::Client::new();

    let resp: Value = client
        .post(format!("{}/devices", base))
        .json(&json!({"type": "test-sensor"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert!(resp["uuid"].is_string());
    assert!(resp["token"].is_string());
    assert_eq!(resp["type"], "test-sensor");
}

#[tokio::test]
async fn whoami_after_register() {
    let base = start_server().await;
    let client = reqwest::Client::new();

    // Register
    let reg: Value = client
        .post(format!("{}/devices", base))
        .json(&json!({}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let uuid = reg["uuid"].as_str().unwrap();
    let token = reg["token"].as_str().unwrap();

    // Whoami
    let whoami: Value = client
        .get(format!("{}/whoami", base))
        .header("Authorization", basic_auth(uuid, token))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(whoami["uuid"], uuid);
}

#[tokio::test]
async fn get_device_by_uuid() {
    let base = start_server().await;
    let client = reqwest::Client::new();

    let reg: Value = client
        .post(format!("{}/devices", base))
        .json(&json!({"type": "beacon"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let uuid = reg["uuid"].as_str().unwrap();
    let token = reg["token"].as_str().unwrap();

    let device: Value = client
        .get(format!("{}/devices/{}", base, uuid))
        .header("Authorization", basic_auth(uuid, token))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(device["uuid"], uuid);
    assert_eq!(device["type"], "beacon");
}

#[tokio::test]
async fn update_device_properties() {
    let base = start_server().await;
    let client = reqwest::Client::new();

    let reg: Value = client
        .post(format!("{}/devices", base))
        .json(&json!({}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let uuid = reg["uuid"].as_str().unwrap();
    let token = reg["token"].as_str().unwrap();
    let auth = basic_auth(uuid, token);

    let updated: Value = client
        .put(format!("{}/devices/{}", base, uuid))
        .header("Authorization", &auth)
        .json(&json!({"color": "blue", "temp": 22}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(updated["color"], "blue");
    assert_eq!(updated["temp"], 22);
}

#[tokio::test]
async fn unregister_device() {
    let base = start_server().await;
    let client = reqwest::Client::new();

    let reg: Value = client
        .post(format!("{}/devices", base))
        .json(&json!({}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let uuid = reg["uuid"].as_str().unwrap();
    let token = reg["token"].as_str().unwrap();
    let auth = basic_auth(uuid, token);

    // Delete
    let status = client
        .delete(format!("{}/devices/{}", base, uuid))
        .header("Authorization", &auth)
        .send()
        .await
        .unwrap()
        .status();

    assert!(status.is_success());

    // Whoami should fail now
    let resp = client
        .get(format!("{}/whoami", base))
        .header("Authorization", &auth)
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn search_devices() {
    let base = start_server().await;
    let client = reqwest::Client::new();

    // Register two devices with different types
    let reg1: Value = client
        .post(format!("{}/devices", base))
        .json(&json!({"type": "sensor"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let _reg2: Value = client
        .post(format!("{}/devices", base))
        .json(&json!({"type": "actuator"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let uuid1 = reg1["uuid"].as_str().unwrap();
    let token1 = reg1["token"].as_str().unwrap();

    let results: Value = client
        .post(format!("{}/devices/search", base))
        .header("Authorization", basic_auth(uuid1, token1))
        .json(&json!({"type": "sensor"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert!(results.as_array().unwrap().len() >= 1);
}

#[tokio::test]
async fn send_message() {
    let base = start_server().await;
    let client = reqwest::Client::new();

    let reg: Value = client
        .post(format!("{}/devices", base))
        .json(&json!({}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let uuid = reg["uuid"].as_str().unwrap();
    let token = reg["token"].as_str().unwrap();

    let resp: Value = client
        .post(format!("{}/messages", base))
        .header("Authorization", basic_auth(uuid, token))
        .json(&json!({"devices": ["*"], "payload": {"temp": 22}}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(resp["sent"], true);
}

#[tokio::test]
async fn token_generate_and_revoke() {
    let base = start_server().await;
    let client = reqwest::Client::new();

    let reg: Value = client
        .post(format!("{}/devices", base))
        .json(&json!({}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let uuid = reg["uuid"].as_str().unwrap();
    let token = reg["token"].as_str().unwrap();
    let auth = basic_auth(uuid, token);

    // Generate a new token
    let gen: Value = client
        .post(format!("{}/devices/{}/tokens", base, uuid))
        .header("Authorization", &auth)
        .json(&json!({}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let new_token = gen["token"].as_str().unwrap();
    assert!(!new_token.is_empty());

    // The new token should work for whoami
    let whoami: Value = client
        .get(format!("{}/whoami", base))
        .header("Authorization", basic_auth(uuid, new_token))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(whoami["uuid"], uuid);

    // Revoke the new token
    let status = client
        .delete(format!("{}/devices/{}/tokens/{}", base, uuid, new_token))
        .header("Authorization", &auth)
        .send()
        .await
        .unwrap()
        .status();

    assert!(status.is_success());
}

#[tokio::test]
async fn subscribe_to_broadcasts() {
    let base = start_server().await;
    let client = reqwest::Client::new();

    let reg: Value = client
        .post(format!("{}/devices", base))
        .json(&json!({}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let uuid = reg["uuid"].as_str().unwrap();
    let token = reg["token"].as_str().unwrap();
    let auth = basic_auth(uuid, token);

    let resp = client
        .post(format!("{}/devices/{}/subscriptions", base, uuid))
        .header("Authorization", &auth)
        .json(&json!({
            "emitterUuid": uuid,
            "subscriberUuid": uuid,
            "type": "broadcast-sent"
        }))
        .send()
        .await
        .unwrap();

    assert!(
        resp.status().is_success(),
        "subscribe failed with status {}",
        resp.status()
    );

    let sub: Value = resp.json().await.unwrap();
    assert_eq!(sub["emitterUuid"], uuid);
    assert_eq!(sub["subscriptionType"], "broadcast-sent");
}

#[tokio::test]
async fn unauthenticated_whoami_returns_401() {
    let base = start_server().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{}/whoami", base))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn wrong_credentials_returns_401() {
    let base = start_server().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{}/whoami", base))
        .header("Authorization", basic_auth("fake-uuid", "fake-token"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn full_device_lifecycle() {
    let base = start_server().await;
    let client = reqwest::Client::new();

    // 1. Register
    let reg: Value = client
        .post(format!("{}/devices", base))
        .json(&json!({"type": "lifecycle-test"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let uuid = reg["uuid"].as_str().unwrap();
    let token = reg["token"].as_str().unwrap();
    let auth = basic_auth(uuid, token);

    // 2. Update
    let updated: Value = client
        .put(format!("{}/devices/{}", base, uuid))
        .header("Authorization", &auth)
        .json(&json!({"status": "active"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(updated["status"], "active");

    // 3. Generate token
    let gen: Value = client
        .post(format!("{}/devices/{}/tokens", base, uuid))
        .header("Authorization", &auth)
        .json(&json!({}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(gen["token"].is_string());

    // 4. Send message
    let msg: Value = client
        .post(format!("{}/messages", base))
        .header("Authorization", &auth)
        .json(&json!({"devices": [uuid], "payload": "hello"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(msg["sent"], true);

    // 5. Subscribe
    let sub_resp = client
        .post(format!("{}/devices/{}/subscriptions", base, uuid))
        .header("Authorization", &auth)
        .json(&json!({
            "emitterUuid": uuid,
            "subscriberUuid": uuid,
            "type": "broadcast-sent"
        }))
        .send()
        .await
        .unwrap();
    assert!(sub_resp.status().is_success());

    // 6. Unregister
    let del = client
        .delete(format!("{}/devices/{}", base, uuid))
        .header("Authorization", &auth)
        .send()
        .await
        .unwrap();
    assert!(del.status().is_success());

    // 7. Verify gone
    let resp = client
        .get(format!("{}/whoami", base))
        .header("Authorization", &auth)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
}
