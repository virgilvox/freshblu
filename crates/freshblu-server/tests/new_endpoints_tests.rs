mod helpers;

use axum::http::{Method, StatusCode};
use helpers::*;
use serde_json::json;

// ---------------------------------------------------------------------------
// Healthcheck
// ---------------------------------------------------------------------------

#[tokio::test]
async fn healthcheck_returns_200() {
    let (app, _) = setup_router().await;
    let resp = http_request(&app, Method::GET, "/healthcheck", None, None).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = response_json(resp).await;
    assert_eq!(body["healthy"], true);
}

// ---------------------------------------------------------------------------
// Server public key
// ---------------------------------------------------------------------------

#[tokio::test]
async fn server_publickey_returns_null_when_not_configured() {
    let (app, _) = setup_router().await;
    let resp = http_request(&app, Method::GET, "/publickey", None, None).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = response_json(resp).await;
    assert!(body["publicKey"].is_null());
}

// ---------------------------------------------------------------------------
// Device public key
// ---------------------------------------------------------------------------

#[tokio::test]
async fn device_publickey_returns_null_when_not_set() {
    let (app, state) = setup_router().await;
    let (uuid, _) = register_device(&state).await;

    let resp = http_request(
        &app,
        Method::GET,
        &format!("/devices/{}/publickey", uuid),
        None,
        None,
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = response_json(resp).await;
    assert!(body["publicKey"].is_null());
}

// ---------------------------------------------------------------------------
// Broadcast endpoint
// ---------------------------------------------------------------------------

#[tokio::test]
async fn broadcast_endpoint_works() {
    let (app, state) = setup_router().await;
    let (uuid, token) = register_device(&state).await;
    let auth = basic_auth(&uuid, &token);

    let resp = http_request(
        &app,
        Method::POST,
        "/broadcasts",
        Some(&auth),
        Some(json!({
            "payload": { "temp": 72 }
        })),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = response_json(resp).await;
    assert_eq!(body["sent"], true);
}

// ---------------------------------------------------------------------------
// Claim device
// ---------------------------------------------------------------------------

#[tokio::test]
async fn claim_device_sets_owner() {
    let (app, state) = setup_router().await;
    let (claimer_uuid, claimer_token) = register_device(&state).await;
    let (target_uuid, _) = register_device(&state).await;
    let auth = basic_auth(&claimer_uuid, &claimer_token);

    let resp = http_request(
        &app,
        Method::POST,
        &format!("/claimdevice/{}", target_uuid),
        Some(&auth),
        None,
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = response_json(resp).await;
    assert_eq!(body["owner"], claimer_uuid);
}

#[tokio::test]
async fn claim_device_fails_if_already_claimed() {
    let (app, state) = setup_router().await;
    let (claimer_uuid, claimer_token) = register_device(&state).await;
    let (target_uuid, _) = register_device(&state).await;
    let auth = basic_auth(&claimer_uuid, &claimer_token);

    // First claim succeeds
    let resp = http_request(
        &app,
        Method::POST,
        &format!("/claimdevice/{}", target_uuid),
        Some(&auth),
        None,
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Second claim fails
    let resp = http_request(
        &app,
        Method::POST,
        &format!("/claimdevice/{}", target_uuid),
        Some(&auth),
        None,
    )
    .await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

// ---------------------------------------------------------------------------
// Reset token
// ---------------------------------------------------------------------------

#[tokio::test]
async fn reset_token_returns_new_token() {
    let (app, state) = setup_router().await;
    let (uuid, token) = register_device(&state).await;
    let auth = basic_auth(&uuid, &token);

    let resp = http_request(
        &app,
        Method::POST,
        &format!("/devices/{}/token", uuid),
        Some(&auth),
        None,
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = response_json(resp).await;
    assert!(body["token"].is_string());
    let new_token = body["token"].as_str().unwrap();
    assert_ne!(new_token, token);
}

#[tokio::test]
async fn reset_token_invalidates_old_token() {
    let (app, state) = setup_router().await;
    let (uuid, token) = register_device(&state).await;
    let auth = basic_auth(&uuid, &token);

    // Reset
    let resp = http_request(
        &app,
        Method::POST,
        &format!("/devices/{}/token", uuid),
        Some(&auth),
        None,
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = response_json(resp).await;
    let new_token = body["token"].as_str().unwrap().to_string();

    // Old token should no longer work
    let resp = http_request(&app, Method::GET, "/whoami", Some(&auth), None).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    // New token should work
    let new_auth = basic_auth(&uuid, &new_token);
    let resp = http_request(&app, Method::GET, "/whoami", Some(&new_auth), None).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

// ---------------------------------------------------------------------------
// Token search
// ---------------------------------------------------------------------------

#[tokio::test]
async fn search_tokens_by_tag() {
    let (app, state) = setup_router().await;
    let (uuid, token) = register_device(&state).await;
    let auth = basic_auth(&uuid, &token);

    // Generate a token with a tag
    let resp = http_request(
        &app,
        Method::POST,
        &format!("/devices/{}/tokens", uuid),
        Some(&auth),
        Some(json!({ "tag": "test-tag" })),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Search for tokens with that tag
    let resp = http_request(
        &app,
        Method::POST,
        "/search/tokens",
        Some(&auth),
        Some(json!({ "tag": "test-tag" })),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = response_json(resp).await;
    let tokens: Vec<serde_json::Value> = serde_json::from_value(body).unwrap();
    assert!(!tokens.is_empty());
    assert_eq!(tokens[0]["tag"], "test-tag");
}

// ---------------------------------------------------------------------------
// Message size validation
// ---------------------------------------------------------------------------

#[tokio::test]
async fn message_too_large_returns_413() {
    let mut config = freshblu_server::ServerConfig::default();
    config.max_message_size = 100; // Very small limit

    let (_, state) = setup_with_config(config).await;
    let (uuid, token) = register_device(&state).await;
    let auth = basic_auth(&uuid, &token);

    let app = freshblu_server::build_router(state.clone());

    // Create a large payload
    let large_payload = "x".repeat(200);
    let resp = http_request(
        &app,
        Method::POST,
        "/messages",
        Some(&auth),
        Some(json!({
            "devices": ["*"],
            "payload": large_payload
        })),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE);
}

// ---------------------------------------------------------------------------
// Subscribe (firehose) endpoint exists
// ---------------------------------------------------------------------------

#[tokio::test]
async fn subscribe_endpoint_requires_auth() {
    let (app, _) = setup_router().await;
    let resp = http_request(&app, Method::GET, "/subscribe", None, None).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}
