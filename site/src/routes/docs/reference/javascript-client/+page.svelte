<script lang="ts">
  import CodeBlock from '$lib/components/ui/CodeBlock.svelte';
</script>

<svelte:head>
  <title>JavaScript Client - FreshBlu</title>
</svelte:head>

<div class="doc-page">
  <h1 class="doc-title">JavaScript Client</h1>
  <p>FreshBlu provides a TypeScript/JavaScript SDK for both REST and WebSocket communication. Use it in the browser or any JavaScript runtime.</p>

  <h2>Installation</h2>
  <h3>npm / Bundler</h3>
  <CodeBlock code={`npm install freshblu`} lang="bash" />
  <CodeBlock code={`import { FreshBlu, FreshBluHttp } from 'freshblu';`} lang="javascript" />

  <h3>CDN / Script Tag</h3>
  <p>Include the SDK directly in your HTML via unpkg:</p>
  <CodeBlock code={`<script src="https://unpkg.com/freshblu@1.0.0/dist/index.global.js"><\/script>
<script>
  // FreshBlu and FreshBluHttp are available as globals
  const client = new FreshBluHttp('https://api.freshblu.org');
  const device = await client.register({ type: 'browser' });
  console.log('Registered:', device.uuid);
<\/script>`} lang="html" />

  <h2>REST Client API</h2>
  <p>The <code>FreshBluHttp</code> class wraps all HTTP endpoints with typed methods. Use <code>FreshBlu</code> if you also need WebSocket support (it extends <code>FreshBluHttp</code>).</p>

  <h3>Constructor</h3>
  <p>Pass a URL string or an options object:</p>
  <CodeBlock code={`// URL string
const client = new FreshBluHttp('https://api.freshblu.org');

// Options object
const client = new FreshBluHttp({
  hostname: 'api.freshblu.org',
  port: 443,
  secure: true,
  uuid: 'my-uuid',
  token: 'my-token'
});

// Default: http://localhost:3000
const client = new FreshBluHttp();`} lang="javascript" />

  <h3>Authentication</h3>
  <CodeBlock code={`client.setCredentials(uuid, token);
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
const results = await client.search({ type: 'sensor' });

// Claim a device
const claimed = await client.claimDevice(uuid);`} lang="javascript" />

  <h3>Messages</h3>
  <CodeBlock code={`// Send to specific devices
await client.message({
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
await client.createSubscription({
  subscriberUuid: myUuid,
  emitterUuid: emitterUuid,
  type: 'message.received'
});

// List subscriptions
const subs = await client.subscriptions(myUuid);

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
  <p>The <code>FreshBlu</code> class extends <code>FreshBluHttp</code> with real-time WebSocket support. All HTTP methods are available plus the WebSocket API below.</p>

  <h3>Constructor &amp; Connection</h3>
  <CodeBlock code={`const client = new FreshBlu('https://api.freshblu.org');
client.setCredentials(uuid, token);

// Connect (sends identity, resolves on 'ready')
await client.connect();

// Check connection status
console.log(client.connected); // boolean`} lang="javascript" />

  <h3>Event Handling</h3>
  <CodeBlock code={`// Listen to specific events
client.on('message', (event) => {
  console.log('From:', event.fromUuid);
  console.log('Payload:', event.payload);
});

client.on('broadcast', (event) => { /* ... */ });
client.on('config', (event) => { /* ... */ });
client.on('ready', (event) => { /* ... */ });
client.on('notReady', (event) => { /* ... */ });
client.on('unregistered', (event) => { /* ... */ });

// Listen to ALL events
client.on('*', (event) => {
  console.log(event.event, event);
});

// Remove a listener
client.off('message', handler);`} lang="javascript" />

  <h3>Sending Messages</h3>
  <CodeBlock code={`// Convenience method (positional args)
client.sendMessage(['target-uuid'], { temp: 22.5 }, 'readings');

// Object form
client.sendMessage({ devices: ['target-uuid'], payload: { temp: 22.5 }, topic: 'readings' });

// Raw send (any event)
client.send({ event: 'message', devices: ['*'], payload: {} });`} lang="javascript" />

  <h3>Cleanup</h3>
  <CodeBlock code={`client.close();
// or
client.disconnect();`} lang="javascript" />

  <h2>Complete Example</h2>
  <CodeBlock code={`import { FreshBlu, FreshBluHttp } from 'freshblu';

const SERVER = 'https://api.freshblu.org';

// Register two devices (no auth needed for registration)
const http = new FreshBluHttp(SERVER);
const deviceA = await http.register({ type: 'sender' });
const deviceB = await http.register({ type: 'receiver' });

// Device B subscribes to messages from Device A
const clientB = new FreshBluHttp(SERVER);
clientB.setCredentials(deviceB.uuid, deviceB.token);
await clientB.createSubscription({
  subscriberUuid: deviceB.uuid,
  emitterUuid: deviceA.uuid,
  type: 'message.received'
});

// Device B connects via WebSocket
const wsB = new FreshBlu(SERVER);
wsB.setCredentials(deviceB.uuid, deviceB.token);
wsB.on('message', (event) => {
  console.log('Received:', event.payload);
});
await wsB.connect();

// Device A sends a message to Device B
const clientA = new FreshBluHttp(SERVER);
clientA.setCredentials(deviceA.uuid, deviceA.token);
await clientA.message({
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
