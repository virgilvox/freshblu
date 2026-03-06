# FreshBlu

**A modern, high-performance reimplementation of Meshblu/Octoblu in Rust.**

FreshBlu is a cross-protocol IoT machine-to-machine messaging platform. It is 100% API-compatible with the original Meshblu, single-binary deployable, SQLite-default (no deps), and compiles to WASM for browser/edge use.

---

## What is Meshblu?

Meshblu (originally SkyNet.im, later Citrix Octoblu) was the IoT backbone that treated every device, service, or API as a "device" with a UUID — like Twitter for machines. Any device could message any other device regardless of protocol. The original was Node.js + MongoDB + Redis. FreshBlu keeps every API and concept, rewrites the internals in Rust.

---

## Architecture

```
┌───────────────────────────────────────────────────────────┐
│                    Protocol Layer                          │
│  HTTP/REST   WebSocket (native)   MQTT (port 1883)        │
└───────────────────┬───────────────────────────────────────┘
                    │
┌───────────────────▼───────────────────────────────────────┐
│                 FreshBlu Core                               │
│                                                            │
│  Device Registry  │  Permission Engine  │  Message Router │
│                   │                     │                  │
│  UUID + Token     │  Whitelists v2.0    │  Pub/Sub fanout  │
│  (bcrypt)         │  Per-operation      │  Subscription    │
│                   │  per-direction      │  routing         │
└───────────────────┬───────────────────────────────────────┘
                    │
┌───────────────────▼───────────────────────────────────────┐
│                 Storage Layer (pluggable)                  │
│  SQLite (default, zero deps)  │  PostgreSQL               │
└───────────────────────────────────────────────────────────┘
```

---

## Quick Start

### Run with Docker

```bash
docker pull freshblu/freshblu:latest
docker run -p 3000:3000 -p 1883:1883 freshblu/freshblu
```

Or with docker-compose:
```bash
cd docker && docker-compose up
```

### Run from source

```bash
cargo build --release
./target/release/freshblu-server
# → HTTP on :3000, WebSocket on :3000/ws, MQTT on :1883
```

### Single binary, zero config
```bash
# Default: SQLite database in current directory
freshblu-server

# Custom config via env
DATABASE_URL=sqlite:/tmp/myapp.db \
FRESHBLU_HTTP_PORT=8080 \
FRESHBLU_PEPPER=my-secret \
freshblu-server
```

---

## CLI

```bash
# Install
cargo install freshblu-cli

# Register a device (saves uuid/token to ./freshblu.json)
freshblu register
freshblu register -d '{"type":"sensor","location":"mesa-lab"}'

# Use registered credentials
freshblu whoami
freshblu get <uuid>
freshblu update <uuid> -d '{"firmware":"2.0"}'

# Messaging
freshblu message -d '{"devices":["*"],"payload":{"temp":72}}'
freshblu message -d '{"devices":["<target-uuid>"],"topic":"alert","payload":"fire"}'

# Subscriptions
freshblu subscribe <emitter-uuid> broadcast.sent
freshblu subscribe <emitter-uuid> message.received

# Token management
freshblu token generate
freshblu token generate --expires-on 1735689600 --tag session
freshblu token revoke <token>

# Server status
freshblu status

# Target a different server
freshblu --server http://my-freshblu.example.com:3000 whoami
```

---

## HTTP API (Meshblu-compatible)

All endpoints accept Basic Auth with `uuid:token`.

### Device Management

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/devices` | Register a new device |
| `GET` | `/devices/:uuid` | Get device by UUID |
| `PUT` | `/devices/:uuid` | Update device properties |
| `DELETE` | `/devices/:uuid` | Unregister device |
| `POST` | `/devices/search` | Search devices by properties |
| `GET` | `/whoami` | Get authenticated device |
| `GET` | `/mydevices` | Get devices owned by auth device |

### Messaging

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/messages` | Send message to device(s) or broadcast |

### Subscriptions

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/devices/:uuid/subscriptions` | Create subscription |
| `GET` | `/devices/:uuid/subscriptions` | List subscriptions |
| `DELETE` | `/devices/:uuid/subscriptions/:emitter/:type` | Delete subscription |

### Token Management

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/devices/:uuid/tokens` | Generate a new token |
| `DELETE` | `/devices/:uuid/tokens/:token` | Revoke a token |

### Utility

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/status` | Server health check |

---

## WebSocket API

Connect to `ws://hostname:3000/ws`.

### Events (client → server)

```json
// Authenticate
{ "event": "identity", "uuid": "...", "token": "..." }

// Send a message
{ "event": "message", "devices": ["*"], "payload": { "temp": 72 } }

// Subscribe to events
{ "event": "subscribe", "emitterUuid": "...", "type": "broadcast.sent" }

// Whoami
{ "event": "whoami" }

// Update own device
{ "event": "update", "firmware": "2.0" }
```

### Events (server → client)

```json
// Auth successful
{ "event": "ready", "uuid": "...", "meshblu": { ... } }

// Auth failed
{ "event": "notReady", "reason": "unauthorized" }

// Message received
{ "event": "message", "devices": ["..."], "fromUuid": "...", "payload": {...} }

// Broadcast received (via subscription)
{ "event": "broadcast", "devices": ["*"], "fromUuid": "...", "payload": {...} }

// Device config changed
{ "event": "config", "device": { "uuid": "...", ... } }

// Device unregistered
{ "event": "unregistered", "uuid": "..." }
```

