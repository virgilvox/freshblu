# FreshBlu -- Architecture Plan

A modern, scalable reimplementation of Meshblu/Octoblu.

## Current State (as of 2026-03-06)

### What's Built and Working

| Crate | Status | Notes |
|---|---|---|
| `freshblu-core` | **Done** | Types, permissions (Meshblu v2.0 whitelists), auth (bcrypt, feature-gated), subscriptions, tokens |
| `freshblu-proto` | **Done** | NATS subject helpers (`subjects.rs`) + wire types (`DeliveryEnvelope`, `NatsEvent`) |
| `freshblu-store` | **Done** | `DeviceStore` trait, SQLite backend, PostgreSQL backend, Redis `CachedStore` decorator -- all feature-gated |
| `freshblu-server` | **Done** | HTTP/WS/MQTT on axum + rumqttd, `MessageBus` trait with `LocalBus` (in-memory) and `NatsBus` (NATS-backed), Prometheus metrics |
| `freshblu-router` | **Done** | NATS consumer + fanout worker with presence-based delivery routing |
| `freshblu-cli` | **Done** | Full meshblu-util compatible CLI (clap v4) |
| `freshblu-wasm` | **Done** | WASM HTTP client (wasm-bindgen + gloo-net), uses freshblu-core without auth feature |

### Test Coverage

185 tests passing + 5 stress tests (ignored) + 5 MQTT broker tests (ignored) + 16 benchmarks:

- 21 core unit tests
- 2 proto unit tests
- 6 bus unit tests
- 21 API integration tests (HTTP handlers)
- 6 security regression tests
- 10 end-to-end delivery tests
- 37 permission matrix tests
- 9 WebSocket protocol tests
- 7 MQTT unit tests
- 26 store conformance tests
- 13 router/proto tests
- 8 WebSocket integration tests
- 17 store unit tests
- 5 stress tests (ignored)
- 5 MQTT broker integration tests (ignored)
- 16 benchmarks (criterion)

### Infrastructure Phases Completed

1. **freshblu-proto** -- Shared NATS protocol types (subjects, envelopes)
2. **PostgreSQL backend** -- Full `DeviceStore` implementation with JSONB, migrations
3. **Redis cache layer** -- `CachedStore` decorator (auth 5min, device 60s, subscribers 60s)
4. **MessageBus trait** -- `LocalBus` (wraps existing `MessageHub`) + `NatsBus` (NATS-backed)
5. **Router worker** -- NATS consumer, subscription resolution, presence-based fanout
6. **Presence tracking** -- Redis-based `PresenceTracker` (30s TTL, 15s heartbeat) -- defined but NOT wired into server startup
7. **Prometheus metrics** -- WS/MQTT connection gauges, message counters, auth counters, `/metrics` endpoint
8. **WASM feature-gating** -- `freshblu-core` auth feature, WASM uses core types without bcrypt
9. **Production Docker** -- `docker-compose.prod.yml` with NATS + Postgres + Redis + 2x gateway + 2x router

### AppState

```rust
pub struct AppState {
    pub store: DynStore,       // DeviceStore trait object
    pub bus: DynBus,           // MessageBus trait object (LocalBus or NatsBus)
    pub config: ServerConfig,  // Runtime configuration
}
```

### Backward Compatibility

Single-process mode fully preserved:
```bash
cargo run --bin freshblu-server
# Uses LocalBus + SQLite -- no NATS/Postgres/Redis needed
```

Multi-process mode activated by env vars:
- `NATS_URL` -> NatsBus instead of LocalBus
- `DATABASE_URL` (starts with `postgresql://`) -> Postgres instead of SQLite
- `REDIS_URL` -> CachedStore wrapper + PresenceTracker

## Known Issues (Audit Findings)

### Security (medium severity)

1. **WS Register ignores open_registration** -- The HTTP handler checks `state.config.open_registration`, but the WS handler calls `state.store.register()` directly without checking the flag.
   - File: `crates/freshblu-server/src/ws.rs`

2. **WS Unregister only allows self-deletion** -- The HTTP DELETE handler checks `configure.update` permission so other authorized devices can delete, but the WS handler only checks `uuid == device_uuid`.
   - File: `crates/freshblu-server/src/ws.rs`

3. **Broadcast delivery skips subscriber permission recheck** -- It checks the subscriber device exists but does not verify current whitelist state. A subscriber whose permissions were revoked after subscribing will still receive broadcasts.
   - File: `crates/freshblu-server/src/handlers/messages.rs`

4. **Subscription handlers lack x-meshblu-as support** -- All other handler groups support the impersonation header, but subscriptions do not.
   - File: `crates/freshblu-server/src/handlers/subscriptions.rs`

5. **Redis cache uses KEYS command** -- Blocks the Redis server (O(N)). Should use SCAN. Invalidation failures are silently ignored.
   - File: `crates/freshblu-store/src/cache.rs`

### Functional gaps

6. **PresenceTracker not wired in** -- Defined in `presence.rs` but never instantiated in `main.rs`. Multi-pod deployments have no presence tracking.

7. **MQTT auth nested runtime** -- Uses `block_on` inside `spawn_blocking`, which could deadlock under heavy load.
   - File: `crates/freshblu-server/src/mqtt.rs`

