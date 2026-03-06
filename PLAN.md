# FreshBlu — Architecture Plan

A modern, scalable reimplementation of Meshblu/Octoblu.
The scaffold in this repo gives you correct core types, permissions, CLI, and SDKs.
This document is the full plan for making it production-grade and horizontally scalable.

---

## What the scaffold gives you (use it)

| Crate / File | Status | Notes |
|---|---|---|
| `crates/freshblu-core` | ✅ Keep as-is | Types, permissions, auth utils — all correct |
| `crates/freshblu-store/src/store.rs` | ✅ Keep trait | The `DeviceStore` async trait is the right abstraction |
| `crates/freshblu-store/src/sqlite.rs` | ⚠️ Dev only | Fine for local dev, replace with Postgres for prod |
| `crates/freshblu-server/src/handlers/` | ✅ Keep | HTTP handler logic is correct — just swap the message bus |
| `crates/freshblu-server/src/hub.rs` | ❌ Replace | `DashMap<Uuid, broadcast::Sender>` doesn't scale past one process |
| `crates/freshblu-server/src/ws.rs` | ⚠️ Revise | Keep WS protocol logic, replace hub calls with NATS publishes |
| `crates/freshblu-cli` | ✅ Keep as-is | Full meshblu-util compatible CLI |
| `crates/freshblu-wasm` | ✅ Keep as-is | WASM HTTP client, build with wasm-pack |
| `sdks/js` | ✅ Keep as-is | TS SDK, works in browser + Node |
| `sdks/python` | ✅ Keep as-is | Python SDK, sync + async + WebSocket |
| `examples/` | ✅ Keep | Good integration test material |

---

## The Core Problem with the Scaffold's Message Bus

The scaffold's `MessageHub` is a `DashMap<Uuid, broadcast::Sender<DeviceEvent>>`.
This works perfectly on one machine. It breaks when you run two gateway pods because
Device A connected to Pod 1 can't receive messages routed by Pod 2.

The fix: **make the gateway stateless**. Every message event goes through NATS.
Subscription fanout happens in a separate worker. Gateways only hold connections.

---

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
                    │  1. Auth (Redis cache → Postgres) │
                    │  2. Permission check (cached)    │
                    │  3. Publish event → NATS         │
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
              │  PostgreSQL  — device registry, subs, tokens  │
              │  Redis       — auth cache, presence, rate lim │
              │  NATS KV     — device last-known state        │
              └───────────────────────────────────────────────┘
```

---

## NATS JetStream — Why Not Kafka, Why Not Redis Streams

**Kafka** is correct if you need infinite replay and downstream analytics pipelines.
The ops overhead is real: broker management, partition rebalancing, consumer group lag
monitoring. Overkill for a messaging platform.

**Redis Streams** is the "just ship it" option. Simple, you probably already have Redis.
Bottleneck: single-threaded per shard, persistence requires careful AOF config.
Fine to ~100k msg/sec on a single stream.

**NATS JetStream** — recommended:
- Single binary, no ZooKeeper, ~20MB, clusters via Raft
- Built-in pub/sub + persistent streams + KV store + object store
- At-least-once delivery with ack/nack built in
- Consumer groups for competing router workers
- NATS subject hierarchy maps 1:1 to the Meshblu device/event model
- 10-20M msg/sec on modest hardware
- The KV store replaces Redis for device state

One NATS JetStream cluster handles message routing, presence, device state, AND
can replace Redis for most caching needs. Fewer moving parts.

---

## Crate Plan (what to build)

### Keep from scaffold, minimal changes
- `freshblu-core` — no changes
- `freshblu-cli` — no changes
- `freshblu-wasm` — no changes
- `sdks/` — no changes

### Replace `freshblu-store`

```
freshblu-store/
  src/
    lib.rs           -- re-export trait + impls
    store.rs         -- DeviceStore trait (keep as-is)
    postgres.rs      -- NEW: sqlx + PostgreSQL implementation
    cache.rs         -- NEW: Redis permission + auth cache layer
    sqlite.rs        -- keep for dev/test only, feature-flagged
```

**PostgreSQL schema:**
```sql
CREATE TABLE devices (
    uuid        UUID PRIMARY KEY,
    data        JSONB NOT NULL,          -- full device document
    online      BOOLEAN DEFAULT FALSE,
    created_at  TIMESTAMPTZ NOT NULL,
    updated_at  TIMESTAMPTZ
);
CREATE INDEX idx_devices_type ON devices ((data->>'type'));
CREATE INDEX idx_devices_online ON devices (online) WHERE online = true;

