# freshblu-server

HTTP/WebSocket/MQTT server for the FreshBlu IoT messaging platform.

## Overview

The server binary provides three transport layers:

- **HTTP REST** -- Meshblu-compatible API on configurable port (default 3000)
- **WebSocket** -- Real-time bidirectional communication at `/ws` and `/socket.io`
- **MQTT** -- IoT-standard protocol via embedded rumqttd broker (default port 1883)

Built with axum for HTTP/WS and rumqttd for MQTT.

## Architecture

```
AppState
├── store: DynStore        -- pluggable storage backend
├── bus: DynBus            -- message routing (LocalBus or NatsBus)
└── config: ServerConfig   -- runtime configuration

MessageBus trait (bus.rs)
├── LocalBus   -- in-process event routing via DashMap + broadcast channels
└── NatsBus    -- cross-pod routing via NATS (set NATS_URL to enable)
```

In single-process mode, `LocalBus` wraps the internal `MessageHub` for in-memory pub/sub. When `NATS_URL` is set, `NatsBus` publishes events to NATS and listens on a pod-specific delivery topic.

## Running

```bash
cargo run --bin freshblu-server

# With custom config
FRESHBLU_HTTP_PORT=8080 FRESHBLU_MQTT_PORT=1884 cargo run --bin freshblu-server
```

## As a Library

```rust
use freshblu_server::{build_router, AppState, ServerConfig, DynBus};
use freshblu_server::local_bus::LocalBus;
use freshblu_store::sqlite::SqliteStore;
use std::sync::Arc;

let store = Arc::new(SqliteStore::new("sqlite::memory:").await?);
let bus: DynBus = Arc::new(LocalBus::new());
let state = AppState { store, bus, config: ServerConfig::default() };
let router = build_router(state);
// Use with axum::serve()
```

## Feature Flags

| Feature | Default | Description |
|---|---|---|
| `sqlite` | Yes | SQLite storage backend |
| `postgres` | No | PostgreSQL storage backend |
| `cache` | No | Redis cache layer (CachedStore) |

## License

MIT OR Apache-2.0
