use axum::{extract::State, Json};
use serde_json::{json, Value};

use crate::AppState;

pub async fn status(State(state): State<AppState>) -> Json<Value> {
    Json(json!({
        "meshblu": true,
        "version": "2.0.0",
        "online": true,
        "connections": state.hub.online_count(),
        "engine": "freshblu"
    }))
}
