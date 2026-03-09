<script lang="ts">
  import CodeBlock from '$lib/components/ui/CodeBlock.svelte';
</script>

<svelte:head>
  <title>Python Client - FreshBlu</title>
</svelte:head>

<div class="doc-page">
  <h1 class="doc-title">Python Client</h1>
  <p>FreshBlu provides a Python SDK with sync, async, and WebSocket clients. Works with Python 3.8+.</p>

  <h2>Installation</h2>
  <h3>HTTP only (no dependencies)</h3>
  <CodeBlock code={`pip install freshblu`} lang="bash" />

  <h3>With WebSocket support</h3>
  <CodeBlock code={`pip install freshblu[ws]`} lang="bash" />

  <h3>Everything</h3>
  <CodeBlock code={`pip install freshblu[all]`} lang="bash" />

  <p>The HTTP client works with zero dependencies using <code>urllib</code>. Install <code>httpx</code> for connection pooling and better performance. WebSocket support requires <code>websockets</code>.</p>

  <h2>REST Client API</h2>
  <p>The <code>FreshBluHttp</code> class wraps all HTTP endpoints. Use <code>FreshBlu</code> if you also need WebSocket support (it extends <code>FreshBluHttp</code>).</p>

  <h3>Constructor</h3>
  <p>Pass a URL string or keyword arguments:</p>
  <CodeBlock code={`from freshblu import FreshBluHttp

# URL string (recommended)
client = FreshBluHttp("https://api.freshblu.org")

# Keyword arguments
client = FreshBluHttp(hostname="api.freshblu.org", port=443, secure=True)

# Default: https://api.freshblu.org
client = FreshBluHttp()`} lang="python" />

  <h3>Authentication</h3>
  <CodeBlock code={`client.set_credentials(uuid, token)
me = client.whoami()`} lang="python" />

  <h3>Devices</h3>
  <CodeBlock code={`# Register a new device
device = client.register({"type": "sensor", "name": "temp-01"})
# device = {"uuid": "...", "token": "...", "online": False, "meshblu": {...}}

# Get a device
device = client.get_device(uuid)

# Get device as another device (proxy)
device = client.get_device(uuid, as_uuid="proxy-uuid")

# Update device properties
updated = client.update_device(uuid, {"name": "new-name"})

# Delete a device
client.unregister(uuid)

# Get authenticated device
me = client.whoami()

# List owned devices
mine = client.my_devices()

# Search devices
results = client.search({"type": "sensor"})

# Claim a device
claimed = client.claim_device(uuid)`} lang="python" />

  <h3>Messages</h3>
  <CodeBlock code={`# Send to specific devices
client.message({
    "devices": ["target-uuid"],
    "topic": "temperature",
    "payload": {"value": 22.5}
})

# Broadcast to subscribers
client.broadcast({
    "topic": "alert",
    "payload": {"level": "warning"}
})`} lang="python" />

  <h3>Subscriptions</h3>
  <CodeBlock code={`from freshblu import SubscriptionType

# Subscribe to events from another device
client.create_subscription(
    subscriber_uuid=my_uuid,
    emitter_uuid=emitter_uuid,
    subscription_type=SubscriptionType.MESSAGE_RECEIVED
)

# Or use a string
client.create_subscription(
    subscriber_uuid=my_uuid,
    emitter_uuid=emitter_uuid,
    subscription_type="message.received"
)

# List subscriptions
subs = client.subscriptions(my_uuid)

# Delete a subscription
client.delete_subscription(my_uuid, emitter_uuid, "message.received")`} lang="python" />

  <h3>Tokens</h3>
  <CodeBlock code={`# Generate a new token
result = client.generate_token(device_uuid)
# result = {"token": "..."}

# Generate with options
result = client.generate_token(device_uuid, tag="ci-runner", expires_on=1700000000)

# Revoke a specific token
client.revoke_token(device_uuid, token_to_revoke)

# Reset token (invalidates all previous tokens)
result = client.reset_token(device_uuid)
# result = {"uuid": "...", "token": "..."}`} lang="python" />

  <h3>Context Manager</h3>
  <CodeBlock code={`with FreshBluHttp("https://api.freshblu.org") as client:
    device = client.register({"type": "sensor"})
    client.set_credentials(device["uuid"], device["token"])
    client.message({"devices": ["*"], "payload": {"temp": 22.5}})
# connection is closed automatically`} lang="python" />

  <h2>WebSocket Client API</h2>
  <p>The <code>FreshBlu</code> class extends <code>FreshBluHttp</code> with real-time WebSocket support. Requires <code>pip install freshblu[ws]</code>.</p>

  <h3>Constructor &amp; Connection</h3>
  <CodeBlock code={`from freshblu import FreshBlu

client = FreshBlu("https://api.freshblu.org")
client.set_credentials(uuid, token)

# Connect (spawns a background thread, authenticates automatically)
client.connect()

# With a callback on ready
client.connect(callback=lambda: print("connected!"))`} lang="python" />

  <h3>Event Handling</h3>
  <CodeBlock code={`# Listen to specific events
client.on("message", lambda event: print(f"From: {event['fromUuid']}, Payload: {event['payload']}"))
client.on("broadcast", lambda event: print(event))
client.on("config", lambda event: print(event))
client.on("ready", lambda event: print("authenticated"))

# Chaining
client.on("message", handler_a).on("broadcast", handler_b)`} lang="python" />

  <h3>Sending Messages</h3>
  <CodeBlock code={`client.send_message({
    "devices": ["target-uuid"],
    "payload": {"temp": 22.5}
})

# Subscribe via WebSocket
client.subscribe_ws("emitter-uuid", "broadcast.sent")`} lang="python" />

  <h3>Cleanup</h3>
  <CodeBlock code={`client.disconnect()`} lang="python" />

  <h2>Async Client</h2>
  <p><code>AsyncFreshBlu</code> provides the same HTTP API using <code>async</code>/<code>await</code>. Requires <code>httpx</code>.</p>
  <CodeBlock code={`from freshblu import AsyncFreshBlu

async with AsyncFreshBlu("https://api.freshblu.org") as client:
    device = await client.register({"type": "sensor"})
    client.set_credentials(device["uuid"], device["token"])
    await client.message({
        "devices": ["*"],
        "payload": {"status": "online"}
    })
    results = await client.search({"type": "sensor"})
    status = await client.status()`} lang="python" />

  <h2>Subscription Types</h2>
  <p>The <code>SubscriptionType</code> enum provides all 8 types:</p>
  <CodeBlock code={`from freshblu import SubscriptionType

SubscriptionType.BROADCAST_SENT      # "broadcast.sent"
SubscriptionType.BROADCAST_RECEIVED  # "broadcast.received"
SubscriptionType.MESSAGE_SENT        # "message.sent"
SubscriptionType.MESSAGE_RECEIVED    # "message.received"
SubscriptionType.CONFIGURE_SENT      # "configure.sent"
SubscriptionType.CONFIGURE_RECEIVED  # "configure.received"
SubscriptionType.UNREGISTER_SENT     # "unregister.sent"
SubscriptionType.UNREGISTER_RECEIVED # "unregister.received"`} lang="python" />

  <h2>Error Handling</h2>
  <CodeBlock code={`from freshblu import FreshBluHttp, FreshBluError

client = FreshBluHttp("https://api.freshblu.org")

try:
    device = client.get_device("nonexistent-uuid")
except FreshBluError as e:
    print(f"Error: {e}")
    print(f"Status code: {e.status_code}")  # 404`} lang="python" />

  <h2>Complete Example</h2>
  <CodeBlock code={`from freshblu import FreshBluHttp, FreshBlu

SERVER = "https://api.freshblu.org"

# Register two devices (no auth needed for registration)
http = FreshBluHttp(SERVER)
device_a = http.register({"type": "sender"})
device_b = http.register({"type": "receiver"})

# Device B subscribes to messages from Device A
http.set_credentials(device_b["uuid"], device_b["token"])
http.create_subscription(
    subscriber_uuid=device_b["uuid"],
    emitter_uuid=device_a["uuid"],
    subscription_type="message.received"
)

# Device B connects via WebSocket
ws = FreshBlu(SERVER)
ws.set_credentials(device_b["uuid"], device_b["token"])
ws.on("message", lambda event: print(f"Received: {event['payload']}"))
ws.connect()

# Device A sends a message to Device B
sender = FreshBluHttp(SERVER)
sender.set_credentials(device_a["uuid"], device_a["token"])
sender.message({
    "devices": [device_b["uuid"]],
    "payload": {"hello": "world"}
})`} lang="python" />
</div>

<style>
  .doc-page {
    max-width: 760px;
  }
  .doc-title {
    font-family: var(--font-display);
    font-size: var(--text-3xl);
    font-weight: 700;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    margin-bottom: 16px;
  }
  h2 {
    font-family: var(--font-display);
    font-size: var(--text-xl);
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    margin: 32px 0 12px;
    padding-top: 16px;
    border-top: 1px solid var(--border);
  }
  h3 {
    font-family: var(--font-display);
    font-size: var(--text-base);
    font-weight: 700;
    letter-spacing: 0.06em;
    margin: 24px 0 8px;
  }
  p {
    font-size: var(--text-sm);
    color: var(--ink-soft);
    line-height: var(--leading-relaxed);
    margin-bottom: 12px;
  }
  code {
    font-family: var(--font-body);
    font-size: 0.9em;
    color: var(--pulse);
    background: var(--void-lift);
    padding: 1px 5px;
  }
</style>
