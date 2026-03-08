<script>
  import CodeBlock from '$lib/components/ui/CodeBlock.svelte';
</script>

<svelte:head><title>WebSocket Protocol - FreshBlu Docs</title></svelte:head>
<div class="doc-page">
  <h1 class="doc-title">WebSocket Protocol</h1>
  <p class="doc-intro">FreshBlu exposes a WebSocket endpoint for real-time bidirectional communication. The protocol is compatible with Meshblu Socket.io event names, so existing JavaScript clients work without modification.</p>

  <h2>Connection</h2>
  <p>Connect to <code>ws://host:port/ws</code> or <code>ws://host:port/socket.io</code>. Both endpoints are identical. No authentication headers are required at connection time. Authentication happens via the identity message after the socket is open.</p>

  <h2>Authentication Flow</h2>
  <p>After opening the WebSocket, send an <code>identity</code> message. The server responds with either <code>ready</code> or <code>notReady</code>. No other messages are processed until authentication succeeds.</p>

  <h3>1. Client sends identity</h3>
  <CodeBlock lang="json" code={`{
  "event": "identity",
  "uuid": "your-device-uuid",
  "token": "your-device-token"
}`} />

  <h3>2a. Server responds with ready</h3>
  <p>On successful authentication, the device is marked online and subscribed to its event bus channel.</p>
  <CodeBlock lang="json" code={`{
  "event": "ready",
  "uuid": "your-device-uuid",
  "fromUuid": "your-device-uuid",
  "meshblu": { ... }
}`} />

  <h3>2b. Server responds with notReady</h3>
  <p>On failure. The socket remains open and the client can retry with new credentials.</p>
  <CodeBlock lang="json" code={`{
  "event": "notReady",
  "reason": "unauthorized"
}`} />
  <p>If the UUID is malformed, the reason will be <code>"invalid uuid"</code>.</p>

  <h2>Client Messages</h2>
  <p>All messages are JSON objects with an <code>event</code> field that determines the message type.</p>

  <h3>message</h3>
  <p>Send a message to one or more devices. Requires <code>message.from</code> permission on each target. Messages exceeding <code>max_message_size</code> are silently dropped.</p>
  <CodeBlock lang="json" code={`{
  "event": "message",
  "devices": ["target-uuid"],
  "topic": "sensor-reading",
  "payload": { "temp": 22.5 }
}`} />
  <p>If <code>devices</code> contains <code>"*"</code>, the message is treated as a broadcast and delivered to all <code>broadcast.sent</code> subscribers.</p>

  <h3>update</h3>
  <p>Update the connected device's properties. Emits a <code>config</code> event to the device and all <code>configure.sent</code> subscribers. Fires <code>configure.sent</code> forwarders.</p>
  <CodeBlock lang="json" code={`{
  "event": "update",
  "color": "blue",
  "status": "active"
}`} />

  <h3>subscribe</h3>
  <p>Create a subscription to another device's events. Requires the appropriate permission on the emitter device (e.g., <code>broadcast.sent</code> permission for <code>broadcast.sent</code> subscriptions).</p>
  <CodeBlock lang="json" code={`{
  "event": "subscribe",
  "emitterUuid": "other-device-uuid",
  "type": "broadcast.sent"
}`} />
  <p>If permission is denied, the server sends an error event:</p>
  <CodeBlock lang="json" code={`{
  "event": "error",
  "message": "forbidden: insufficient permission to subscribe"
}`} />

  <h3>unsubscribe</h3>
  <p>Remove a subscription. The <code>type</code> field is optional. If omitted, all subscription types for the given emitter are removed.</p>
  <CodeBlock lang="json" code={`{
  "event": "unsubscribe",
  "emitterUuid": "other-device-uuid",
  "type": "broadcast.sent"
}`} />

  <h3>whoami</h3>
  <p>Request the connected device's current properties.</p>
  <CodeBlock lang="json" code={`{ "event": "whoami" }`} />
  <p>Server responds with:</p>
  <CodeBlock lang="json" code={`{
  "event": "whoami",
  "device": { "uuid": "...", "name": "...", ... }
}`} />

  <h3>register</h3>
  <p>Register a new device through the WebSocket connection.</p>
  <CodeBlock lang="json" code={`{
  "event": "register",
  "name": "new-device",
  "type": "sensor"
}`} />
  <p>Server responds with:</p>
  <CodeBlock lang="json" code={`{
  "event": "registered",
  "uuid": "new-device-uuid",
  "token": "plaintext-token"
}`} />
  <p>On failure, the server sends an error event with the error message.</p>

  <h3>unregister</h3>
  <p>Unregister a device. Only the currently connected device can unregister itself via WebSocket. The connection closes after unregistration.</p>
  <CodeBlock lang="json" code={`{
  "event": "unregister",
  "uuid": "your-device-uuid"
}`} />

  <h3>ping</h3>
  <p>Application-level keepalive. Works both before and after authentication.</p>
  <CodeBlock lang="json" code={`{ "event": "ping" }`} />
  <p>Server responds with:</p>
  <CodeBlock lang="json" code={`{ "event": "pong" }`} />

  <h2>Server Events</h2>
  <p>Events pushed from the server to the client after authentication.</p>

  <h3>message</h3>
  <p>A direct message from another device.</p>
  <CodeBlock lang="json" code={`{
  "event": "message",
  "devices": ["your-uuid"],
  "fromUuid": "sender-uuid",
  "topic": "sensor-reading",
  "payload": { "temp": 22.5 }
}`} />

  <h3>config</h3>
  <p>Emitted when the device's properties are updated (by itself or by another device with <code>configure.update</code> permission).</p>
  <CodeBlock lang="json" code={`{
  "event": "config",
  "device": { "uuid": "...", "name": "...", ... }
}`} />

  <h3>broadcast</h3>
  <p>A broadcast message. Received when subscribed to a device's <code>broadcast.sent</code> events.</p>
  <CodeBlock lang="json" code={`{
  "event": "broadcast",
  "devices": ["*"],
  "fromUuid": "broadcaster-uuid",
  "payload": { "status": "online" }
}`} />

  <h3>unregistered</h3>
  <p>Emitted when a device the client is subscribed to (via <code>unregister.sent</code>) is deleted.</p>
  <CodeBlock lang="json" code={`{
  "event": "unregistered",
  "uuid": "deleted-device-uuid"
}`} />

  <h2>Connection Lifecycle</h2>
  <p>On disconnect, the server decrements the connection counter, marks the device offline, and removes it from the message bus. Standard WebSocket ping/pong frames are handled automatically at the protocol level.</p>
  <p>If the client falls behind on reading events, the server logs a warning and skips the missed messages rather than buffering indefinitely.</p>
</div>

<style>
  .doc-page { max-width: 740px; }
  .doc-title { font-family: var(--font-display); font-size: var(--text-2xl); font-weight: 700; letter-spacing: 0.04em; text-transform: uppercase; margin-bottom: 8px; }
  .doc-intro { font-size: var(--text-sm); color: var(--ink-soft); line-height: var(--leading-relaxed); margin-bottom: 40px; }
  h2 { font-family: var(--font-display); font-size: var(--text-lg); font-weight: 700; letter-spacing: 0.04em; margin: 40px 0 16px; padding-bottom: 8px; border-bottom: 1px solid var(--border); }
  h3 { font-family: var(--font-display); font-size: var(--text-md); font-weight: 600; margin: 24px 0 8px; }
  p { font-size: var(--text-sm); color: var(--ink-soft); line-height: var(--leading-relaxed); margin-bottom: 16px; }
  code { font-family: var(--font-body); font-size: var(--text-sm); color: var(--pulse); }
</style>
