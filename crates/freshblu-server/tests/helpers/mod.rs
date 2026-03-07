#![allow(dead_code)]
use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
    response::Response,
};
use freshblu_core::device::RegisterParams;
use freshblu_core::permissions::Whitelists;
use freshblu_server::{build_router, AppState, DynBus, ServerConfig};
use freshblu_store::{sqlite::SqliteStore, DynStore};
use futures::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tower::ServiceExt;

pub type WsStream =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

/// Start a server, return WS URL + state
pub async fn setup() -> (String, AppState) {
    let store: DynStore = Arc::new(SqliteStore::new("sqlite::memory:").await.unwrap());
    let bus: DynBus = Arc::new(freshblu_server::local_bus::LocalBus::new());
    let state = AppState {
        store,
        bus,
        config: ServerConfig::default(),
    };
    let app = build_router(state.clone());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let ws_url = format!("ws://127.0.0.1:{}/ws", addr.port());
    (ws_url, state)
}

/// Start server with custom config
pub async fn setup_with_config(config: ServerConfig) -> (String, AppState) {
    let store: DynStore = Arc::new(SqliteStore::new("sqlite::memory:").await.unwrap());
    let bus: DynBus = Arc::new(freshblu_server::local_bus::LocalBus::new());
    let state = AppState { store, bus, config };
    let app = build_router(state.clone());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let ws_url = format!("ws://127.0.0.1:{}/ws", addr.port());
    (ws_url, state)
}

/// Setup returning just the router (no live server needed for oneshot tests)
pub async fn setup_router() -> (axum::Router, AppState) {
    let store: DynStore = Arc::new(SqliteStore::new("sqlite::memory:").await.unwrap());
    let bus: DynBus = Arc::new(freshblu_server::local_bus::LocalBus::new());
    let state = AppState {
        store,
        bus,
        config: ServerConfig::default(),
    };
    let app = build_router(state.clone());
    (app, state)
}

/// Register a device with open (public) whitelists via the store directly
pub async fn register_device(state: &AppState) -> (String, String) {
    let params = RegisterParams {
        device_type: Some("test".into()),
        ..Default::default()
    };
    let (device, token) = state.store.register(params).await.unwrap();
    (device.uuid.to_string(), token)
}

/// Register a device with empty (locked-down) whitelists
pub async fn register_private_device(state: &AppState) -> (String, String) {
    let params = RegisterParams {
        device_type: Some("private".into()),
        meshblu: Some(freshblu_core::device::WhitelistOverride {
            whitelists: Some(Whitelists::default()),
        }),
        ..Default::default()
    };
    let (device, token) = state.store.register(params).await.unwrap();
    (device.uuid.to_string(), token)
}

/// Register a device with specific whitelists
pub async fn register_device_with_whitelists(
    state: &AppState,
    whitelists: Whitelists,
) -> (String, String) {
    let params = RegisterParams {
        device_type: Some("test".into()),
        meshblu: Some(freshblu_core::device::WhitelistOverride {
            whitelists: Some(whitelists),
        }),
        ..Default::default()
    };
    let (device, token) = state.store.register(params).await.unwrap();
    (device.uuid.to_string(), token)
}

/// Format a Basic auth header
pub fn basic_auth(uuid: &str, token: &str) -> String {
    use base64::Engine;
    let encoded = base64::engine::general_purpose::STANDARD.encode(format!("{}:{}", uuid, token));
    format!("Basic {}", encoded)
}

/// Read a JSON message from a WS stream with 2s timeout
pub async fn recv_json(
    ws: &mut (impl StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin),
) -> Option<Value> {
    match tokio::time::timeout(std::time::Duration::from_secs(2), ws.next()).await {
        Ok(Some(Ok(Message::Text(text)))) => serde_json::from_str(&text).ok(),
        _ => None,
    }
}

/// Connect WS, authenticate, and return the ready stream
pub async fn connect_and_auth(ws_url: &str, uuid: &str, token: &str) -> WsStream {
    let (mut ws, _) = connect_async(ws_url).await.expect("failed to connect");

    let identity = json!({
        "event": "identity",
        "uuid": uuid,
        "token": token,
    });
    ws.send(Message::Text(identity.to_string())).await.unwrap();

    let resp = recv_json(&mut ws).await.expect("expected ready response");
    assert_eq!(resp["event"], "ready");
    ws
}

/// Make an HTTP request against the router
pub async fn http_request(
    app: &axum::Router,
    method: Method,
    uri: &str,
    auth: Option<&str>,
    body: Option<Value>,
) -> Response<Body> {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(a) = auth {
        builder = builder.header("authorization", a);
    }
    if body.is_some() {
        builder = builder.header("content-type", "application/json");
    }

    let body = match body {
        Some(v) => Body::from(serde_json::to_string(&v).unwrap()),
        None => Body::empty(),
    };

    app.clone()
        .oneshot(builder.body(body).unwrap())
        .await
        .unwrap()
}

/// Extract JSON body from response
pub async fn response_json(resp: Response<Body>) -> Value {
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&body).unwrap()
}
