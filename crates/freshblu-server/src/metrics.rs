use axum::response::IntoResponse;
use lazy_static::lazy_static;
use prometheus::{
    Encoder, IntCounter, IntGauge, TextEncoder,
};

lazy_static! {
    pub static ref WS_CONNECTIONS: IntGauge =
        IntGauge::new("freshblu_ws_connections", "Current WebSocket connections")
            .expect("metric can be created");

    pub static ref MQTT_CONNECTIONS: IntGauge =
        IntGauge::new("freshblu_mqtt_connections", "Current MQTT connections")
            .expect("metric can be created");

    pub static ref MESSAGES_SENT: IntCounter =
        IntCounter::new("freshblu_messages_sent_total", "Total messages published")
            .expect("metric can be created");

    pub static ref MESSAGES_DELIVERED: IntCounter =
        IntCounter::new("freshblu_messages_delivered_total", "Total messages delivered locally")
            .expect("metric can be created");

    pub static ref AUTH_REQUESTS: IntCounter =
        IntCounter::new("freshblu_auth_requests_total", "Total authentication attempts")
            .expect("metric can be created");

    pub static ref AUTH_CACHE_HITS: IntCounter =
        IntCounter::new("freshblu_auth_cache_hits_total", "Redis auth cache hits")
            .expect("metric can be created");
}

pub fn register_metrics() {
    let registry = prometheus::default_registry();
    let _ = registry.register(Box::new(WS_CONNECTIONS.clone()));
    let _ = registry.register(Box::new(MQTT_CONNECTIONS.clone()));
    let _ = registry.register(Box::new(MESSAGES_SENT.clone()));
    let _ = registry.register(Box::new(MESSAGES_DELIVERED.clone()));
    let _ = registry.register(Box::new(AUTH_REQUESTS.clone()));
    let _ = registry.register(Box::new(AUTH_CACHE_HITS.clone()));
}

pub async fn metrics_handler() -> impl IntoResponse {
    register_metrics();
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    (
        [(axum::http::header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        buffer,
    )
}
