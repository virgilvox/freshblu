<script lang="ts">
  import CodeBlock from '$lib/components/ui/CodeBlock.svelte';
</script>

<svelte:head>
  <title>JavaScript Client - FreshBlu</title>
</svelte:head>

<div class="doc-page">
  <h1 class="doc-title">JavaScript Client</h1>
  <p>FreshBlu provides a TypeScript/JavaScript client for both REST and WebSocket communication. Use it in the browser or any JavaScript runtime.</p>

  <h2>Browser (ES Module)</h2>
  <p>Include the client directly in your HTML:</p>
  <CodeBlock code={`<script type="module">
  import { FreshBluClient } from '/path/to/client.js';

  const client = new FreshBluClient('http://localhost:3000');
  const device = await client.register({ type: 'browser' });
  console.log('Registered:', device.uuid);
</script>`} lang="javascript" />

  <h2>npm / Bundler</h2>
  <p>Import the client in your project:</p>
  <CodeBlock code={`import { FreshBluClient } from './lib/api/client';
import { FreshBluWs } from './lib/api/ws';`} lang="javascript" />

  <h2>REST Client API</h2>
  <p>The <code>FreshBluClient</code> class wraps all HTTP endpoints with typed methods.</p>

  <h3>Constructor</h3>
  <CodeBlock code={`const client = new FreshBluClient(baseUrl?: string);
// defaults to PUBLIC_API_URL or http://localhost:3000`} lang="javascript" />

  <h3>Authentication</h3>
  <CodeBlock code={`client.setCredentials(uuid: string, token: string);
const device = await client.authenticate();`} lang="javascript" />

  <h3>Devices</h3>
  <CodeBlock code={`// Register a new device
const res = await client.register({ type: 'sensor', name: 'temp-01' });
// res = { uuid, token, online, meshblu }

// Get a device
const device = await client.getDevice(uuid);

// Update device properties
const updated = await client.updateDevice(uuid, { name: 'new-name' });

// Delete a device
await client.unregister(uuid);

// Get authenticated device
const me = await client.whoami();

// List owned devices
const mine = await client.myDevices();

// Search devices
const results = await client.searchDevices({ type: 'sensor' });

// Claim a device
const claimed = await client.claimDevice(uuid);`} lang="javascript" />

  <h3>Messages</h3>
  <CodeBlock code={`// Send to specific devices
await client.sendMessage({
  devices: ['target-uuid'],
  topic: 'temperature',
  payload: { value: 22.5 }
});

// Broadcast to subscribers
await client.broadcast({
  topic: 'alert',
  payload: { level: 'warning' }
});`} lang="javascript" />

  <h3>Subscriptions</h3>
  <CodeBlock code={`// Subscribe to events from another device
await client.createSubscription(
  myUuid,
  emitterUuid,
  'message.received'
);

// List subscriptions
const subs = await client.listSubscriptions(myUuid);

// Delete a subscription
await client.deleteSubscription(myUuid, emitterUuid, 'message.received');`} lang="javascript" />

  <h3>Tokens</h3>
  <CodeBlock code={`// Generate a new token
const { uuid, token } = await client.generateToken(deviceUuid);

// Revoke a specific token
await client.revokeToken(deviceUuid, tokenToRevoke);

// Reset token (invalidates all previous tokens)
const { uuid, token } = await client.resetToken(deviceUuid);`} lang="javascript" />

  <h2>WebSocket Client API</h2>
  <p>The <code>FreshBluWs</code> class provides real-time event streaming.</p>

  <h3>Constructor &amp; Connection</h3>
  <CodeBlock code={`const ws = new FreshBluWs(uuid, token, baseUrl?);

// Connect (sends identity, resolves on 'ready')
await ws.connect();

// Check connection status
console.log(ws.connected); // boolean`} lang="javascript" />

  <h3>Event Handling</h3>
  <CodeBlock code={`// Listen to specific events
ws.on('message', (event) => {
  console.log('From:', event.fromUuid);
  console.log('Payload:', event.payload);
});

ws.on('broadcast', (event) => { /* ... */ });
ws.on('config', (event) => { /* ... */ });
ws.on('ready', (event) => { /* ... */ });
ws.on('notReady', (event) => { /* ... */ });
ws.on('unregistered', (event) => { /* ... */ });

// Listen to ALL events
ws.on('*', (event) => {
  console.log(event.event, event);
});

// Remove a listener
ws.off('message', handler);`} lang="javascript" />

  <h3>Sending Messages</h3>
  <CodeBlock code={`// Convenience method
ws.sendMessage(['target-uuid'], { temp: 22.5 }, 'readings');

// Raw send (any event)
ws.send({ event: 'message', devices: ['*'], payload: {} });`} lang="javascript" />

  <h3>Cleanup</h3>
  <CodeBlock code="ws.close();" lang="javascript" />

  <h2>Complete Example</h2>
  <CodeBlock code={`import { FreshBluClient } from './lib/api/client';
import { FreshBluWs } from './lib/api/ws';

const SERVER = 'http://localhost:3000';

// Register two devices
const client = new FreshBluClient(SERVER);
const deviceA = await client.register({ type: 'sender' });
const deviceB = await client.register({ type: 'receiver' });

// Device B subscribes to messages from Device A
const clientB = new FreshBluClient(SERVER);
clientB.setCredentials(deviceB.uuid, deviceB.token);
await clientB.createSubscription(
  deviceB.uuid, deviceA.uuid, 'message.received'
);

// Device B connects via WebSocket
const wsB = new FreshBluWs(deviceB.uuid, deviceB.token, SERVER);
wsB.on('message', (event) => {
  console.log('Received:', event.payload);
});
await wsB.connect();

// Device A sends a message to Device B
const clientA = new FreshBluClient(SERVER);
clientA.setCredentials(deviceA.uuid, deviceA.token);
await clientA.sendMessage({
  devices: [deviceB.uuid],
  payload: { hello: 'world' }
});`} lang="javascript" />
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
