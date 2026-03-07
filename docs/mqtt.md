# MQTT Protocol

FreshBlu includes an embedded MQTT broker (rumqttd) for IoT device connectivity.

## Connection

- **Default port:** 1883
- **Protocol:** MQTT v4 (v3.1.1)
- **Authentication:** Required

```
Host: localhost
Port: 1883
Username: <device-uuid>
Password: <device-token>
```

## Authentication

MQTT authentication uses device credentials:

- **Username:** The device's UUID
- **Password:** A valid token for that device

```bash
mosquitto_pub -h localhost -p 1883 \
  -u "550e8400-e29b-41d4-a716-446655440000" \
  -P "abc123tokenhere" \
  -t "550e8400-e29b-41d4-a716-446655440000/message" \
  -m '{"devices":["target-uuid"],"payload":{"temp":72}}'
```

## Topic Format

Topics follow the pattern: `<device-uuid>/<event-type>`

| Topic | Description |
|---|---|
| `{uuid}/message` | Send a direct message to specific devices |
| `{uuid}/broadcast` | Broadcast to all subscribers |

The `{uuid}` in the topic is the **sender's** UUID.

## Message Format

### Direct Message (`{uuid}/message`)

Publish a JSON payload with `devices` and optional `payload`/`topic`:

```json
{
  "devices": ["target-device-uuid"],
  "payload": {"temperature": 72.4, "unit": "F"},
  "topic": "sensor-reading"
}
```

Permission check: each target device's `message.from` whitelist must include the sender.

### Broadcast (`{uuid}/broadcast`)

Publish any JSON value -- it will be wrapped as a broadcast message:

```json
{"temperature": 72.4, "unit": "F", "timestamp": 1704067200}
```

This is delivered to all devices subscribed to the sender's `broadcast.sent` events.

## Message Routing

When a message is published to `{sender-uuid}/message`:

1. The payload is parsed as `SendMessageParams`
2. For each target device in `devices`:
   - Permission check: sender must be in target's `message.from` whitelist
   - If allowed: message is delivered via the MessageBus
   - If denied: message is dropped with a warning log
3. If `devices` contains `"*"`: treated as a broadcast
   - Delivered to all `broadcast.sent` subscribers of the sender

## Configuration

| Environment Variable | Default | Description |
|---|---|---|
| `FRESHBLU_MQTT_PORT` | `1883` | MQTT broker listen port |

## Broker Settings

- Max connections: 10,000
- Max payload size: 1 MB
- Connection timeout: 5 seconds
- Max inflight messages: 100
- Dynamic filters: enabled

## Integration with MessageBus

The MQTT adapter bridges between the rumqttd broker and FreshBlu's internal MessageBus:

1. A bridge link subscribes to all topics (`#`)
2. Incoming publishes are parsed and routed through the bus
3. Permission checks are enforced before delivery
4. Events from other transports (HTTP, WebSocket) can also be received by MQTT clients via bus subscriptions
