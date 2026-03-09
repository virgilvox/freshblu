# freshblu-proto

Shared protocol types and NATS subject helpers for the FreshBlu IoT messaging platform.

## Overview

This crate defines the wire format used between FreshBlu components when communicating over NATS:

- **NATS Subject Helpers** -- Functions that produce standardized subject strings for device inboxes, broadcasts, config updates, presence, and pod delivery.
- **DeliveryEnvelope** -- The envelope type routed between gateway pods and the router via NATS.
- **NatsEvent** -- Tagged enum of all event types transported over NATS (message, broadcast, config update, unregister).

## Usage

```rust
use freshblu_proto::{subjects, DeliveryEnvelope, NatsEvent};
use uuid::Uuid;

let device_id = Uuid::new_v4();

// Build NATS subjects
let inbox = subjects::device_inbox(&device_id);   // freshblu.device.{uuid}.inbox
let bcast = subjects::broadcast(&device_id);       // freshblu.broadcast.{uuid}
let config = subjects::configure(&device_id);      // freshblu.configure.{uuid}
let deliver = subjects::delivery("pod-west-1");    // freshblu.delivery.pod-west-1

// Create a delivery envelope
let envelope = DeliveryEnvelope {
    target: device_id,
    event: NatsEvent::Unregister { uuid: device_id },
    source_pod: "pod-east-1".to_string(),
};

let json = serde_json::to_vec(&envelope).unwrap();
```

## NATS Subject Map

| Subject | Description |
|---------|-------------|
| `freshblu.device.{uuid}.inbox` | Direct messages to a device |
| `freshblu.broadcast.{uuid}` | Broadcast events from a device |
| `freshblu.configure.{uuid}` | Config update events for a device |
| `freshblu.presence.{uuid}` | Presence updates |
| `freshblu.delivery.{pod_id}` | Delivery envelopes to a gateway pod |
| `freshblu.system.unregister` | System-wide unregister events |

## NatsEvent Variants

```rust
enum NatsEvent {
    Message { from: Uuid, msg: Message },
    Broadcast { from: Uuid, msg: Message },
    ConfigUpdate { uuid: Uuid, device: Box<DeviceView> },
    Unregister { uuid: Uuid },
}
```

Serialized as JSON with a `"type"` tag field (serde `#[serde(tag = "type", rename_all = "snake_case")]`).

## License

MIT OR Apache-2.0
