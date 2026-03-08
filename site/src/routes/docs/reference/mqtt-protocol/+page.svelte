<script>
  import CodeBlock from '$lib/components/ui/CodeBlock.svelte';
</script>

<svelte:head><title>MQTT Protocol - FreshBlu Docs</title></svelte:head>
<div class="doc-page">
  <h1 class="doc-title">MQTT Protocol</h1>
  <p class="doc-intro">FreshBlu includes an embedded MQTT v3.1.1 broker (rumqttd) that bridges MQTT clients into the messaging system. Devices connect using standard MQTT clients and publish/subscribe using Meshblu-compatible topic patterns.</p>

  <h2>Connection</h2>
  <p>Connect to <code>mqtt://host:1883</code> (default port). The port is configurable via <code>FRESHBLU_MQTT_PORT</code>.</p>

  <h3>Authentication</h3>
  <p>MQTT authentication maps directly to FreshBlu device credentials:</p>
  <table class="config-table">
    <thead>
      <tr>
        <th>MQTT Field</th>
        <th>Value</th>
      </tr>
    </thead>
    <tbody>
      <tr>
        <td>Username</td>
        <td><code>device-uuid</code></td>
      </tr>
      <tr>
        <td>Password</td>
        <td><code>device-token</code></td>
      </tr>
      <tr>
        <td>Client ID</td>
        <td>Any valid MQTT client ID</td>
      </tr>
    </tbody>
  </table>
  <p>Authentication is performed against the FreshBlu device store using bcrypt verification. Invalid credentials cause the MQTT CONNECT to be rejected.</p>

  <h2>Topic Format</h2>
  <p>Topics follow the pattern <code>{'{uuid}'}/{'{event_type}'}</code> where <code>uuid</code> is the publishing device's UUID.</p>

  <table class="config-table">
    <thead>
      <tr>
        <th>Topic</th>
        <th>Description</th>
      </tr>
    </thead>
    <tbody>
      <tr>
        <td><code>{'{uuid}'}/message</code></td>
        <td>Send a direct message. Payload is a <code>SendMessageParams</code> JSON object.</td>
      </tr>
      <tr>
        <td><code>{'{uuid}'}/broadcast</code></td>
        <td>Broadcast a message. Payload is a JSON object delivered to all <code>broadcast.sent</code> subscribers.</td>
      </tr>
      <tr>
        <td><code>{'{uuid}'}/config</code></td>
        <td>Reserved for config change notifications.</td>
      </tr>
    </tbody>
  </table>

  <h2>Sending Messages</h2>
  <p>Publish to <code>{'{your-uuid}'}/message</code> with a JSON payload:</p>
  <CodeBlock lang="json" code={`{
  "devices": ["target-uuid"],
  "topic": "sensor-data",
  "payload": { "temp": 22.5 }
}`} />
  <p>Permission checks apply. The target device must have the sender in its <code>message.from</code> whitelist. Unauthorized messages are dropped with a warning log.</p>
  <p>If <code>devices</code> contains <code>"*"</code>, the message is also broadcast to all <code>broadcast.sent</code> subscribers.</p>

  <h2>Broadcasting</h2>
  <p>Publish to <code>{'{your-uuid}'}/broadcast</code> with a JSON payload:</p>
  <CodeBlock lang="json" code={`{
  "status": "online",
  "temperature": 22.5
}`} />
  <p>The payload is wrapped in a broadcast message and delivered to all devices subscribed to your <code>broadcast.sent</code> events.</p>

  <h2>Receiving Messages</h2>
  <p>Messages are delivered through the FreshBlu message bus. To receive messages over MQTT, subscribe to the MQTT topic matching your device's UUID. The broker uses dynamic filters, so wildcard subscriptions (<code>#</code>) are supported at the broker level.</p>

  <h2>Broker Configuration</h2>
  <table class="config-table">
    <thead>
      <tr>
        <th>Setting</th>
        <th>Value</th>
      </tr>
    </thead>
    <tbody>
      <tr>
        <td>Max connections</td>
        <td>10,000</td>
      </tr>
      <tr>
        <td>Max payload size</td>
        <td>1 MB (1,048,576 bytes)</td>
      </tr>
      <tr>
        <td>Max inflight messages</td>
        <td>100</td>
      </tr>
      <tr>
        <td>Connection timeout</td>
        <td>5,000 ms</td>
      </tr>
      <tr>
        <td>Protocol version</td>
        <td>MQTT v3.1.1 (v4)</td>
      </tr>
    </tbody>
  </table>

  <h2>QoS Support</h2>
  <p>The embedded broker supports QoS 0 (at most once) and QoS 1 (at least once). QoS 2 (exactly once) is not supported. The outgoing packet count limit is 200 per connection.</p>

  <h2>Bridge Architecture</h2>
  <p>The MQTT adapter creates a programmatic link into the embedded broker and subscribes to all topics via the <code>#</code> wildcard. When a client publishes to a topic, the bridge task parses the topic, applies permission checks, and forwards the message through the FreshBlu message bus. This means MQTT messages are subject to the same permission system as HTTP and WebSocket messages.</p>
</div>

<style>
  .doc-page { max-width: 740px; }
  .doc-title { font-family: var(--font-display); font-size: var(--text-2xl); font-weight: 700; letter-spacing: 0.04em; text-transform: uppercase; margin-bottom: 8px; }
  .doc-intro { font-size: var(--text-sm); color: var(--ink-soft); line-height: var(--leading-relaxed); margin-bottom: 40px; }
  h2 { font-family: var(--font-display); font-size: var(--text-lg); font-weight: 700; letter-spacing: 0.04em; margin: 40px 0 16px; padding-bottom: 8px; border-bottom: 1px solid var(--border); }
  h3 { font-family: var(--font-display); font-size: var(--text-md); font-weight: 600; margin: 24px 0 8px; }
  p { font-size: var(--text-sm); color: var(--ink-soft); line-height: var(--leading-relaxed); margin-bottom: 16px; }
  code { font-family: var(--font-body); font-size: var(--text-sm); color: var(--pulse); }
  .config-table { width: 100%; border-collapse: collapse; margin-bottom: 24px; }
  .config-table th { font-family: var(--font-ui); font-size: 9px; letter-spacing: 0.15em; text-transform: uppercase; color: var(--ink-muted); text-align: left; padding: 8px 12px; border-bottom: 1px solid var(--border); }
  .config-table td { font-family: var(--font-ui); font-size: var(--text-xs); padding: 10px 12px; border-bottom: 1px solid var(--border); color: var(--ink-soft); }
  .config-table td code { color: var(--pulse); }
</style>
