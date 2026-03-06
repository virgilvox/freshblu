/**
 * FreshBlu Node.js Example
 * 
 * Run: node examples/node/example.js
 * Requires: npm install freshblu
 * Or: npx ts-node examples/node/example.ts
 */

import { FreshBlu, FreshBluHttp, SubscriptionType } from '../../sdks/js/src/index.js';

const SERVER = process.env.FRESHBLU_SERVER || 'http://localhost:3000';

async function main() {
  console.log('=== FreshBlu Node.js Example ===\n');

  const [hostname, portStr] = SERVER.replace(/https?:\/\//, '').split(':');
  const port = parseInt(portStr || '3000', 10);

  const http = new FreshBluHttp({ hostname, port });

  // 1. Check status
  console.log('1. Checking server status...');
  const status = await http.status();
  console.log('Status:', status, '\n');

  // 2. Register two devices
  console.log('2. Registering two devices...');

  const sensorDevice = await http.register({
    type: 'temperature-sensor',
    location: 'mesa-lab',
    meshblu: {
      whitelists: {
        broadcast: { sent: [{ uuid: '*' }] },
        discover: { view: [{ uuid: '*' }] },
        configure: { update: [{ uuid: '*' }] },
        message: { from: [{ uuid: '*' }] }
      }
    }
  });

  console.log('Sensor device registered:', {
    uuid: sensorDevice.uuid,
    token: sensorDevice.token,
    type: sensorDevice.type,
  });

  const listenerDevice = await http.register({
    type: 'data-aggregator',
    name: 'aggregator-1',
  });

  console.log('Listener device registered:', {
    uuid: listenerDevice.uuid,
    token: listenerDevice.token,
  });

  // Set credentials to sensor device for subsequent calls
  http.setCredentials(sensorDevice.uuid, sensorDevice.token!);

  // 3. Subscribe listener to sensor broadcasts
  console.log('\n3. Creating subscriptions...');

  const listenerHttp = new FreshBluHttp({
    hostname, port,
    uuid: listenerDevice.uuid,
    token: listenerDevice.token,
  });

  await listenerHttp.createSubscription({
    subscriberUuid: listenerDevice.uuid,
    emitterUuid: sensorDevice.uuid,
    type: 'broadcast.sent',
  });

  console.log('Listener subscribed to sensor broadcasts');

  // Self-subscription for broadcast.received (so we can receive in WS)
  await listenerHttp.createSubscription({
    subscriberUuid: listenerDevice.uuid,
    emitterUuid: listenerDevice.uuid,
    type: 'broadcast.received',
  });

  // 4. Connect listener via WebSocket
  console.log('\n4. Connecting listener via WebSocket...');

  const listenerWs = new FreshBlu({
    hostname, port,
    uuid: listenerDevice.uuid,
    token: listenerDevice.token,
  });

  listenerWs.on('ready', (data) => {
    console.log('Listener WebSocket ready:', data.uuid);
  });

  listenerWs.on('broadcast', (msg) => {
    console.log('\nReceived broadcast:', JSON.stringify(msg, null, 2));
  });

  listenerWs.on('message', (msg) => {
    console.log('\nReceived direct message:', JSON.stringify(msg, null, 2));
  });

  listenerWs.on('config', (data) => {
    console.log('\nDevice config changed:', data.device.uuid);
  });

  await new Promise<void>((resolve) => {
    listenerWs.connect(() => {
      console.log('Listener connected!\n');
      resolve();
    });
    setTimeout(resolve, 2000); // fallback
  });

  // 5. Send messages from sensor
  console.log('5. Sending messages from sensor device...\n');

  // Broadcast (to all subscribers)
  await http.message({
    devices: ['*'],
    topic: 'temperature',
    payload: { celsius: 23.4, humidity: 55, unit: 'celsius' },
  });
  console.log('Broadcast sent: temperature reading');

  // Direct message to listener
  await http.message({
    devices: [listenerDevice.uuid],
    topic: 'direct',
    payload: { type: 'ping', timestamp: Date.now() },
  });
  console.log('Direct message sent to listener');

  await sleep(500);

  // 6. Update device
  console.log('\n6. Updating sensor device...');
  const updated = await http.updateDevice(sensorDevice.uuid, {
    firmware: '2.1.0',
    lastSeen: new Date().toISOString(),
  });
  console.log('Updated:', { uuid: updated.uuid, firmware: updated.firmware });

  // 7. Search for devices
  console.log('\n7. Searching for temperature sensors...');
  const sensors = await listenerHttp.search({ type: 'temperature-sensor' });
  console.log('Found sensors:', sensors.map((d) => ({ uuid: d.uuid, type: d.type })));

  // 8. Generate a session token
  console.log('\n8. Generating a session token...');
  const tokenRecord = await listenerHttp.generateToken(listenerDevice.uuid);
  console.log('Generated token (first 8 chars):', tokenRecord.token.substring(0, 8) + '...');

  // 9. Cleanup
  await sleep(500);
  console.log('\n9. Cleaning up...');
  listenerWs.disconnect();
  await http.unregister(sensorDevice.uuid);
  await listenerHttp.unregister(listenerDevice.uuid);
  console.log('Devices unregistered\n');

  console.log('=== Example complete ===');
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

main().catch(console.error);
