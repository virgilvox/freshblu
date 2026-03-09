# FreshBlu

A high-performance reimplementation of Meshblu/Octoblu in Rust.

FreshBlu is a cross-protocol IoT machine-to-machine messaging platform. It is API-compatible with the original Meshblu, deploys as a single binary with zero dependencies (SQLite default), and scales horizontally with NATS, PostgreSQL, and Redis.

Public instance available at `https://api.freshblu.org`

## What is Meshblu?

Meshblu (originally SkyNet.im, later Citrix Octoblu) was an IoT backbone that treated every device, service, or API as a "device" with a UUID. Any device could message any other device regardless of protocol. The original was Node.js + MongoDB + Redis. FreshBlu keeps every API and concept, rewrites the internals in Rust.

## Architecture

| Crate | Description | Package |
|---|---|---|
| [freshblu-core](crates/freshblu-core) | Core types: Device, Message, Permissions, Subscriptions | [crates.io](https://crates.io/crates/freshblu-core) |
| [freshblu-proto](crates/freshblu-proto) | NATS subject helpers + wire types | [crates.io](https://crates.io/crates/freshblu-proto) |
| [freshblu-store](crates/freshblu-store) | Storage trait + SQLite, PostgreSQL, Redis cache backends | [crates.io](https://crates.io/crates/freshblu-store) |
| [freshblu-server](crates/freshblu-server) | HTTP/WS/MQTT server (axum + rumqttd), MessageBus trait | [crates.io](https://crates.io/crates/freshblu-server) |
| [freshblu-router](crates/freshblu-router) | NATS consumer + subscription fanout worker | [crates.io](https://crates.io/crates/freshblu-router) |
| [freshblu-cli](crates/freshblu-cli) | Command-line client (meshblu-util compatible) | [crates.io](https://crates.io/crates/freshblu-cli) |
| [freshblu-wasm](crates/freshblu-wasm) | Browser/Node.js WASM client | [npm](https://www.npmjs.com/package/freshblu-wasm) |

```
┌───────────────────────────────────────────────────────────┐
│                    Protocol Layer                          │
│  HTTP/REST   WebSocket (native)   MQTT (port 1883)        │
└───────────────────┬───────────────────────────────────────┘
                    │
┌───────────────────▼───────────────────────────────────────┐
│                 FreshBlu Core                               │
│                                                            │
│  Device Registry  │  Permission Engine  │  MessageBus      │
│                   │                     │                  │
│  UUID + Token     │  Whitelists v2.0    │  LocalBus (dev)  │
│  (bcrypt)         │  Per-operation      │  NatsBus (prod)  │
│                   │  per-direction      │                  │
└───────────────────┬───────────────────────────────────────┘
                    │
┌───────────────────▼───────────────────────────────────────┐
│                 Storage Layer (pluggable)                  │
│  SQLite (default)  │  PostgreSQL  │  Redis (cache layer)  │
└───────────────────────────────────────────────────────────┘
```

## Quick Start

### Run from source

```bash
cargo run --bin freshblu-server
# HTTP on :3000, WebSocket on :3000/ws, MQTT on :1883
```

### Docker

```bash
docker build -f docker/Dockerfile -t freshblu .
docker run -p 3000:3000 -p 1883:1883 -v freshblu-data:/data freshblu
```

### Docker Compose (single-process)

```bash
docker compose up
```

### Scaled deployment

Uses NATS for cross-pod messaging, PostgreSQL for storage, and Redis for caching and presence.

```bash
docker compose -f docker/docker-compose.prod.yml up
# 2x gateway pods + 2x router workers + NATS + PostgreSQL + Redis
```

### DigitalOcean Droplet

Single-droplet deployment with Caddy (auto-HTTPS), PostgreSQL, Redis, and NATS on a DO Block Storage volume. See [deploy/digitalocean/README.md](deploy/digitalocean/README.md).

### Custom config via env

```bash
DATABASE_URL=sqlite:/tmp/myapp.db \
FRESHBLU_HTTP_PORT=8080 \
FRESHBLU_PEPPER=my-secret \
FRESHBLU_OPEN_REGISTRATION=false \
freshblu-server
```

## Configuration

| Variable | Default | Description |
|---|---|---|
| `FRESHBLU_HTTP_PORT` | `3000` | HTTP and WebSocket port |
| `FRESHBLU_MQTT_PORT` | `1883` | MQTT broker port |
| `DATABASE_URL` | `sqlite:freshblu.db` | SQLite or PostgreSQL connection string |
| `NATS_URL` | (unset) | Set to enable NatsBus for cross-pod messaging (e.g. `nats://localhost:4222`) |
| `REDIS_URL` | (unset) | Set to enable Redis cache layer and presence tracking |
| `FRESHBLU_PEPPER` | `change-me-in-production` | Bcrypt pepper for token hashing |
| `FRESHBLU_OPEN_REGISTRATION` | `true` | Allow unauthenticated device registration |
| `FRESHBLU_MAX_MESSAGE_SIZE` | `1048576` | Max message payload in bytes (1MB) |
| `RUST_LOG` | `info` | Log level (`trace`, `debug`, `info`, `warn`, `error`) |

## HTTP API (Meshblu-compatible)

All authenticated endpoints use HTTP Basic Auth: `Authorization: Basic base64(uuid:token)`.

### Device Management

| Method | Path | Description |
|---|---|---|
| `POST` | `/devices` | Register a new device |
| `GET` | `/devices/:uuid` | Get device by UUID |
| `PUT` | `/devices/:uuid` | Update device properties |
| `DELETE` | `/devices/:uuid` | Unregister device |
| `POST` | `/devices/search` | Search devices by properties |
| `GET` | `/whoami` | Get authenticated device |
| `GET` | `/mydevices` | Get devices owned by auth device |

### Messaging

| Method | Path | Description |
|---|---|---|
| `POST` | `/messages` | Send message to device(s) or broadcast |

### Subscriptions

| Method | Path | Description |
|---|---|---|
| `POST` | `/devices/:uuid/subscriptions` | Create subscription |
| `GET` | `/devices/:uuid/subscriptions` | List subscriptions |
| `DELETE` | `/devices/:uuid/subscriptions/:emitter/:type` | Delete subscription |

### Token Management

| Method | Path | Description |
|---|---|---|
| `POST` | `/devices/:uuid/tokens` | Generate a new token |
| `DELETE` | `/devices/:uuid/tokens/:token` | Revoke a token |

### Utility

| Method | Path | Description |
|---|---|---|
| `GET` | `/status` | Server health check |
| `GET` | `/metrics` | Prometheus metrics |
| `POST` | `/authenticate` | Verify credentials |

V2/V3 aliases are available: `/v2/devices/:uuid`, `/v2/whoami`, `/v2/messages`, `/v3/devices/:uuid`.

## WebSocket API

Connect to `ws://hostname:3000/ws` or `ws://hostname:3000/socket.io`.

### Events (client to server)

```json
{"event": "identity", "uuid": "...", "token": "..."}
{"event": "message", "devices": ["*"], "payload": {"temp": 72}}
{"event": "subscribe", "emitterUuid": "...", "type": "broadcast.sent"}
{"event": "unsubscribe", "emitterUuid": "...", "type": "broadcast.sent"}
{"event": "update", "firmware": "2.0"}
{"event": "whoami"}
{"event": "register", "type": "sensor"}
{"event": "unregister", "uuid": "..."}
{"event": "ping"}
```

### Events (server to client)

```json
{"event": "ready", "uuid": "...", "meshblu": {...}}
{"event": "notReady", "reason": "unauthorized"}
{"event": "message", "devices": ["..."], "fromUuid": "...", "payload": {...}}
{"event": "broadcast", "devices": ["*"], "fromUuid": "...", "payload": {...}}
{"event": "config", "device": {"uuid": "...", ...}}
{"event": "unregistered", "uuid": "..."}
{"event": "pong"}
{"event": "error", "message": "..."}
```

## MQTT

Auth: `username=device-uuid`, `password=device-token`.

| Topic | Direction | Description |
|---|---|---|
| `{uuid}/message` | Publish | Send a message (JSON body with `devices` + `payload`) |
| `{uuid}/broadcast` | Publish | Broadcast to subscribers |

## Permission System (v2.0)

Every device has a `meshblu.whitelists` block. The special UUID `"*"` means everyone is allowed. An empty whitelist means nobody except the device itself is allowed.

```json
{
  "meshblu": {
    "whitelists": {
      "discover":   {"view": [{"uuid": "*"}], "as": []},
      "configure":  {"update": [{"uuid": "owner-uuid"}], "sent": [{"uuid": "*"}], "received": [], "as": []},
      "message":    {"from": [{"uuid": "*"}], "sent": [{"uuid": "*"}], "received": [], "as": []},
      "broadcast":  {"sent": [{"uuid": "*"}], "received": [], "as": []}
    }
  }
}
```

| Category | Sub-type | Controls |
|---|---|---|
| `discover` | `view` | Who can GET this device |
| `discover` | `as` | Who can act as this device for discovery |
| `configure` | `update` | Who can PUT/DELETE this device |
| `configure` | `sent` | Who can subscribe to config-change events from this device |
| `configure` | `received` | Who can see config-change events sent to this device |
| `configure` | `as` | Who can act as this device for configure ops |
| `message` | `from` | Who can send messages TO this device |
| `message` | `sent` | Who can subscribe to messages SENT BY this device |
| `message` | `received` | Who can subscribe to messages RECEIVED by this device |
| `message` | `as` | Who can send messages pretending to be this device |
| `broadcast` | `sent` | Who can subscribe to broadcasts FROM this device |
| `broadcast` | `received` | Who can subscribe to broadcasts received BY this device |
| `broadcast` | `as` | Who can broadcast pretending to be this device |

### Acting as another device

Add the `x-meshblu-as: <uuid>` header to act as another device. The actor must have the appropriate `as` permission on the target device's whitelist.

See [docs/permissions.md](docs/permissions.md) for the full reference.

## Subscription Types

| Type | Description |
|---|---|
| `broadcast.sent` | Broadcasts sent FROM the emitter |
| `broadcast.received` | Broadcasts the emitter receives |
| `message.sent` | Direct messages sent BY the emitter |
| `message.received` | Direct messages received BY the emitter |
| `configure.sent` | Config-update events sent FROM the emitter |
| `configure.received` | Config-update events sent TO the emitter |
| `unregister.sent` | Emitter unregistered itself |
| `unregister.received` | Someone unregistered the emitter |

## CLI

```bash
cargo install freshblu-cli

freshblu register
freshblu whoami
freshblu get <uuid>
freshblu update <uuid> -d '{"firmware":"2.0"}'
freshblu message -d '{"devices":["*"],"payload":{"temp":72}}'
freshblu subscribe <emitter-uuid> broadcast.sent
freshblu token generate
freshblu token revoke <token>
freshblu status
freshblu --server http://my-freshblu:3000 whoami
```

### Embedded Server

The CLI can also run the full FreshBlu server (no separate binary needed):

```bash
cargo install freshblu-cli --features server
freshblu server --port 3000 --db sqlite:freshblu.db
```

### Node.js CLI

```bash
npx freshblu-cli status --server https://api.freshblu.org
npx freshblu-cli register --type sensor
npx freshblu-cli whoami
npx freshblu-cli message -d '{"devices":["*"],"payload":{"temp":22}}'
```

## Python SDK

```bash
pip install freshblu
```

```python
from freshblu import FreshBluHttp

client = FreshBluHttp("https://api.freshblu.org")
device = client.register({"type": "sensor"})
print(device["uuid"])

client.set_credentials(device["uuid"], device["token"])
me = client.whoami()
```

## WASM (Browser + Edge)

```bash
cd crates/freshblu-wasm
wasm-pack build --target web
```

```javascript
import init, { FreshBluConfig, FreshBluHttp } from './pkg/freshblu_wasm.js';
await init();
const config = new FreshBluConfig('localhost', 3000);
const client = new FreshBluHttp(config);
const status = await client.status();
```

## Building and Testing

```bash
# Build
cargo build --release

# Run all tests (185 passing)
cargo test --workspace

# Run stress tests (ignored by default)
cargo test -p freshblu-server stress -- --ignored

# Run MQTT broker integration tests (needs port availability)
cargo test -p freshblu-server mqtt -- --ignored

# Run benchmarks (16 across all crates)
cargo bench --workspace

# Lint
cargo clippy --workspace --exclude freshblu-wasm

# Build with specific feature flags
cargo build -p freshblu-store --features postgres
cargo build -p freshblu-store --features cache
```

## License

MIT OR Apache-2.0
