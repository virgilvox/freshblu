# freshblu

Python SDK for the [FreshBlu](https://github.com/virgilvox/freshblu) IoT messaging platform. Meshblu-compatible.

## Install

```bash
# HTTP only (uses urllib as fallback, or httpx if available)
pip install freshblu

# With httpx (recommended)
pip install freshblu[http]

# With WebSocket support
pip install freshblu[ws]

# Everything
pip install freshblu[all]
```

Requires Python 3.8+.

## Quick Start

```python
from freshblu import FreshBluHttp

client = FreshBluHttp("https://api.freshblu.org")

# Register a device
device = client.register({"type": "temperature-sensor"})
client.set_credentials(device["uuid"], device["token"])

# Send a message
client.message({
    "devices": ["target-uuid"],
    "payload": {"temp": 72.4}
})

# Get device info
me = client.whoami()
```

## HTTP Client

```python
from freshblu import FreshBluHttp

client = FreshBluHttp("https://api.freshblu.org")
# or
client = FreshBluHttp(hostname="api.freshblu.org", port=443, secure=True)
```

Works as a context manager:

```python
with FreshBluHttp("https://api.freshblu.org") as client:
    device = client.register({"type": "sensor"})
    client.set_credentials(device["uuid"], device["token"])
    print(client.whoami())
```

### Methods

| Method | Description |
|--------|-------------|
| `register(properties?)` | Register a new device. Returns dict with `uuid` and `token`. |
| `whoami()` | Get authenticated device info. |
| `get_device(uuid, as_uuid?)` | Get a device by UUID. |
| `update_device(uuid, properties)` | Update device properties. |
| `unregister(uuid)` | Delete a device. |
| `search(query?)` | Search for devices. |
| `my_devices()` | Get devices owned by the authenticated device. |
| `claim_device(uuid)` | Claim an unclaimed device. |
| `message(msg)` | Send a message (`{"devices": [...], "payload": ...}`). |
| `broadcast(payload)` | Broadcast to all subscribers. |
| `create_subscription(subscriber_uuid, emitter_uuid, type)` | Subscribe to events. |
| `delete_subscription(subscriber_uuid, emitter_uuid, type)` | Remove a subscription. |
| `subscriptions(subscriber_uuid)` | List subscriptions. |
| `generate_token(uuid, expires_on?, tag?)` | Generate a new auth token. |
| `revoke_token(uuid, token)` | Revoke a token. |
| `reset_token(uuid)` | Revoke all tokens and return a new one. |
| `status()` | Server health check (no auth required). |
| `set_credentials(uuid, token)` | Set auth credentials. |

## WebSocket Client

Real-time messaging with event callbacks. Requires `pip install freshblu[ws]`.

```python
from freshblu import FreshBlu

client = FreshBlu("https://api.freshblu.org")

device = client.register({"type": "listener"})
client.set_credentials(device["uuid"], device["token"])

@client.on("message")
def on_message(data):
    print(f"From {data['fromUuid']}: {data['payload']}")

def on_ready():
    print("Connected!")

client.connect(callback=on_ready)

# Send over WebSocket
client.send_message({
    "devices": ["target-uuid"],
    "payload": {"hello": "world"}
})
```

The WebSocket runs in a background thread so it doesn't block your main program.

## Async Client

For asyncio applications. Requires `pip install freshblu[http]`.

```python
import asyncio
from freshblu import AsyncFreshBlu

async def main():
    async with AsyncFreshBlu("https://api.freshblu.org") as client:
        device = await client.register({"type": "async-sensor"})
        client.set_credentials(device["uuid"], device["token"])

        me = await client.whoami()
        print(me)

asyncio.run(main())
```

## Subscription Types

```python
from freshblu import SubscriptionType

SubscriptionType.BROADCAST_SENT      # "broadcast.sent"
SubscriptionType.BROADCAST_RECEIVED  # "broadcast.received"
SubscriptionType.CONFIGURE_SENT      # "configure.sent"
SubscriptionType.CONFIGURE_RECEIVED  # "configure.received"
SubscriptionType.MESSAGE_SENT        # "message.sent"
SubscriptionType.MESSAGE_RECEIVED    # "message.received"
SubscriptionType.UNREGISTER_SENT     # "unregister.sent"
SubscriptionType.UNREGISTER_RECEIVED # "unregister.received"
```

## Error Handling

```python
from freshblu import FreshBluHttp, FreshBluError

client = FreshBluHttp("https://api.freshblu.org")

try:
    client.whoami()
except FreshBluError as e:
    print(f"Error: {e}")
    print(f"HTTP status: {e.status_code}")
```

## Zero-Dependency Mode

The SDK works without any external packages using Python's built-in `urllib`. Install `httpx` for connection pooling, HTTP/2, and async support.

## License

MIT
