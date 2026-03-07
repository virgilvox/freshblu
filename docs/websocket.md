# WebSocket Protocol

## Connection

Connect to `ws://hostname:3000/ws` or `ws://hostname:3000/socket.io` (compatibility endpoint).

## Authentication Flow

1. Client connects to the WebSocket endpoint
2. Client sends an `identity` message with UUID and token
3. Server responds with `ready` (success) or `notReady` (failure)
4. After `ready`, bidirectional event exchange begins

```
Client                          Server
  │                               │
  ├──── identity {uuid, token} ──►│
  │                               ├── authenticate
  │◄──── ready {uuid, meshblu} ───┤
  │                               │
  ├──── message {devices, ...} ──►│ (normal operation)
  │◄──── message {fromUuid, ...} ─┤
  │                               │
```

## Client Events (client → server)

### identity

Authenticate the WebSocket connection. Must be sent before any other event.

```json
{
  "event": "identity",
  "uuid": "550e8400-e29b-41d4-a716-446655440000",
  "token": "abc123..."
}
```

### message

Send a message to specific devices or broadcast.

```json
{
  "event": "message",
  "devices": ["target-uuid"],
  "payload": {"temperature": 72.4},
  "topic": "sensor-reading"
}
```

Broadcast to all subscribers:
```json
{
  "event": "message",
  "devices": ["*"],
  "payload": {"status": "online"}
}
```

### subscribe

Subscribe to events from another device. Requires appropriate whitelist permission on the emitter.

```json
{
  "event": "subscribe",
  "emitterUuid": "other-device-uuid",
  "type": "broadcast.sent"
}
```

Available types: `broadcast.sent`, `broadcast.received`, `message.sent`, `message.received`, `configure.sent`, `configure.received`, `unregister.sent`, `unregister.received`.

### unsubscribe

Remove a subscription.

```json
{
  "event": "unsubscribe",
  "emitterUuid": "other-device-uuid",
  "type": "broadcast.sent"
}
```

Omit `type` to unsubscribe from all event types for that emitter.

### update

Update the connected device's properties.

```json
{
  "event": "update",
  "color": "blue",
  "firmware": "2.1"
}
```

### whoami

Request the connected device's current data.

```json
{"event": "whoami"}
```

### register

Register a new device.

```json
{
  "event": "register",
  "type": "sensor",
  "name": "new-device"
}
```

### unregister

Unregister the connected device (disconnects the WebSocket).

```json
{
  "event": "unregister",
  "uuid": "my-uuid"
}
```

Only the connected device can unregister itself via WebSocket.

### ping

Keepalive ping.

```json
{"event": "ping"}
```

## Server Events (server → client)

### ready

Authentication successful.

```json
{
  "event": "ready",
  "uuid": "550e8400-e29b-41d4-a716-446655440000",
  "fromUuid": "550e8400-e29b-41d4-a716-446655440000",
  "meshblu": {
    "version": "2.0.0",
    "createdAt": "2024-01-01T00:00:00Z",
    "whitelists": { ... }
  }
}
```

### notReady

Authentication failed.

```json
{
  "event": "notReady",
  "reason": "unauthorized"
}
```

### message

A direct message was received.

```json
{
  "event": "message",
  "devices": ["my-uuid"],
  "fromUuid": "sender-uuid",
  "payload": {"hello": "world"},
  "topic": "greeting"
}
```

### broadcast

A broadcast was received via subscription.

```json
{
  "event": "broadcast",
  "devices": ["*"],
  "fromUuid": "sender-uuid",
  "payload": {"temp": 72.4}
}
```

### config

The connected device's configuration was updated (or a subscribed device's config changed).

```json
{
  "event": "config",
  "device": {
    "uuid": "...",
    "online": true,
    "type": "sensor",
    "meshblu": { ... }
  }
}
```

### unregistered

A subscribed device was unregistered.

```json
{
  "event": "unregistered",
  "uuid": "deleted-device-uuid"
}
```

### error

An operation failed (e.g., insufficient permissions for subscribe).

```json
{
  "event": "error",
  "message": "forbidden: insufficient permission to subscribe"
}
```

### pong

Response to a ping.

```json
{"event": "pong"}
```

## Connection Lifecycle

- On connect: device is marked as online in the store and bus
- On disconnect: device is marked as offline, bus connection is cleaned up
- WebSocket ping/pong (protocol-level) is handled transparently
- Application-level ping/pong uses the `ping`/`pong` events
- Slow consumers that fall behind will have missed messages dropped (broadcast channel with 256 capacity)
