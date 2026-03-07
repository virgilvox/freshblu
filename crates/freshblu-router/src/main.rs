mod consumer;
mod fanout;

use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "freshblu=info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let nats_url =
        std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for router");
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    tracing::info!("FreshBlu Router starting");
    tracing::info!("NATS: {}", nats_url);
    tracing::info!("Database: {}", database_url);
    tracing::info!("Redis: {}", redis_url);

    // Connect to PostgreSQL
    let inner_store: freshblu_store::DynStore =
        Arc::new(freshblu_store::postgres::PostgresStore::new(&database_url).await?);

    // Wrap with Redis cache
    let store: freshblu_store::DynStore =
        Arc::new(freshblu_store::cache::CachedStore::new(inner_store, &redis_url).await?);

    // Connect to NATS
    let nats = async_nats::connect(&nats_url).await?;

    // Connect to Redis for presence lookups
    let redis_client = redis::Client::open(redis_url.as_str())?;
    let redis = redis::aio::ConnectionManager::new(redis_client).await?;

    let fanout = fanout::Fanout { store, redis, nats };

    tracing::info!("Router consumer starting...");
    consumer::run_consumer(fanout).await?;

    Ok(())
}
