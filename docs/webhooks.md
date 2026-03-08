# Webhooks (Forwarders)

FreshBlu supports outbound HTTP webhooks via the forwarder system. Forwarders are configured per-device in the `meshblu.forwarders` block and fire automatically when matching events occur.

## Forwarder Structure

Forwarders are stored on the device document under `meshblu.forwarders`. Each event category has its own array:

```json
{
  "meshblu": {
    "forwarders": {
      "message.sent": [...],
      "message.received": [...],
      "broadcast.sent": [...],
      "broadcast.received": [...],
      "configure.sent": [...],
      "configure.received": [...],
      "unregister.sent": [...],
      "unregister.received": [...]
    }
  }
}
```

## Forwarder Types

### webhook

Sends an HTTP POST to an external URL when the event fires.

```json
{
  "type": "webhook",
  "url": "https://example.com/hook",
  "method": "POST",
  "generateAndForwardMeshbluCredentials": false
}
```

### meshblu

Re-emits the event as a message back into the FreshBlu bus. Used for event chaining.

```json
{
  "type": "meshblu",
  "devices": ["target-uuid"],
  "payload": { "forwarded": true }
}
```

## Event Categories

| Category | Fires When |
|---|---|
| `message.sent` | This device sends a direct message |
| `message.received` | This device receives a direct message |
| `broadcast.sent` | This device sends a broadcast |
| `broadcast.received` | This device receives a broadcast |
| `configure.sent` | This device's config is updated |
| `configure.received` | A config update is directed at this device |
| `unregister.sent` | This device is deleted |
| `unregister.received` | This device's unregistration is observed |

## Request Format

Webhook-type forwarders send an HTTP request with:

**Headers:**
- `Content-Type: application/json`
- `X-Meshblu-Uuid` - the UUID of the device that owns the forwarder

**Body:**
The full event payload as JSON, including `fromUuid`, `devices`, `payload`, and any other fields from the original event.

## Credential Forwarding

When `generateAndForwardMeshbluCredentials` is set to `true`, the webhook request includes the device's auth credentials in the headers:

- `meshblu_auth_uuid` - the device UUID
- `meshblu_auth_token` - a valid token for the device

This allows the receiving endpoint to make authenticated calls back to FreshBlu on behalf of the device.

## SSRF Protections and Limits

- **Max forwarders per event category:** 10. Entries beyond the cap are ignored.
- **HTTP timeout:** 10 seconds per webhook request.
- **Blocked destinations:** Private/internal IP ranges (127.0.0.0/8, 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16, etc.) are blocked to prevent SSRF.
- **Fire and forget:** Failed webhooks are logged and counted in Prometheus metrics (`WEBHOOKS_FAILED`) but do not retry.

## Loop Prevention

- **Depth limit:** Meshblu-type forwarding chains stop at depth 5.
- **Cycle detection:** If a device UUID appears twice in the forwarding chain, the loop is broken.

## Configuration Example

Configure forwarders via `PUT /devices/:uuid`:

```bash
CREDS=$(echo -n "$UUID:$TOKEN" | base64)

curl -X PUT https://api.freshblu.org/devices/$UUID \
  -H "Authorization: Basic $CREDS" \
  -H "Content-Type: application/json" \
  -d '{
    "meshblu": {
      "forwarders": {
        "message.received": [
          {
            "type": "webhook",
            "url": "https://example.com/on-message",
            "method": "POST"
          }
        ],
        "configure.sent": [
          {
            "type": "webhook",
            "url": "https://example.com/on-config-change",
            "method": "POST"
          }
        ]
      }
    }
  }'
```

## Execution Model

- Forwarders fire in a spawned task and do not block the event pipeline or HTTP response.
- All webhook-type forwarders for a single event fire concurrently.
- Meshblu-type forwarders execute sequentially because they mutate bus state.
