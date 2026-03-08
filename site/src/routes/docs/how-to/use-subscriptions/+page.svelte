<script lang="ts">
  import CodeBlock from '$lib/components/ui/CodeBlock.svelte';
</script>

<svelte:head><title>Use Subscriptions - FreshBlu Docs</title></svelte:head>

<div class="doc-page">
  <h1 class="doc-title">Use Subscriptions</h1>
  <p class="doc-intro">Create subscriptions to receive events from other devices. Subscriptions are the mechanism that connects devices in the FreshBlu event system.</p>

  <h2>What a Subscription Does</h2>
  <p>A subscription tells FreshBlu: "deliver events of type X from device A to device B." Device B is the subscriber. Device A is the emitter. When the emitter produces the subscribed event type, FreshBlu routes a copy to the subscriber.</p>

  <h2>Create a Subscription</h2>
  <p>POST to <code>/devices/:subscriber_uuid/subscriptions</code> with the emitter UUID and subscription type.</p>
  <CodeBlock lang="bash" code={`CREDS=$(echo -n "SUBSCRIBER_UUID:SUBSCRIBER_TOKEN" | base64)

curl -X POST http://localhost:3000/devices/SUBSCRIBER_UUID/subscriptions \\
  -H "Authorization: Basic $CREDS" \\
  -H "Content-Type: application/json" \\
  -d '{
    "emitterUuid": "EMITTER_UUID",
    "subscriberUuid": "SUBSCRIBER_UUID",
    "type": "broadcast-sent"
  }'`} />
  <p>Response:</p>
  <CodeBlock lang="json" code={`{
  "emitterUuid": "EMITTER_UUID",
  "subscriberUuid": "SUBSCRIBER_UUID",
  "subscriptionType": "broadcast-sent"
}`} />

  <h2>Subscription Types</h2>
  <p>There are 8 subscription types, each mapping to a specific event:</p>
  <ul>
    <li><code>broadcast-sent</code> - broadcasts sent FROM the emitter</li>
    <li><code>broadcast-received</code> - broadcasts received BY the emitter</li>
    <li><code>message-sent</code> - direct messages sent FROM the emitter</li>
    <li><code>message-received</code> - direct messages received BY the emitter</li>
    <li><code>configure-sent</code> - config changes made BY the emitter</li>
    <li><code>configure-received</code> - config changes made TO the emitter</li>
    <li><code>unregister-sent</code> - the emitter unregisters itself</li>
    <li><code>unregister-received</code> - the emitter is unregistered by another device</li>
  </ul>

  <h2>Permission Requirements</h2>
  <p>Creating a subscription requires permission on the emitter's whitelist. The required permission depends on the subscription type:</p>
  <ul>
    <li><code>broadcast-sent</code> requires <code>broadcast.sent</code> whitelist entry</li>
    <li><code>message-sent</code> requires <code>message.sent</code> whitelist entry</li>
    <li><code>configure-sent</code> requires <code>configure.sent</code> whitelist entry</li>
    <li><code>unregister-sent</code> / <code>unregister-received</code> require <code>discover.view</code> whitelist entry</li>
  </ul>
  <p>If the emitter has your UUID in the matching whitelist (or uses <code>*</code>), the subscription is created. Otherwise you get a <code>403</code>.</p>

  <h2>Subscribe to Broadcasts</h2>
  <p>The most common use case. Device B wants to receive all broadcasts from device A.</p>
  <CodeBlock lang="bash" code={`# Device B subscribes to Device A's broadcasts
curl -X POST http://localhost:3000/devices/DEVICE_B/subscriptions \\
  -H "Authorization: Basic $B_CREDS" \\
  -H "Content-Type: application/json" \\
  -d '{
    "emitterUuid": "DEVICE_A",
    "subscriberUuid": "DEVICE_B",
    "type": "broadcast-sent"
  }'`} />
  <p>Now when Device A sends a broadcast, Device B receives it over its WebSocket or MQTT connection.</p>

  <h2>Monitor Config Changes</h2>
  <p>Subscribe to <code>configure-sent</code> to get notified when a device updates its configuration.</p>
  <CodeBlock lang="bash" code={`curl -X POST http://localhost:3000/devices/MONITOR_UUID/subscriptions \\
  -H "Authorization: Basic $MONITOR_CREDS" \\
  -H "Content-Type: application/json" \\
  -d '{
    "emitterUuid": "WATCHED_DEVICE",
    "subscriberUuid": "MONITOR_UUID",
    "type": "configure-sent"
  }'`} />

  <h2>Watch for Unregistration</h2>
  <p>Subscribe to <code>unregister-sent</code> to know when a device is removed.</p>
  <CodeBlock lang="bash" code={`curl -X POST http://localhost:3000/devices/WATCHER_UUID/subscriptions \\
  -H "Authorization: Basic $WATCHER_CREDS" \\
  -H "Content-Type: application/json" \\
  -d '{
    "emitterUuid": "TARGET_UUID",
    "subscriberUuid": "WATCHER_UUID",
    "type": "unregister-sent"
  }'`} />

  <h2>List Subscriptions</h2>
  <p>GET a device's subscriptions to see what it is listening to.</p>
  <CodeBlock lang="bash" code={`curl http://localhost:3000/devices/MY_UUID/subscriptions \\
  -H "Authorization: Basic $CREDS"`} />

  <h2>Delete a Subscription</h2>
  <p>DELETE by specifying the emitter UUID and subscription type in the path. Use the dot-separated type with dots replaced by hyphens.</p>
  <CodeBlock lang="bash" code={`curl -X DELETE http://localhost:3000/devices/SUBSCRIBER_UUID/subscriptions/EMITTER_UUID/broadcast-sent \\
  -H "Authorization: Basic $CREDS"`} />
  <p>Response:</p>
  <CodeBlock lang="json" code={`{"deleted": true}`} />

  <h2>Subscription for Another Device</h2>
  <p>You can create subscriptions on behalf of another device if you have <code>configure.update</code> permission on that device.</p>
  <CodeBlock lang="bash" code={`# Admin creates a subscription for SENSOR_UUID
curl -X POST http://localhost:3000/devices/SENSOR_UUID/subscriptions \\
  -H "Authorization: Basic $ADMIN_CREDS" \\
  -H "Content-Type: application/json" \\
  -d '{
    "emitterUuid": "GATEWAY_UUID",
    "subscriberUuid": "SENSOR_UUID",
    "type": "broadcast-sent"
  }'`} />
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