CREATE TABLE tokens (
    id          BIGSERIAL PRIMARY KEY,
    device_uuid UUID NOT NULL REFERENCES devices(uuid) ON DELETE CASCADE,
    hash        TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL,
    expires_on  BIGINT,
    tag         TEXT
);
CREATE INDEX idx_tokens_device ON tokens(device_uuid);

CREATE TABLE subscriptions (
    emitter_uuid      UUID NOT NULL,
    subscriber_uuid   UUID NOT NULL,
    subscription_type TEXT NOT NULL,
    PRIMARY KEY (emitter_uuid, subscriber_uuid, subscription_type)
);
CREATE INDEX idx_subs_emitter ON subscriptions(emitter_uuid, subscription_type);
CREATE INDEX idx_subs_subscriber ON subscriptions(subscriber_uuid);
```

**Redis cache keys:**
```
freshblu:auth:{uuid}          → bcrypt hash (TTL 5m, invalidate on token revoke)
freshblu:perms:{uuid}         → serialized Whitelists JSON (TTL 60s, invalidate on update)
freshblu:presence:{uuid}      → "{pod-id}" (TTL 30s, refreshed on heartbeat)
freshblu:ratelimit:{uuid}     → message count (TTL 1s sliding window)
```

### Rename + split `freshblu-server` → `freshblu-gateway`

```
freshblu-gateway/
  src/
    main.rs          -- startup: connect to NATS, Postgres, Redis; start Axum
    config.rs        -- env-based config (keep, minor changes)
    lib.rs           -- router (keep handlers, replace hub)
    handlers/        -- keep all handler files, change message routing:
                        instead of hub.deliver() → nats.publish()
    ws.rs            -- keep WS protocol, replace hub with NATS
    mqtt.rs          -- expand: full rumqttd integration with NATS bridge
    nats.rs          -- NEW: NATS client wrapper, subject helpers
    presence.rs      -- NEW: presence tracking via Redis/NATS KV
    auth_cache.rs    -- NEW: Redis-backed auth + permission cache
```

**Key change in message handlers** — instead of:
```rust
// scaffold (single-process)
state.hub.deliver(&target_uuid, DeviceEvent::Message(msg));
```
Do:
```rust
// production (NATS)
state.nats.publish(
    format!("freshblu.device.{}.inbox", target_uuid),
    serde_json::to_vec(&msg)?
).await?;
```

### New crate: `freshblu-router`

```
freshblu-router/
  src/
    main.rs          -- startup: connect NATS, Postgres, Redis
    consumer.rs      -- NATS JetStream consumer (competing consumers)
    fanout.rs        -- subscription resolution + delivery routing
    presence.rs      -- shared with gateway via freshblu-core
```

**Router loop (pseudocode):**
```rust
// Consume from freshblu.broadcast.>
while let Some(msg) = consumer.next().await {
    let emitter_uuid = parse_subject(&msg.subject);
    
    // Get subscribers from Postgres (or cache)
    let subscribers = store.get_subscribers(&emitter_uuid, BroadcastSent).await?;
    
    for subscriber_uuid in subscribers {
        // Which gateway pod is this subscriber on?
        let pod_id = redis.get(format!("freshblu:presence:{}", subscriber_uuid)).await?;
        
        if let Some(pod) = pod_id {
            // Route to that pod's delivery topic
            nats.publish(
                format!("freshblu.delivery.{}", pod),
                DeliveryEnvelope { target: subscriber_uuid, event: msg.payload }
            ).await?;
        } else {
            // Subscriber offline — store in JetStream inbox for later delivery
            // or drop, based on device QoS setting
        }
    }
    
    msg.ack().await?;
}
```

### New crate: `freshblu-proto`

Shared message types between gateway and router. Avoids duplicating structs.

```
freshblu-proto/
  src/
    lib.rs
    subjects.rs      -- subject name helpers (type-safe)
    envelope.rs      -- DeliveryEnvelope, NatsEvent types
```

---

## Message Delivery Guarantees

Three modes, set per-device in `meshblu.qos`:

| Mode | Behavior | How |
|---|---|---|
| `fire-and-forget` (default) | Meshblu original — drop if offline | NATS core pub/sub, no persistence |
| `at-least-once` | Retry until ack, TTL-based expiry | NATS JetStream with ack, inbox queue |
| `state` | Store value, device reads on connect | NATS KV, device polls on ready |

`state` mode is how the original Meshblu "set the device state directly" feature worked —
you set `device.color = "green"` and the device reads it whenever it connects, regardless
of whether it was online when you set it.

---

## Presence Tracking

```
On WS connect + auth success:
  redis.setex("freshblu:presence:{uuid}", 30, "{pod-id}")
  nats_kv.put("presence.{uuid}", "{pod-id}")

