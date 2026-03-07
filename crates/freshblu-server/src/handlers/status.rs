use axum::{extract::State, http::StatusCode, Json};
use serde_json::{json, Value};

use crate::AppState;

pub async fn status(State(state): State<AppState>) -> Json<Value> {
    Json(json!({
        "meshblu": true,
        "version": "2.0.0",
        "online": true,
        "connections": state.bus.online_count(),
        "engine": "freshblu"
    }))
}

/// GET /healthcheck — verify store connectivity
pub async fn healthcheck(State(state): State<AppState>) -> (StatusCode, Json<Value>) {
    // Try a simple store operation to verify connectivity
    let dummy = uuid::Uuid::nil();
    match state.store.get_device(&dummy).await {
        Ok(_) => (StatusCode::OK, Json(json!({ "healthy": true }))),
        Err(e) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({ "healthy": false, "error": e.to_string() })),
        ),
    }
}

/// GET /publickey — return the server's global public key
pub async fn server_public_key(State(state): State<AppState>) -> Json<Value> {
    match &state.config.public_key {
        Some(key) => Json(json!({ "publicKey": key })),
        None => Json(json!({ "publicKey": null })),
    }
}
