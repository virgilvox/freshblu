# REST API Reference

All authenticated endpoints require HTTP Basic Auth: `Authorization: Basic base64(uuid:token)`.

Alternatively, use legacy headers: `skynet_auth_uuid` + `skynet_auth_token`.

## Status

### `GET /status`

No auth required.

```bash
curl http://localhost:3000/status
```

```json
{"meshblu": true, "version": "2.0.0", "online": true, "connections": 5, "engine": "freshblu"}
```

## Device Registration

### `POST /devices`

Register a new device. No auth required when `FRESHBLU_OPEN_REGISTRATION=true` (default).

```bash
curl -X POST http://localhost:3000/devices \
  -H 'Content-Type: application/json' \
  -d '{"type": "sensor", "name": "temp-01"}'
```

```json
{
  "uuid": "550e8400-e29b-41d4-a716-446655440000",
  "token": "abc123...",
  "type": "sensor",
  "name": "temp-01",
  "online": false,
  "meshblu": {
    "version": "2.0.0",
    "createdAt": "2024-01-01T00:00:00Z",
    "hash": "...",
    "whitelists": { ... }
  }
}
```

With custom permissions:

```bash
curl -X POST http://localhost:3000/devices \
  -H 'Content-Type: application/json' \
  -d '{
    "type": "private-device",
    "meshblu": {
      "whitelists": {
        "discover": {"view": [{"uuid": "owner-uuid"}], "as": []},
        "configure": {"update": [{"uuid": "owner-uuid"}], "sent": [], "received": [], "as": []},
        "message": {"from": [{"uuid": "owner-uuid"}], "sent": [], "received": [], "as": []},
        "broadcast": {"sent": [], "received": [], "as": []}
      }
    }
  }'
```

## Device Operations

### `GET /devices/:uuid`

Get a device by UUID. Requires `discover.view` permission.

```bash
curl http://localhost:3000/devices/550e8400-e29b-41d4-a716-446655440000 \
  -H "Authorization: Basic $(echo -n 'my-uuid:my-token' | base64)"
```

### `PUT /devices/:uuid`

Update device properties. Requires `configure.update` permission.

```bash
curl -X PUT http://localhost:3000/devices/550e8400-e29b-41d4-a716-446655440000 \
  -H "Authorization: Basic $(echo -n 'my-uuid:my-token' | base64)" \
  -H 'Content-Type: application/json' \
  -d '{"color": "blue", "firmware": "2.1"}'
```

System fields (`uuid`, `token`, `meshblu`) cannot be overwritten via update.

### `DELETE /devices/:uuid`

Unregister a device. Requires `configure.update` permission.

```bash
curl -X DELETE http://localhost:3000/devices/550e8400-e29b-41d4-a716-446655440000 \
  -H "Authorization: Basic $(echo -n 'my-uuid:my-token' | base64)"
```

### `POST /devices/search`

Search devices by property filters. Results are filtered by `discover.view` permission.

```bash
curl -X POST http://localhost:3000/devices/search \
  -H "Authorization: Basic $(echo -n 'my-uuid:my-token' | base64)" \
  -H 'Content-Type: application/json' \
  -d '{"type": "sensor"}'
```

Results are limited to 100 devices per query.

### `GET /whoami`

Get the authenticated device's own data.

```bash
curl http://localhost:3000/whoami \
  -H "Authorization: Basic $(echo -n 'my-uuid:my-token' | base64)"
```

### `POST /authenticate`

Verify credentials without returning device data.

```bash
curl -X POST http://localhost:3000/authenticate \
  -H 'Content-Type: application/json' \
  -d '{"uuid": "my-uuid", "token": "my-token"}'
```

## Messaging

### `POST /messages`

Send a message to specific devices or broadcast to all subscribers.

Direct message:
```bash
curl -X POST http://localhost:3000/messages \
  -H "Authorization: Basic $(echo -n 'my-uuid:my-token' | base64)" \
  -H 'Content-Type: application/json' \
  -d '{"devices": ["target-uuid"], "payload": {"alert": "temperature high"}}'
```

Broadcast:
```bash
curl -X POST http://localhost:3000/messages \
  -H "Authorization: Basic $(echo -n 'my-uuid:my-token' | base64)" \
  -H 'Content-Type: application/json' \
  -d '{"devices": ["*"], "payload": {"temp": 72.4}}'
```

Permission checks:
- Direct messages: target must have sender in `message.from` whitelist
- Broadcasts: delivered to all `broadcast.sent` subscribers

## Subscriptions

### `POST /devices/:uuid/subscriptions`

Subscribe to events from an emitter device. Requires appropriate whitelist permission on the emitter.

```bash
curl -X POST http://localhost:3000/devices/my-uuid/subscriptions \
  -H "Authorization: Basic $(echo -n 'my-uuid:my-token' | base64)" \
  -H 'Content-Type: application/json' \
  -d '{"emitterUuid": "other-uuid", "subscriberUuid": "my-uuid", "type": "broadcast-sent"}'
```

### `GET /devices/:uuid/subscriptions`

List all subscriptions for a device.

```bash
curl http://localhost:3000/devices/my-uuid/subscriptions \
  -H "Authorization: Basic $(echo -n 'my-uuid:my-token' | base64)"
```

### `DELETE /devices/:uuid/subscriptions/:emitter_uuid/:type`

Delete a specific subscription. Type uses hyphens: `broadcast-sent`, `message-received`, etc.

```bash
curl -X DELETE http://localhost:3000/devices/my-uuid/subscriptions/other-uuid/broadcast-sent \
  -H "Authorization: Basic $(echo -n 'my-uuid:my-token' | base64)"
```

## Token Management

### `POST /devices/:uuid/tokens`

Generate a new authentication token. Requires `configure.update` permission.

```bash
curl -X POST http://localhost:3000/devices/my-uuid/tokens \
  -H "Authorization: Basic $(echo -n 'my-uuid:my-token' | base64)" \
  -H 'Content-Type: application/json' \
  -d '{"expiresOn": 1735689600, "tag": "session"}'
```

Body can be `null` for a token with no expiry.

### `DELETE /devices/:uuid/tokens/:token`

Revoke a specific token.

```bash
curl -X DELETE http://localhost:3000/devices/my-uuid/tokens/abc123... \
  -H "Authorization: Basic $(echo -n 'my-uuid:my-token' | base64)"
```

## Acting As Another Device

Add the `x-meshblu-as` header to perform operations as another device. The actor must have the appropriate `as` permission on the target device's whitelist.

```bash
curl http://localhost:3000/devices/target-uuid \
  -H "Authorization: Basic $(echo -n 'my-uuid:my-token' | base64)" \
  -H "x-meshblu-as: acting-as-uuid"
```

Permission required depends on the operation:
- `GET /devices/:uuid` — requires `discover.as` on the as-device
- `PUT /devices/:uuid` — requires `configure.as` on the as-device
- `DELETE /devices/:uuid` — requires `configure.as` on the as-device
- `POST /messages` — requires `message.as` on the as-device

## Error Responses

All errors return JSON:

```json
{"error": "description of the error"}
```

| Status | Meaning |
|---|---|
| 401 | Unauthorized — missing or invalid credentials |
| 403 | Forbidden — insufficient permissions |
| 404 | Not Found — device doesn't exist or no discover permission |
| 409 | Conflict — resource already exists |
| 422 | Validation Error — invalid input |
| 429 | Rate Limited |
| 500 | Internal Server Error |
