# freshblu-router

NATS-based event router for FreshBlu horizontal scaling. Consumes device events from NATS, resolves subscriptions, and delivers envelopes to the correct gateway pod.

## Overview

In a multi-pod FreshBlu deployment, gateway pods publish events to NATS subjects. The router consumes these events, looks up which devices are subscribed, finds which pod each subscriber is connected to (via Redis presence), and publishes `DeliveryEnvelope`s to the target pod's delivery subject.

```
Gateway Pod A                              Gateway Pod B
    │                                           ▲
    │ publish to NATS                           │ delivery envelope
    ▼                                           │
┌─────────────────────────────────────────────────┐
│                    NATS                          │
└────────────────────┬────────────────────────────┘
                     │
              ┌──────▼──────┐
              │   Router    │
              │             │
              │ 1. Consume  │
              │ 2. Resolve  │──▶ PostgreSQL (subscriptions)
              │ 3. Lookup   │──▶ Redis (presence: device → pod)
              │ 4. Deliver  │──▶ NATS (freshblu.delivery.{pod})
              └─────────────┘
```

## Running

```bash
# Required
export DATABASE_URL=postgres://user:pass@localhost/freshblu
export NATS_URL=nats://localhost:4222

# Optional (defaults shown)
export REDIS_URL=redis://localhost:6379
export RUST_LOG=freshblu=info

cargo run --bin freshblu-router
```

Or with Docker:

```bash
docker compose -f docker/docker-compose.prod.yml up router
```

## Event Routing

| NATS Subject | Routing Behavior |
|--------------|-----------------|
| `freshblu.device.{uuid}.inbox` | Direct delivery to target device's pod |
| `freshblu.broadcast.{uuid}` | Fan out to all `broadcast.sent` subscribers |
| `freshblu.configure.{uuid}` | Deliver to device + fan out to `configure.sent` subscribers |
| `freshblu.system.unregister` | Fan out to `unregister.sent` subscribers |

## Dependencies

- **PostgreSQL** -- Subscription storage (via `freshblu-store` with `postgres` feature)
- **Redis** -- Device presence lookups + cached store layer
- **NATS** -- Event transport

## License

MIT OR Apache-2.0