---

## Permission System (v2.0)

Every device has a `meshblu.whitelists` block. The special UUID `"*"` means everyone is allowed.

```json
{
  "uuid": "device-uuid",
  "meshblu": {
    "version": "2.0.0",
    "whitelists": {
      "discover":   { "view": [{"uuid": "*"}], "as": [] },
      "configure":  { "update": [{"uuid": "owner-uuid"}], "sent": [{"uuid": "*"}] },
      "message":    { "from": [{"uuid": "*"}], "sent": [{"uuid": "*"}] },
      "broadcast":  { "sent": [{"uuid": "*"}], "received": [{"uuid": "*"}] }
    }
  }
}
```

### Whitelist types

| Category | Sub-type | Controls |
|----------|----------|----------|
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

Add the `x-meshblu-as: <uuid>` header to act as another device (requires `as` permission).

---

## Subscription Types

| Type | Description |
|------|-------------|
| `broadcast.sent` | Broadcasts sent FROM the emitter |
| `broadcast.received` | Broadcasts the emitter receives (via its own subscriptions) |
| `message.sent` | Direct messages sent BY the emitter |
| `message.received` | Direct messages received BY the emitter |
| `configure.sent` | Config-update events sent FROM the emitter |
| `configure.received` | Config-update events sent TO the emitter |
| `unregister.sent` | Emitter unregistered itself |
| `unregister.received` | Someone unregistered the emitter |

### Message routing with subscriptions

```
Device A broadcasts → all devices in A's broadcast.sent whitelist
                   → each of those devices' broadcast.received subscribers

Device A messages Device B → Device B (direct delivery)
                           → devices subscribed to B's message.received
                           → devices subscribed to A's message.sent
```

---

## JavaScript / TypeScript SDK

```bash
npm install freshblu
```

```typescript
import { FreshBlu, FreshBluHttp } from 'freshblu';

// HTTP client
const http = new FreshBluHttp({ hostname: 'localhost', port: 3000 });

const device = await http.register({ type: 'my-sensor' });
http.setCredentials(device.uuid, device.token);

await http.message({ devices: ['*'], payload: { temp: 72.4 } });

// WebSocket client (real-time)
const ws = new FreshBlu({
  hostname: 'localhost', port: 3000,
  uuid: device.uuid, token: device.token,
});

ws.on('ready', () => console.log('connected'));
ws.on('message', (msg) => console.log('received:', msg));
ws.on('broadcast', (msg) => console.log('broadcast:', msg));

ws.connect();
ws.sendMessage({ devices: ['*'], payload: 'hello world' });
```

---

## Python SDK

```bash
pip install freshblu[all]
```

```python
from freshblu import FreshBlu, FreshBluHttp, SubscriptionType

client = FreshBluHttp(hostname="localhost", port=3000)

device = client.register({"type": "python-sensor"})
client.set_credentials(device["uuid"], device["token"])

client.message({"devices": ["*"], "payload": {"temp": 23.4}})
```

---

## WASM (Browser + Edge)

```bash
cd crates/freshblu-wasm
wasm-pack build --target web    # Browser
wasm-pack build --target nodejs # Node.js
wasm-pack build --target bundler # Webpack/Vite
```

```javascript
import init, { FreshBluConfig, FreshBluHttp } from './pkg/freshblu_wasm.js';

await init();

const config = new FreshBluConfig('localhost', 3000);
const client = new FreshBluHttp(config);

const status = await client.status();
console.log(status);
```

---

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `FRESHBLU_HTTP_PORT` | `3000` | HTTP server port |
| `FRESHBLU_MQTT_PORT` | `1883` | MQTT broker port |
| `DATABASE_URL` | `sqlite:freshblu.db` | Database (SQLite or PostgreSQL) |
| `FRESHBLU_PEPPER` | `change-me-in-production` | Extra secret for token security |
| `FRESHBLU_OPEN_REGISTRATION` | `true` | Allow unauthenticated device registration |
| `FRESHBLU_MAX_MESSAGE_SIZE` | `1048576` | Max message size in bytes (1MB) |
| `RUST_LOG` | `info` | Log level (`trace`, `debug`, `info`, `warn`, `error`) |

---

## PostgreSQL (Production)

```bash
DATABASE_URL=postgresql://user:pass@localhost/freshblu freshblu-server
```

---

## Migrating from Meshblu

FreshBlu is a drop-in replacement. Change your connection config:

```javascript
// Before (Meshblu)
const conn = require('meshblu')({
  hostname: 'meshblu.octoblu.com',
  port: 443,
  uuid: '...',
  token: '...'
});

// After (FreshBlu)
import { FreshBlu } from 'freshblu';
const conn = new FreshBlu({
  hostname: 'your-freshblu.example.com',
  port: 3000,
  uuid: '...',
  token: '...'
});
// All events, methods, and behavior are identical
```

---

## Building

```bash
# Server + CLI
cargo build --release

# WASM client
cargo install wasm-pack
cd crates/freshblu-wasm && wasm-pack build --target web

# JS SDK
cd sdks/js && npm install && npm run build

# Python SDK
cd sdks/python && pip install -e ".[all]"
```

---

## License

MIT
