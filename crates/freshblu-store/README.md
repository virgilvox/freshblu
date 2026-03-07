# freshblu-store

Storage abstraction layer for FreshBlu with pluggable backends.

## Overview

Defines the `DeviceStore` trait and provides three implementations: SQLite (default), PostgreSQL, and a Redis-backed cache decorator. Each backend is gated behind a feature flag.

## DeviceStore Trait

```rust
#[async_trait]
pub trait DeviceStore: Send + Sync + 'static {
    async fn register(&self, params: RegisterParams) -> Result<(Device, String)>;
    async fn get_device(&self, uuid: &Uuid) -> Result<Option<Device>>;
    async fn update_device(&self, uuid: &Uuid, properties: HashMap<String, Value>) -> Result<Device>;
    async fn authenticate(&self, uuid: &Uuid, token: &str) -> Result<Option<Device>>;
    async fn create_subscription(&self, params: &CreateSubscriptionParams) -> Result<Subscription>;
    // ... and more
}
```

## SQLite Backend

Enabled by default with the `sqlite` feature.

```rust
use freshblu_store::sqlite::SqliteStore;

let store = SqliteStore::new("sqlite:freshblu.db").await?;
// or in-memory for testing:
let store = SqliteStore::new("sqlite::memory:").await?;
```

Tables are created automatically on first run. Foreign keys are enforced (subscriptions and tokens cascade on device deletion).

## PostgreSQL Backend

Enable with the `postgres` feature flag.

```rust
use freshblu_store::postgres::PostgresStore;

let store = PostgresStore::new("postgresql://user:pass@localhost/freshblu").await?;
```

Uses JSONB for device properties. Migrations are in `migrations/001_initial.sql`.

## CachedStore (Redis)

Enable with the `cache` feature flag. Wraps any `DeviceStore` with a Redis cache layer.

```rust
use freshblu_store::cache::CachedStore;
use freshblu_store::sqlite::SqliteStore;

let inner = Arc::new(SqliteStore::new("sqlite:freshblu.db").await?);
let cached = CachedStore::new(inner, "redis://localhost:6379").await?;
```

Cache TTLs: auth results (5 min), device lookups (60s), subscriber lists (60s).

## Feature Flags

| Feature | Default | What it gates |
|---|---|---|
| `sqlite` | Yes | SQLite backend (`SqliteStore`) |
| `postgres` | No | PostgreSQL backend (`PostgresStore`) |
| `cache` | No | Redis cache decorator (`CachedStore`) |

## Extending

Implement `DeviceStore` for your backend:

```rust
use freshblu_store::DeviceStore;

struct MyStore { /* ... */ }

#[async_trait]
impl DeviceStore for MyStore {
    // implement all trait methods
}
```

Then use it as `DynStore`:

```rust
let store: DynStore = Arc::new(MyStore::new());
```

## License

MIT OR Apache-2.0
