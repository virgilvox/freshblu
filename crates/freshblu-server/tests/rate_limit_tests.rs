mod helpers;

use axum::http::{Method, StatusCode};
use helpers::*;

// ---------------------------------------------------------------------------
// Rate limiting tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn requests_within_limit_succeed() {
    let (app, state) = setup_router().await;
    let (uuid, token) = register_device(&state).await;
    let auth = basic_auth(&uuid, &token);

    // Make a few requests — should all succeed
    for _ in 0..5 {
        let resp = http_request(&app, Method::GET, "/whoami", Some(&auth), None).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }
}

#[tokio::test]
async fn exceeding_limit_returns_429() {
    // Create a server with a very low rate limit
    let mut config = freshblu_server::ServerConfig::default();
    config.rate_limit = 3;
    config.rate_window = 60;

    let (_, state) = setup_with_config(config).await;
    let (uuid, token) = register_device(&state).await;
    let auth = basic_auth(&uuid, &token);

    let app = freshblu_server::build_router(state.clone());

    // First 3 requests should succeed
    for i in 0..3 {
        let resp = http_request(&app, Method::GET, "/whoami", Some(&auth), None).await;
        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "request {} should succeed",
            i
        );
    }

    // 4th request should be rate-limited
    let resp = http_request(&app, Method::GET, "/whoami", Some(&auth), None).await;
    assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn rate_limit_per_device_isolation() {
    let mut config = freshblu_server::ServerConfig::default();
    config.rate_limit = 2;
    config.rate_window = 60;

    let (_, state) = setup_with_config(config).await;
    let (uuid1, token1) = register_device(&state).await;
    let (uuid2, token2) = register_device(&state).await;
    let auth1 = basic_auth(&uuid1, &token1);
    let auth2 = basic_auth(&uuid2, &token2);

    let app = freshblu_server::build_router(state.clone());

    // Exhaust device 1's limit
    for _ in 0..2 {
        let resp = http_request(&app, Method::GET, "/whoami", Some(&auth1), None).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }
    let resp = http_request(&app, Method::GET, "/whoami", Some(&auth1), None).await;
    assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);

    // Device 2 should still be fine
    let resp = http_request(&app, Method::GET, "/whoami", Some(&auth2), None).await;
    assert_eq!(resp.status(), StatusCode::OK);
}
