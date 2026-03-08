<script lang="ts">
  import CodeBlock from '$lib/components/ui/CodeBlock.svelte';
</script>

<svelte:head><title>Send Messages - FreshBlu Docs</title></svelte:head>

<div class="doc-page">
  <h1 class="doc-title">Send Messages</h1>
  <p class="doc-intro">Send direct messages to specific devices and broadcasts to all subscribers. Works over HTTP and WebSocket.</p>

  <h2>Direct Message via HTTP</h2>
  <p>POST to <code>/messages</code> with a <code>devices</code> array of target UUIDs and a freeform <code>payload</code>.</p>
  <CodeBlock lang="bash" code={`CREDS=$(echo -n "SENDER_UUID:SENDER_TOKEN" | base64)

curl -X POST http://localhost:3000/messages \\
  -H "Authorization: Basic $CREDS" \\
  -H "Content-Type: application/json" \\
  -d '{
    "devices": ["TARGET_UUID"],
    "payload": {"temperature": 22.5, "unit": "celsius"}
  }'`} />
  <p>Response on success:</p>
  <CodeBlock lang="json" code={`{"sent": true}`} />
  <p>The target must have your UUID in its <code>message.from</code> whitelist, or the message is silently dropped.</p>

  <h2>Send to Multiple Devices</h2>
  <p>List multiple UUIDs in the <code>devices</code> array. Each target is checked independently for permissions.</p>
  <CodeBlock lang="bash" code={`curl -X POST http://localhost:3000/messages \\
  -H "Authorization: Basic $CREDS" \\
  -H "Content-Type: application/json" \\
  -d '{
    "devices": ["UUID_A", "UUID_B", "UUID_C"],
    "payload": {"alert": "firmware update available"}
  }'`} />

  <h2>Include a Topic</h2>
  <p>The optional <code>topic</code> field lets receivers filter or route messages.</p>
  <CodeBlock lang="bash" code={`curl -X POST http://localhost:3000/messages \\
  -H "Authorization: Basic $CREDS" \\
  -H "Content-Type: application/json" \\
  -d '{
    "devices": ["TARGET_UUID"],
    "topic": "sensor-reading",
    "payload": {"humidity": 45}
  }'`} />

  <h2>Broadcast via HTTP</h2>
  <p>A broadcast delivers to all devices that have a <code>broadcast.sent</code> subscription to the sender. Two equivalent methods:</p>

  <h3>Option 1: POST /broadcasts</h3>
  <CodeBlock lang="bash" code={`curl -X POST http://localhost:3000/broadcasts \\
  -H "Authorization: Basic $CREDS" \\
  -H "Content-Type: application/json" \\
  -d '{
    "payload": {"status": "online", "battery": 87}
  }'`} />

  <h3>Option 2: POST /messages with wildcard</h3>
  <CodeBlock lang="bash" code={`curl -X POST http://localhost:3000/messages \\
  -H "Authorization: Basic $CREDS" \\
  -H "Content-Type: application/json" \\
  -d '{
    "devices": ["*"],
    "payload": {"status": "online", "battery": 87}
  }'`} />
  <p>Both produce the same result. The <code>/broadcasts</code> endpoint forces <code>devices</code> to <code>["*"]</code> internally.</p>

  <h2>Send Messages via WebSocket</h2>
  <p>After authenticating over WebSocket, send a <code>message</code> event frame.</p>
  <CodeBlock lang="javascript" code={`const ws = new WebSocket('ws://localhost:3000/ws');

ws.onopen = () => {
  ws.send(JSON.stringify({
    event: 'identity',
    uuid: 'SENDER_UUID',
    token: 'SENDER_TOKEN'
  }));
};

ws.onmessage = (event) => {
  const msg = JSON.parse(event.data);
  if (msg.event === 'ready') {
    // send a direct message
    ws.send(JSON.stringify({
      event: 'message',
      devices: ['TARGET_UUID'],
      payload: { command: 'toggle' }
    }));
  }
};`} />

  <h3>Broadcast via WebSocket</h3>
  <CodeBlock lang="javascript" code={`ws.send(JSON.stringify({
  event: 'message',
  devices: ['*'],
  payload: { reading: 42 }
}));`} />

  <h2>Send As Another Device</h2>
  <p>Use the <code>x-meshblu-as</code> header to send a message on behalf of another device. The as-device must have your UUID in its <code>message.as</code> whitelist.</p>
  <CodeBlock lang="bash" code={`curl -X POST http://localhost:3000/messages \\
  -H "Authorization: Basic $CREDS" \\
  -H "x-meshblu-as: PROXY_DEVICE_UUID" \\
  -H "Content-Type: application/json" \\
  -d '{
    "devices": ["TARGET_UUID"],
    "payload": {"proxied": true}
  }'`} />

  <h2>Message Size Limits</h2>
  <p>The server enforces a maximum message size on the combined <code>payload</code> and extra fields. Messages exceeding the limit receive a <code>413</code> response. The default limit is configured via <code>max_message_size</code> in the server config.</p>

  <h2>Subscription Fan-out</h2>
  <p>When a direct message is delivered, FreshBlu also notifies:</p>
  <ul>
    <li><code>message.sent</code> subscribers of the sender</li>
    <li><code>message.received</code> subscribers of the target</li>
  </ul>
  <p>This allows third-party devices to monitor message traffic without being a direct participant. See <a href="/docs/how-to/use-subscriptions">subscriptions</a> for setup.</p>
</div>

<style>
  .doc-page { max-width: 740px; }
  .doc-title { font-family: var(--font-display); font-size: var(--text-2xl); font-weight: 700; letter-spacing: 0.04em; text-transform: uppercase; margin-bottom: 8px; }
  .doc-intro { font-size: var(--text-sm); color: var(--ink-soft); line-height: var(--leading-relaxed); margin-bottom: 40px; }
  h2 { font-family: var(--font-display); font-size: var(--text-lg); font-weight: 700; letter-spacing: 0.04em; margin: 40px 0 16px; padding-bottom: 8px; border-bottom: 1px solid var(--border); }
  h3 { font-family: var(--font-display); font-size: var(--text-md); font-weight: 600; margin: 24px 0 8px; }
  p { font-size: var(--text-sm); color: var(--ink-soft); line-height: var(--leading-relaxed); margin-bottom: 16px; }
  code { font-family: var(--font-body); font-size: var(--text-sm); color: var(--pulse); }
  ul, ol { font-size: var(--text-sm); color: var(--ink-soft); line-height: var(--leading-relaxed); margin-bottom: 16px; padding-left: 20px; }
  li { margin-bottom: 4px; }
</style>
