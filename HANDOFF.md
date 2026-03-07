# FreshBlu Handoff

Last updated: 2026-03-06

## What This Project Is

FreshBlu is a Rust reimplementation of Meshblu (originally SkyNet.im / Citrix Octoblu), an IoT machine-to-machine messaging platform. Every device, service, or API gets a UUID and can message any other device regardless of protocol. The original was Node.js + MongoDB + Redis. FreshBlu rewrites the internals in Rust while keeping full API compatibility.

## Repository Layout

```
freshblu/
  Cargo.toml                    Workspace root
  crates/
    freshblu-core/              Types, permissions (Meshblu v2.0 whitelists), auth (bcrypt)
    freshblu-proto/             NATS subject helpers, wire types (DeliveryEnvelope, NatsEvent)
    freshblu-store/             DeviceStore trait + SQLite + PostgreSQL + Redis CachedStore
    freshblu-server/            HTTP/WS/MQTT server (axum + rumqttd), MessageBus trait
    freshblu-router/            NATS consumer, subscription resolution, delivery routing
    freshblu-cli/               meshblu-util compatible CLI (clap v4)
    freshblu-wasm/              WASM HTTP client (browser/Node.js)
  docker/
    Dockerfile                  Single-process image (SQLite + LocalBus)
    Dockerfile.gateway          Gateway pod for scaled deployment
    Dockerfile.router           Router worker pod
    docker-compose.prod.yml     Full stack: NATS + Postgres + Redis + gateway + router
  docs/
    api.md                      HTTP API reference
    websocket.md                WebSocket protocol reference
    mqtt.md                     MQTT protocol reference
    permissions.md              Permission model reference
```

## Architecture

### Single-Process Mode (default)

```
cargo run --bin freshblu-server
```

Uses SQLite for storage and LocalBus for in-memory pub/sub. No external dependencies. This is the default and works out of the box.

### Scaled Mode

