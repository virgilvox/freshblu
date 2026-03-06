use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use freshblu_server::{build_router, AppState, MessageHub, ServerConfig};
use freshblu_store::sqlite::SqliteStore;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = ServerConfig::from_env();

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            format!("freshblu={},tower_http=debug", config.log_level).into()
        }))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("FreshBlu starting on port {}", config.http_port);
    tracing::info!("Database: {}", config.database_url);

    // Initialize storage
    let store: freshblu_store::DynStore = Arc::new(
        SqliteStore::new(&config.database_url).await?,
    );

    // Initialize message hub
    let hub = MessageHub::new();

    let state = AppState {
        store,
        hub,
        config: config.clone(),
    };

    let router = build_router(state);

    let addr = format!("0.0.0.0:{}", config.http_port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("HTTP/WebSocket listening on http://{}", addr);
    tracing::info!("WebSocket endpoint: ws://{}:{}/ws", "localhost", config.http_port);
    tracing::info!(r#"
 ___                   ____  _
|_ _|_ __ ___  _ __  | __ )| |_   _
 | || '__/ _ \| '_ \ |  _ \| | | | |
 | || | | (_) | | | || |_) | | |_| |
|___|_|  \___/|_| |_||____/|_|\__,_|

Meshblu-compatible IoT messaging platform
"#);

    axum::serve(listener, router).await?;
    Ok(())
}
