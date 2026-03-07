use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use freshblu_server::{build_router, bus::DynBus, AppState, ServerConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = ServerConfig::from_env();

    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("freshblu={},tower_http=debug", config.log_level).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("FreshBlu starting on port {}", config.http_port);
    tracing::info!("Database: {}", config.database_url);

    // Initialize storage
    let store: freshblu_store::DynStore = if config.database_url.starts_with("postgres") {
        #[cfg(feature = "postgres")]
        {
            Arc::new(freshblu_store::postgres::PostgresStore::new(&config.database_url).await?)
        }
        #[cfg(not(feature = "postgres"))]
        {
            anyhow::bail!("PostgreSQL support not compiled in. Enable the 'postgres' feature.");
        }
    } else {
        #[cfg(feature = "sqlite")]
        {
            Arc::new(freshblu_store::sqlite::SqliteStore::new(&config.database_url).await?)
        }
        #[cfg(not(feature = "sqlite"))]
        {
            anyhow::bail!("SQLite support not compiled in. Enable the 'sqlite' feature.");
        }
    };

    // Optionally wrap with Redis cache
    #[cfg(feature = "cache")]
    let store: freshblu_store::DynStore = if let Some(ref redis_url) = config.redis_url {
        tracing::info!("Redis cache enabled: {}", redis_url);
        Arc::new(freshblu_store::cache::CachedStore::new(store, redis_url).await?)
    } else {
        store
    };

    // Initialize message bus
    let bus: DynBus = if let Some(ref nats_url) = config.nats_url {
        tracing::info!("NATS bus enabled: {} (pod: {})", nats_url, config.pod_id);
        Arc::new(freshblu_server::nats_bus::NatsBus::new(nats_url, config.pod_id.clone()).await?)
    } else {
        tracing::info!("Using local in-memory message bus (single-process mode)");
        Arc::new(freshblu_server::local_bus::LocalBus::new())
    };

    let state = AppState {
        store,
        bus,
        config: config.clone(),
    };

    // Start MQTT broker
    let mqtt = freshblu_server::mqtt::MqttAdapter::new(
        state.store.clone(),
        state.bus.clone(),
        config.mqtt_port,
    );
    mqtt.start().await?;

    let router = build_router(state);

    let addr = format!("0.0.0.0:{}", config.http_port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("HTTP/WebSocket listening on http://{}", addr);
    tracing::info!(
        "WebSocket endpoint: ws://{}:{}/ws",
        "localhost",
        config.http_port
    );
    tracing::info!(
        r#"
 _____              _     ____  _
|  ___| __ ___  ___| |__ | __ )| |_   _
| |_ | '__/ _ \/ __| '_ \|  _ \| | | | |
|  _|| | |  __/\__ \ | | | |_) | | |_| |
|_|  |_|  \___||___/_| |_|____/|_|\__,_|

Meshblu-compatible IoT messaging platform
"#
    );

    axum::serve(listener, router).await?;
    Ok(())
}
