# Permission Model

FreshBlu implements the Meshblu v2.0 whitelist-based permission system.

## Overview

Every device has a `meshblu.whitelists` block that controls who can perform specific operations. Permissions are checked on the **target** device's whitelists  - the device being acted upon controls who has access.

### Key Rules

1. A device can always perform any operation on **itself** (self-access is always allowed)
2. The special UUID `"*"` in a whitelist means **anyone** is allowed
3. An **empty** whitelist means nobody (except self) is allowed
4. Newly registered devices get **open** whitelists by default (all `"*"`)

## Whitelist Structure

```json
{
  "meshblu": {
    "whitelists": {
      "discover": {
        "view": [{"uuid": "*"}],
        "as": []
      },
      "configure": {
        "update": [{"uuid": "owner-uuid"}],
        "sent": [{"uuid": "*"}],
        "received": [],
        "as": []
      },
      "message": {
        "from": [{"uuid": "*"}],
        "sent": [{"uuid": "*"}],
        "received": [],
        "as": []
      },
      "broadcast": {
        "sent": [{"uuid": "*"}],
        "received": [],
        "as": []
      }
    }
  }
}
```

## Permission Categories

### discover

Controls who can see this device.

| Sub-type | Operation | Description |
|---|---|---|
| `view` | `GET /devices/:uuid` | Who can retrieve this device's properties |
| `as` | `x-meshblu-as` header | Who can act as this device for discovery operations |

When `discover.view` denies access, the server returns 404 (not 403) to avoid leaking device existence.

### configure

Controls who can modify this device.

| Sub-type | Operation | Description |
|---|---|---|
| `update` | `PUT /devices/:uuid`, `DELETE /devices/:uuid` | Who can update or delete this device |
| `sent` | Subscribe to `configure.sent` | Who can subscribe to config-change events FROM this device |
| `received` | Subscribe to `configure.received` | Who can subscribe to config-change events sent TO this device |
| `as` | `x-meshblu-as` header | Who can act as this device for configure operations |

### message

Controls direct messaging to/from this device.

| Sub-type | Operation | Description |
|---|---|---|
| `from` | `POST /messages` | Who can send messages TO this device |
| `sent` | Subscribe to `message.sent` | Who can subscribe to messages SENT BY this device |
| `received` | Subscribe to `message.received` | Who can subscribe to messages RECEIVED by this device |
| `as` | `x-meshblu-as` header + `POST /messages` | Who can send messages pretending to be this device |

### broadcast

Controls broadcast subscriptions.

| Sub-type | Operation | Description |
|---|---|---|
| `sent` | Subscribe to `broadcast.sent` | Who can subscribe to broadcasts FROM this device |
| `received` | Subscribe to `broadcast.received` | Who can subscribe to broadcasts received BY this device |
| `as` | `x-meshblu-as` header | Who can broadcast pretending to be this device |

## Acting As Another Device (x-meshblu-as)

The `x-meshblu-as` HTTP header allows a device to perform operations on behalf of another device. This is checked in two steps:

1. **as permission**: The actor must be in the target device's appropriate `as` whitelist
2. **operation permission**: The target device (being acted as) must have permission for the actual operation

Example: Device A wants to act as Device B to read Device C:
- Device B's `discover.as` whitelist must include Device A
- Device C's `discover.view` whitelist must include Device B

## Preset Whitelist Configurations

### Open (default for new devices)

Everyone can do everything:

```json
{
  "discover": {"view": [{"uuid": "*"}], "as": []},
  "configure": {"update": [{"uuid": "*"}], "sent": [{"uuid": "*"}], "received": [{"uuid": "*"}], "as": []},
  "message": {"from": [{"uuid": "*"}], "sent": [{"uuid": "*"}], "received": [{"uuid": "*"}], "as": []},
  "broadcast": {"sent": [{"uuid": "*"}], "received": [{"uuid": "*"}], "as": []}
}
```

### Private (locked down)

Only a specific owner UUID has access:

```json
{
  "discover": {"view": [{"uuid": "owner-uuid"}], "as": []},
  "configure": {"update": [{"uuid": "owner-uuid"}], "sent": [{"uuid": "owner-uuid"}], "received": [{"uuid": "owner-uuid"}], "as": []},
  "message": {"from": [{"uuid": "owner-uuid"}], "sent": [{"uuid": "owner-uuid"}], "received": [{"uuid": "owner-uuid"}], "as": []},
  "broadcast": {"sent": [{"uuid": "owner-uuid"}], "received": [{"uuid": "owner-uuid"}], "as": []}
}
```

### Setting permissions at registration

```bash
curl -X POST http://localhost:3000/devices \
  -H 'Content-Type: application/json' \
  -d '{
    "type": "secure-device",
    "meshblu": {
      "whitelists": {
        "discover": {"view": [{"uuid": "admin-uuid"}], "as": []},
        "configure": {"update": [{"uuid": "admin-uuid"}], "sent": [], "received": [], "as": []},
        "message": {"from": [{"uuid": "*"}], "sent": [], "received": [], "as": []},
        "broadcast": {"sent": [], "received": [], "as": []}
      }
    }
  }'
```

## Permission Check Flow

```
Request arrives
    │
    ├─ Extract actor UUID from auth credentials
    ├─ Extract as_uuid from x-meshblu-as header (if present)
    │
    ├─ If as_uuid present:
    │   └─ Check as_device.whitelists.<category>.as includes actor
    │      └─ If denied: return 403
    │
    ├─ Determine effective_actor = as_uuid or actor.uuid
    │
    ├─ Fetch target device
    │
    └─ Check target.whitelists.<category>.<sub_type> includes effective_actor
       ├─ If self (effective_actor == target): always allowed
       ├─ If in whitelist: allowed
       └─ If not in whitelist: denied (403 or 404 for discover)
```
