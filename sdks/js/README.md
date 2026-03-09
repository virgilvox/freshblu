# freshblu

TypeScript/JavaScript client SDK for the [FreshBlu](https://github.com/virgilvox/freshblu) IoT messaging platform. Meshblu-compatible. Works in browser and Node.js.

## Install

```bash
npm install freshblu
```

For MQTT support (Node.js only):

```bash
npm install freshblu mqtt
```

## Quick Start

```ts
import { FreshBlu } from 'freshblu';

const client = new FreshBlu('https://api.freshblu.org');

// Register a device
const device = await client.register({ type: 'sensor' });
client.setCredentials(device.uuid, device.token);

// Connect WebSocket for real-time events
await client.connect();

client.on('message', (msg) => {
  console.log('Message from', msg.fromUuid, msg.payload);
});

// Send a message
client.sendMessage(['target-uuid'], { temperature: 22.5 });
```

## HTTP-Only Client

If you don't need WebSocket, use `FreshBluHttp` for a lighter import:

```ts
import { FreshBluHttp } from 'freshblu';

const client = new FreshBluHttp('https://api.freshblu.org');

const device = await client.register({ type: 'gateway' });
client.setCredentials(device.uuid, device.token);

const me = await client.whoami();
```

## API

### Constructor

```ts
// URL string
const client = new FreshBlu('https://api.freshblu.org');

// Options object (legacy Meshblu-compatible)
const client = new FreshBlu({
  hostname: 'api.freshblu.org',
  port: 443,
  secure: true,
  uuid: 'device-uuid',
  token: 'device-token',
});
```

### HTTP Methods

All methods return Promises.

| Method | Description |
|--------|-------------|
| `register(properties?)` | Register a new device. Returns device with plaintext token. |
| `whoami()` | Get authenticated device info. |
| `getDevice(uuid, asUuid?)` | Get a device by UUID. |
| `updateDevice(uuid, properties)` | Update device properties. |
| `unregister(uuid)` | Delete a device. |
| `search(query?)` | Search for devices. |
| `myDevices()` | Get devices owned by the authenticated device. |
| `claimDevice(uuid)` | Claim an unclaimed device. |
| `message({ devices, payload?, topic? })` | Send a message to specific devices. |
| `broadcast({ payload?, topic? })` | Broadcast to all subscribers. |
| `createSubscription({ subscriberUuid, emitterUuid, type })` | Subscribe to a device's events. |
| `deleteSubscription(subscriberUuid, emitterUuid, type)` | Remove a subscription. |
| `subscriptions(subscriberUuid)` | List subscriptions for a device. |
| `generateToken(uuid, opts?)` | Generate a new auth token. |
| `revokeToken(uuid, token)` | Revoke a token. |
| `resetToken(uuid)` | Revoke all tokens and return a new one. |
| `status()` | Get server health status (no auth required). |
| `setCredentials(uuid, token)` | Set auth credentials after construction. |

### WebSocket Methods

`FreshBlu` extends `FreshBluHttp` with WebSocket support.

```ts
await client.connect();       // Connect and authenticate
client.connected;              // true when authenticated

client.on('message', handler); // Listen for events
client.on('*', handler);       // Wildcard listener
client.off('message', handler);

client.sendMessage(['uuid'], { temp: 22 });  // Send message
client.sendMessage({ devices: ['uuid'], payload: { temp: 22 } });

client.subscribeWs(emitterUuid, 'broadcast.sent');
client.updateWs({ name: 'new-name' });
client.whoamiWs();

client.disconnect();
```

### Events

| Event | Payload | Description |
|-------|---------|-------------|
| `ready` | `{ uuid, fromUuid, meshblu }` | WebSocket authenticated |
| `notReady` | `{ reason }` | Authentication failed |
| `message` | `Message` | Direct message received |
| `broadcast` | `Message` | Broadcast received |
| `config` | `{ device: Device }` | Device config updated |
| `unregistered` | `{ uuid }` | Device was unregistered |

### Subscription Types

```ts
'broadcast.sent' | 'broadcast.received'
'configure.sent' | 'configure.received'
'message.sent'   | 'message.received'
'unregister.sent' | 'unregister.received'
```

## Browser (CDN)

```html
<script src="https://cdn.jsdelivr.net/npm/freshblu/dist/index.global.js"></script>
<script>
  const client = new FreshBlu.FreshBlu('https://api.freshblu.org');
</script>
```

## Builds

| Export | Format | Path |
|--------|--------|------|
| CommonJS | `require('freshblu')` | `dist/index.js` |
| ESM | `import from 'freshblu'` | `dist/index.mjs` |
| Browser global | `<script>` tag | `dist/index.global.js` |
| Types | TypeScript | `dist/index.d.ts` |

## License

MIT
