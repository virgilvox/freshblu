pub mod bus;
pub mod config;
pub mod handlers;
pub mod hub;
pub mod local_bus;
pub mod metrics;
pub mod mqtt;
pub mod nats_bus;
pub mod presence;
pub mod rate_limit;
pub mod webhook;
pub mod ws;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Json, Router,
};
use freshblu_core::error::FreshBluError;
use freshblu_store::DynStore;
use serde_json::json;
use std::sync::Arc;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

pub use bus::DynBus;
pub use config::ServerConfig;
pub use hub::MessageHub;
pub use rate_limit::RateLimiter;
pub use webhook::WebhookExecutor;

/// Newtype wrapper to implement IntoResponse for FreshBluError (orphan rule)
pub struct ApiError(pub FreshBluError);

impl From<FreshBluError> for ApiError {
    fn from(e: FreshBluError) -> Self {
        ApiError(e)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status =
            StatusCode::from_u16(self.0.http_status()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let body = json!({ "error": self.0.to_string() });
        (status, Json(body)).into_response()
    }
}

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub store: DynStore,
    pub bus: DynBus,
    pub config: ServerConfig,
    pub rate_limiter: Arc<RateLimiter>,
    pub webhook_executor: Arc<WebhookExecutor>,
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        // Status & healthcheck
        .route("/status", get(handlers::status::status))
        .route("/healthcheck", get(handlers::status::healthcheck))
        // Authentication
        .route("/authenticate", post(handlers::auth::authenticate))
        // Device registration / auth
        .route("/devices", post(handlers::devices::register))
        .route("/devices/search", post(handlers::devices::search))
        .route("/devices/:uuid", get(handlers::devices::get_device))
        .route("/devices/:uuid", put(handlers::devices::update_device))
        .route("/devices/:uuid", delete(handlers::devices::unregister))
        // v2 aliases
        .route("/v2/devices", post(handlers::devices::register))
        .route("/v2/devices/:uuid", get(handlers::devices::get_device))
        .route("/v2/devices/:uuid", put(handlers::devices::update_device))
        .route("/v2/devices/:uuid", delete(handlers::devices::unregister))
        .route("/v2/devices/search", post(handlers::devices::search))
        // v3 aliases
        .route("/v3/devices/:uuid", get(handlers::devices::get_device))
        // Whoami
        .route("/whoami", get(handlers::devices::whoami))
        .route("/v2/whoami", get(handlers::devices::whoami))
        // Messaging
        .route("/messages", post(handlers::messages::send_message))
        .route("/v2/messages", post(handlers::messages::send_message))
        // Broadcasts
        .route("/broadcasts", post(handlers::messages::broadcast))
        // My devices
        .route("/mydevices", get(handlers::devices::my_devices))
        // Claim device
        .route("/claimdevice/:uuid", post(handlers::devices::claim_device))
        // Public key endpoints
        .route(
            "/devices/:uuid/publickey",
            get(handlers::devices::get_public_key),
        )
        .route("/publickey", get(handlers::status::server_public_key))
        // Subscriptions
        .route(
            "/devices/:uuid/subscriptions",
            post(handlers::subscriptions::create_subscription),
        )
        .route(
            "/devices/:uuid/subscriptions",
            get(handlers::subscriptions::list_subscriptions),
        )
        .route(
            "/devices/:uuid/subscriptions/:emitter_uuid/:sub_type",
            delete(handlers::subscriptions::delete_subscription),
        )
        // Token management
        .route(
            "/devices/:uuid/tokens",
            post(handlers::tokens::generate_token),
        )
        .route(
            "/devices/:uuid/tokens/:token",
            delete(handlers::tokens::revoke_token),
        )
        .route("/devices/:uuid/token", post(handlers::tokens::reset_token))
        // Token search
        .route("/search/tokens", post(handlers::tokens::search_tokens))
        // Firehose / HTTP streaming
        .route("/subscribe", get(handlers::subscribe::subscribe))
        // WebSocket
        .route("/ws", get(ws::ws_handler))
        .route("/socket.io", get(ws::ws_handler))
        // Metrics
        .route("/metrics", get(metrics::metrics_handler))
        // Middleware
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