Set `NATS_URL`, `DATABASE_URL` (postgresql://), and `REDIS_URL` to switch to:

- PostgreSQL for storage
- NatsBus for cross-pod message routing
- Redis for auth/device caching and presence tracking
- Separate router workers for subscription fanout

### Key Abstractions

- **DeviceStore trait** (`freshblu-store/src/store.rs`): All storage operations. Implementations: SqliteStore, PostgresStore, CachedStore (Redis decorator).
- **MessageBus trait** (`freshblu-server/src/bus.rs`): Pub/sub for device events. Implementations: LocalBus (in-process), NatsBus (NATS-backed).
- **PermissionChecker** (`freshblu-core/src/permissions.rs`): Evaluates Meshblu v2.0 whitelists. Used by every handler.

### Feature Flags

| Crate | Feature | Default | What It Gates |
|-------|---------|---------|---------------|
| freshblu-core | `auth` | yes | bcrypt, rand, sha2 (all auth functions) |
| freshblu-store | `sqlite` | yes | SQLite backend |
| freshblu-store | `postgres` | no | PostgreSQL backend |
| freshblu-store | `cache` | no | Redis CachedStore decorator |

## Current State

### What Works

- Full Meshblu HTTP API (register, get, update, delete, search, whoami, messages, subscriptions, tokens)
- WebSocket transport with identity/ready handshake, all event types
- MQTT transport with rumqttd embedded broker, auth, message/broadcast routing
- Meshblu v2.0 permission system with all whitelist categories
- x-meshblu-as impersonation for discover, configure, message, and broadcast
- configure.sent subscriber fanout on both HTTP and WS update paths
- broadcast.as permission enforcement
- Foreign key cascading on device deletion (tokens and subscriptions)
- Parameterized SQL queries throughout (no injection vectors)
- Prometheus metrics at /metrics
- 185 tests passing, 5 stress tests, 5 MQTT broker tests, 16 benchmarks

### Known Issues (not yet fixed)

These are real bugs or gaps discovered during the most recent audit.

**Security (medium severity):**

1. WS Register handler does not enforce `open_registration` flag. The HTTP handler checks `state.config.open_registration`, but the WS handler calls `state.store.register()` directly.
   - File: `crates/freshblu-server/src/ws.rs`, lines 271-289

2. WS Unregister only allows self-deletion. The HTTP DELETE handler checks `configure.update` permission so other authorized devices can delete, but the WS handler only checks `uuid == device_uuid`.
   - File: `crates/freshblu-server/src/ws.rs`, lines 291-298

3. Broadcast delivery does not re-check subscriber permissions at send time. It checks the subscriber device exists but does not verify current whitelist state. A subscriber whose permissions were revoked after subscribing will still receive broadcasts.
   - File: `crates/freshblu-server/src/handlers/messages.rs`, lines 118-124

4. Subscription HTTP handlers (create, list, delete) do not support the `x-meshblu-as` header. All other handler groups support it.
   - File: `crates/freshblu-server/src/handlers/subscriptions.rs`

5. Redis cache uses the `KEYS` command for invalidation, which blocks the Redis server and is O(N). Should use SCAN. Invalidation failures are silently ignored.
   - File: `crates/freshblu-store/src/cache.rs`, lines 60-72

**Functional gaps:**

6. PresenceTracker is defined but never wired into the server startup. Multi-pod deployments have no presence tracking.
   - File: `crates/freshblu-server/src/presence.rs`

7. MQTT auth handler uses nested runtime blocking (`block_on` inside `spawn_blocking`) which could deadlock under heavy load.
   - File: `crates/freshblu-server/src/mqtt.rs`, lines 36-53

**Testing gaps:**

8. `/devices/search` endpoint has no tests verifying actual search/filter behavior.
9. `/mydevices` endpoint is completely untested.
10. `/status` endpoint response body is not validated in tests.
11. HTTP subscription CRUD tests check status codes but do not verify store state.
12. WS subscribe "allowed" test is vacuous (passes even if subscription was not created).

### What Was Fixed in This Session

These issues from the original audit have been resolved:

- WS message handler was missing `can_message_from()` permission check (fixed)
- WS connections were not tracked in Prometheus metrics (fixed)
- NatsBus mapped Ready/NotReady events to Unregister (fixed, now returns None)
- MESSAGES_SENT counter was not incremented (fixed)
- WS Update handler was missing configure.sent subscriber fanout (fixed)
- broadcast.as permission was never checked in messages handler (fixed)
- SQL interpolation in search_devices online filter (fixed, now parameterized)
- SQLite foreign keys were not enforced, subscriptions table lacked FK constraints (fixed)

## How to Build and Test

```bash
# Build everything
cargo build --workspace

# Run all tests (excludes ignored tests)
cargo test --workspace

# Run MQTT broker integration tests (starts real broker, needs port availability)
cargo test -p freshblu-server mqtt -- --ignored

# Run stress tests
cargo test -p freshblu-server stress -- --ignored

# Run benchmarks
cargo bench --workspace

# Start the server (single-process mode)
cargo run --bin freshblu-server
```

## Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| FRESHBLU_HTTP_PORT | 3000 | HTTP and WebSocket port |
| FRESHBLU_MQTT_PORT | 1883 | MQTT broker port |
| DATABASE_URL | sqlite:freshblu.db | SQLite or PostgreSQL connection string |
| NATS_URL | (unset) | Set to enable NatsBus (e.g. nats://localhost:4222) |
| REDIS_URL | (unset) | Set to enable Redis cache and presence |
| FRESHBLU_PEPPER | change-me-in-production | Bcrypt pepper for token hashing |
| FRESHBLU_OPEN_REGISTRATION | true | Allow unauthenticated device registration |
| RUST_LOG | info | Log level |

## Key Files

If you need to understand how things work, start here:

- **Permission logic**: `crates/freshblu-core/src/permissions.rs`
- **HTTP handlers**: `crates/freshblu-server/src/handlers/` (devices.rs, messages.rs, subscriptions.rs, tokens.rs)
- **WebSocket handler**: `crates/freshblu-server/src/ws.rs`
- **MQTT adapter**: `crates/freshblu-server/src/mqtt.rs`
- **Storage trait**: `crates/freshblu-store/src/store.rs`
- **SQLite implementation**: `crates/freshblu-store/src/sqlite.rs`
- **Bus abstraction**: `crates/freshblu-server/src/bus.rs`
- **Server wiring**: `crates/freshblu-server/src/main.rs` and `src/lib.rs`
- **Test helpers**: `crates/freshblu-server/tests/helpers/mod.rs`