### Testing gaps

8. `/devices/search` endpoint has no tests verifying actual search/filter behavior.
9. `/mydevices` endpoint is completely untested.
10. `/status` endpoint response body is not validated in tests.
11. HTTP subscription CRUD tests check status codes but do not verify store state.
12. WS subscribe "allowed" test is vacuous (passes even if subscription was not created).

## Target Architecture

```
                        ┌─────────────────────────────────────┐
                        │           CLIENT LAYER               │
                        │  HTTP  WebSocket  MQTT  CoAP         │
                        └──────────┬──────────────────────────┘
                                   │
                    ┌──────────────▼──────────────────┐
                    │       GATEWAY PODS               │
                    │  (stateless, N replicas)         │
                    │                                  │
                    │  1. Auth (Redis cache -> Postgres)│
                    │  2. Permission check (cached)    │
                    │  3. Publish event -> NATS         │
                    │  4. Subscribe to own delivery    │
                    │     topic, forward to WS/MQTT    │
                    └──────┬───────────────────┬───────┘
                           │ publish            │ subscribe
              ┌────────────▼────────────────────▼────────────┐
              │              NATS JETSTREAM                    │
              │                                               │
              │  freshblu.device.{uuid}.inbox   (direct msg) │
              │  freshblu.broadcast.{uuid}      (broadcasts) │
              │  freshblu.configure.{uuid}      (config evt) │
              │  freshblu.system.unregister     (unreg evt)  │
              │  freshblu.delivery.{pod-id}     (routed out) │
              └──────────────────┬────────────────────────────┘
                                 │ consume
                    ┌────────────▼────────────────┐
                    │       ROUTER WORKERS         │
                    │  (stateless, N replicas)     │
                    │                              │
                    │  Consume events from NATS    │
                    │  Resolve subscriptions       │
                    │  Look up presence (which pod)│
                    │  Publish to delivery topic   │
                    └──────────────────────────────┘
                                 │
              ┌──────────────────▼────────────────────────────┐
              │                 DATA LAYER                     │
              │                                               │
              │  PostgreSQL  -- device registry, subs, tokens │
              │  Redis       -- auth cache, presence, rate lim│
              └───────────────────────────────────────────────┘
```

## NATS Subject Hierarchy

Defined in `crates/freshblu-proto/src/subjects.rs`:

| Subject | Purpose |
|---|---|
| `freshblu.device.{uuid}.inbox` | Direct message to a device |
| `freshblu.broadcast.{uuid}` | Broadcast from a device |
| `freshblu.configure.{uuid}` | Config update for a device |
| `freshblu.system.unregister` | Device unregistration event |
| `freshblu.delivery.{pod-id}` | Routed delivery to a gateway pod |
| `freshblu.presence.{uuid}` | Presence tracking |

## Crate Structure

```
crates/
  freshblu-core/       # Types, permissions, auth (feature-gated)
  freshblu-proto/      # NATS subjects + wire types (DeliveryEnvelope, NatsEvent)
  freshblu-store/      # DeviceStore trait + SQLite/Postgres/CachedStore impls
  freshblu-server/     # Gateway: HTTP/WS/MQTT, MessageBus trait, LocalBus/NatsBus
  freshblu-router/     # NATS consumer + subscription fanout worker
  freshblu-cli/        # CLI (meshblu-util compatible)
  freshblu-wasm/       # WASM HTTP client
```

### Feature Flags

| Crate | Feature | Default | What it gates |
|---|---|---|---|
| `freshblu-core` | `auth` | Yes | bcrypt, rand, sha2, hmac, base64 -- all auth functions |
| `freshblu-store` | `sqlite` | Yes | SQLite backend |
| `freshblu-store` | `postgres` | No | PostgreSQL backend |
| `freshblu-store` | `cache` | No | Redis CachedStore decorator |

## Deployment Modes

### Local dev (zero deps)
```bash
cargo run --bin freshblu-server
# SQLite + LocalBus -- everything in one process
```

### Production (docker-compose)
```bash
docker compose -f docker/docker-compose.prod.yml up
# NATS + Postgres + Redis + 2x gateway + 2x router
```

## What Needs to Be Done Next

### Bug Fixes (from audit)
- [ ] Wire `PresenceTracker` into server `main.rs` and WS/MQTT handlers
- [ ] Fix MQTT auth nested runtime issue (`block_on` inside `spawn_blocking`)
- [ ] Enforce `open_registration` in WS Register handler
- [ ] Add `configure.update` permission check to WS Unregister handler
- [ ] Re-check subscriber permissions at broadcast delivery time
- [ ] Add `x-meshblu-as` support to subscription handlers
- [ ] Replace Redis `KEYS` command with `SCAN` in cache invalidation

### Testing
- [ ] Search endpoint behavior tests
- [ ] `/mydevices` endpoint tests
- [ ] `/status` response body validation
- [ ] Subscription CRUD store-state verification
- [ ] WS subscribe correctness test (verify subscription was actually created)

### Future
- Rate limiting (Redis-based, per-device)
- NATS KV for device last-known state
- CoAP transport
- Kubernetes manifests + HPA configs
- Offline message queuing (JetStream inbox per device)
