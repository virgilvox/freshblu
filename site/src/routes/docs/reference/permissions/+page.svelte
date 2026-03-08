<script>
  import CodeBlock from '$lib/components/ui/CodeBlock.svelte';
</script>

<svelte:head><title>Permissions Reference - FreshBlu Docs</title></svelte:head>
<div class="doc-page">
  <h1 class="doc-title">Permissions</h1>
  <p class="doc-intro">FreshBlu implements the Meshblu v2.0 permission system. Every device has a <code>meshblu.whitelists</code> object that controls who can interact with it. Permissions are organized into four categories, each with sub-types that govern specific operations.</p>

  <h2>Core Rules</h2>
  <p>A device can always perform any operation on itself. Self-access bypasses all whitelist checks.</p>
  <p>An empty whitelist means nobody except the device itself is allowed. This is the default for locked-down devices.</p>
  <p>The wildcard UUID <code>"*"</code> in any whitelist entry means any device is allowed. Newly registered devices in open mode use wildcard whitelists.</p>

  <h2>Whitelist Structure</h2>
  <CodeBlock lang="json" code={`{
  "meshblu": {
    "whitelists": {
      "discover": {
        "view": [{ "uuid": "*" }],
        "as": []
      },
      "configure": {
        "update": [{ "uuid": "owner-uuid" }],
        "sent": [{ "uuid": "*" }],
        "received": [{ "uuid": "*" }],
        "as": []
      },
      "message": {
        "from": [{ "uuid": "*" }],
        "sent": [{ "uuid": "*" }],
        "received": [{ "uuid": "*" }],
        "as": []
      },
      "broadcast": {
        "sent": [{ "uuid": "*" }],
        "received": [{ "uuid": "*" }],
        "as": []
      }
    }
  }
}`} />

  <h2>Discover</h2>
  <p>Controls visibility of the device.</p>

  <table class="config-table">
    <thead>
      <tr>
        <th>Sub-type</th>
        <th>Controls</th>
        <th>Used by</th>
      </tr>
    </thead>
    <tbody>
      <tr>
        <td><code>view</code></td>
        <td>Who can GET this device's properties</td>
        <td><code>GET /devices/:uuid</code>, <code>POST /devices/search</code></td>
      </tr>
      <tr>
        <td><code>as</code></td>
        <td>Who can act as this device for discovery operations (via <code>x-meshblu-as</code> header)</td>
        <td><code>GET /devices/:uuid</code>, <code>POST /devices/search</code> with <code>x-meshblu-as</code></td>
      </tr>
    </tbody>
  </table>
  <p>When <code>discover.view</code> is denied, the API returns 404 (not 403) to avoid leaking device existence.</p>

  <h2>Configure</h2>
  <p>Controls modification and observation of device configuration changes.</p>

  <table class="config-table">
    <thead>
      <tr>
        <th>Sub-type</th>
        <th>Controls</th>
        <th>Used by</th>
      </tr>
    </thead>
    <tbody>
      <tr>
        <td><code>update</code></td>
        <td>Who can modify this device's properties</td>
        <td><code>PUT /devices/:uuid</code>, <code>DELETE /devices/:uuid</code>, token management</td>
      </tr>
      <tr>
        <td><code>sent</code></td>
        <td>Who can receive config-change events emitted by this device</td>
        <td>Subscription type <code>configure.sent</code></td>
      </tr>
      <tr>
        <td><code>received</code></td>
        <td>Who can receive config-change events sent to this device</td>
        <td>Subscription type <code>configure.received</code></td>
      </tr>
      <tr>
        <td><code>as</code></td>
        <td>Who can act as this device for configure operations</td>
        <td><code>PUT /devices/:uuid</code>, <code>DELETE /devices/:uuid</code> with <code>x-meshblu-as</code></td>
      </tr>
    </tbody>
  </table>

  <h2>Message</h2>
  <p>Controls direct messaging between devices.</p>

  <table class="config-table">
    <thead>
      <tr>
        <th>Sub-type</th>
        <th>Controls</th>
        <th>Used by</th>
      </tr>
    </thead>
    <tbody>
      <tr>
        <td><code>from</code></td>
        <td>Who can send messages TO this device</td>
        <td><code>POST /messages</code>, WS <code>message</code>, MQTT <code>message</code></td>
      </tr>
      <tr>
        <td><code>sent</code></td>
        <td>Who can receive copies of messages SENT by this device</td>
        <td>Subscription type <code>message.sent</code></td>
      </tr>
      <tr>
        <td><code>received</code></td>
        <td>Who can receive copies of messages RECEIVED by this device</td>
        <td>Subscription type <code>message.received</code></td>
      </tr>
      <tr>
        <td><code>as</code></td>
        <td>Who can send messages as this device</td>
        <td><code>POST /messages</code> with <code>x-meshblu-as</code></td>
      </tr>
    </tbody>
  </table>
  <p>When <code>message.from</code> is denied, the message is silently dropped. No error is returned to the sender.</p>

  <h2>Broadcast</h2>
  <p>Controls broadcast event subscriptions.</p>

  <table class="config-table">
    <thead>
      <tr>
        <th>Sub-type</th>
        <th>Controls</th>
        <th>Used by</th>
      </tr>
    </thead>
    <tbody>
      <tr>
        <td><code>sent</code></td>
        <td>Who can subscribe to broadcasts SENT by this device</td>
        <td>Subscription type <code>broadcast.sent</code></td>
      </tr>
      <tr>
        <td><code>received</code></td>
        <td>Who can subscribe to broadcasts RECEIVED by this device</td>
        <td>Subscription type <code>broadcast.received</code></td>
      </tr>
      <tr>
        <td><code>as</code></td>
        <td>Who can broadcast as this device</td>
        <td><code>POST /broadcasts</code> with <code>x-meshblu-as</code></td>
      </tr>
    </tbody>
  </table>

  <h2>Preset Configurations</h2>

  <h3>Open (public)</h3>
  <p>Used for newly registered devices in open mode. All whitelist entries contain <code>{`{ "uuid": "*" }`}</code> except <code>as</code> fields, which are empty.</p>
  <CodeBlock lang="json" code={`{
  "discover": { "view": [{"uuid":"*"}], "as": [] },
  "configure": { "update": [{"uuid":"*"}], "sent": [{"uuid":"*"}], "received": [{"uuid":"*"}], "as": [] },
  "message": { "from": [{"uuid":"*"}], "sent": [{"uuid":"*"}], "received": [{"uuid":"*"}], "as": [] },
  "broadcast": { "sent": [{"uuid":"*"}], "received": [{"uuid":"*"}], "as": [] }
}`} />

  <h3>Private (locked down)</h3>
  <p>Only the owner UUID appears in each whitelist. All <code>as</code> fields are empty.</p>
  <CodeBlock lang="json" code={`{
  "discover": { "view": [{"uuid":"owner-uuid"}], "as": [] },
  "configure": { "update": [{"uuid":"owner-uuid"}], "sent": [{"uuid":"owner-uuid"}], "received": [{"uuid":"owner-uuid"}], "as": [] },
  "message": { "from": [{"uuid":"owner-uuid"}], "sent": [{"uuid":"owner-uuid"}], "received": [{"uuid":"owner-uuid"}], "as": [] },
  "broadcast": { "sent": [{"uuid":"owner-uuid"}], "received": [{"uuid":"owner-uuid"}], "as": [] }
}`} />

  <h2>Unregister Subscriptions</h2>
  <p>The subscription types <code>unregister.sent</code> and <code>unregister.received</code> do not have their own whitelist category. Permission to subscribe to unregister events is governed by <code>discover.view</code>. If you can see a device, you can subscribe to know when it is deleted.</p>
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