On heartbeat (every 15s):
  redis.expire("freshblu:presence:{uuid}", 30)

On WS close:
  redis.del("freshblu:presence:{uuid}")
  nats_kv.delete("presence.{uuid}")

On pod crash (no clean disconnect):
  Redis TTL expires naturally after 30s
  NATS KV TTL expires naturally
```

---

## Deployment

### Local dev (zero deps, from scaffold)

```bash
cargo run --bin freshblu-server
# SQLite, in-memory hub, everything in one process
# Good for: local dev, testing, single-user deployments
```

### Docker Compose (full stack)

```yaml
# docker/docker-compose.prod.yml
services:
  nats:
    image: nats:2.10-alpine
    command: --js --cluster_name freshblu
    ports: ["4222:4222", "8222:8222"]

  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_DB: freshblu
      POSTGRES_USER: freshblu
      POSTGRES_PASSWORD: "${POSTGRES_PASSWORD}"
    volumes: [pg_data:/var/lib/postgresql/data]

  redis:
    image: redis:7-alpine
    volumes: [redis_data:/data]

  gateway:
    image: freshblu/gateway:latest
    environment:
      NATS_URL: nats://nats:4222
      DATABASE_URL: postgresql://freshblu:${POSTGRES_PASSWORD}@postgres/freshblu
      REDIS_URL: redis://redis:6379
      FRESHBLU_HTTP_PORT: "3000"
      FRESHBLU_PEPPER: "${FRESHBLU_PEPPER}"
    ports: ["3000:3000", "1883:1883"]
    depends_on: [nats, postgres, redis]
    deploy:
      replicas: 2

  router:
    image: freshblu/router:latest
    environment:
      NATS_URL: nats://nats:4222
      DATABASE_URL: postgresql://freshblu:${POSTGRES_PASSWORD}@postgres/freshblu
      REDIS_URL: redis://redis:6379
    depends_on: [nats, postgres, redis]
    deploy:
      replicas: 2
```

### Kubernetes

```yaml
# Horizontal pod autoscaler for gateways based on connection count
# Horizontal pod autoscaler for routers based on NATS consumer lag
# Both are fully stateless — scale in/out with zero coordination

# Suggested starting sizes:
# gateway:  2 replicas min, scale on CPU/connections
# router:   2 replicas min, scale on NATS consumer lag metric
# NATS:     3 nodes for JetStream raft quorum
# Postgres: primary + 1 read replica (or managed: RDS, Supabase, Neon)
# Redis:    single node fine, or managed (Upstash, ElastiCache)
```

---

## Build Order

1. **Get scaffold running locally** — `cargo run --bin freshblu-server`, verify all HTTP endpoints work with `freshblu` CLI
2. **Add Postgres backend** — implement `postgres.rs` in `freshblu-store`, feature-flag SQLite as `#[cfg(feature = "sqlite")]`
3. **Add Redis cache** — auth cache, permission cache, wrap `DeviceStore` in a `CachedStore` decorator
4. **Add NATS client to gateway** — swap `hub.deliver()` calls with `nats.publish()`, gateway subscribes to its delivery topic
5. **Build router worker** — NATS consumer, subscription fanout, presence lookup, delivery publish
6. **Add `freshblu-proto`** — extract shared subject helpers and envelope types
7. **MQTT integration** — rumqttd embedded broker, bridge to NATS subjects
8. **Observability** — Prometheus metrics on gateway (connections, msg/sec, latency), router (fanout lag, delivery rate), expose via `/metrics`

---

## Key Dependencies (add to workspace)

```toml
# NATS
async-nats = "0.33"

# PostgreSQL  
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-rustls", "uuid", "json", "migrate"] }

# Redis
redis = { version = "0.24", features = ["tokio-comp", "connection-manager"] }

# Metrics
prometheus = "0.13"
axum-prometheus = "0.6"

# MQTT broker (embedded)
rumqttd = "0.19"
```

---

## What FreshBlu is NOT trying to be

- Not a Kafka replacement or general-purpose event streaming platform
- Not a database — device state is light JSON, not time-series data
- Not a flow engine — Octoblu's visual automation layer is a separate concern
- Not a cloud service — the point is self-hostable, single-binary when you want it, clustered when you need it

The scope is: **device identity + permissioned messaging + subscriptions**, done simply and correctly, deployable on a $5 VPS or a 100-node Kubernetes cluster.
