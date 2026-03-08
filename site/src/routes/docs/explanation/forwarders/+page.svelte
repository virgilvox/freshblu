<script lang="ts">
  import CodeBlock from '$lib/components/ui/CodeBlock.svelte';
</script>

<svelte:head><title>Forwarders - FreshBlu Docs</title></svelte:head>

<div class="doc-page">
  <h1 class="doc-title">Forwarders</h1>
  <p class="doc-intro">Forwarders push device events to external HTTP endpoints and back into the message bus. This page explains how they work, how they differ from subscriptions, and the event type mapping.</p>

  <h2>Forwarders vs Subscriptions</h2>
  <p>Subscriptions route events between devices within FreshBlu. Forwarders route events to the outside world. Both are triggered by the same event types, but they serve different purposes.</p>
  <ul>
    <li><strong>Subscriptions</strong> - device-to-device. Subscriber must be a registered FreshBlu device. Delivery is via WebSocket, MQTT, or the internal bus.</li>
    <li><strong>Forwarders</strong> - device-to-HTTP or device-to-self. Target is an external URL or a re-emission into the bus. No registration required on the receiving end.</li>
  </ul>
  <p>A device can have both subscriptions and forwarders for the same event type. Both fire independently.</p>

  <h2>Forwarder Types</h2>
  <p>Each forwarder entry has a <code>type</code> field that determines how the event is delivered.</p>

  <h3>Webhook</h3>
  <p>Sends an HTTP request to an external URL. The event payload is the JSON request body. Configuration fields:</p>
  <CodeBlock lang="rust" code={`pub struct WebhookForwarder {
    pub url: String,
    pub method: String,              // GET, POST, PUT, DELETE (default: POST)
    pub sign_request: bool,
    pub generate_and_forward_meshblu_credentials: bool,
}`} />
  <p>Every webhook request includes an <code>X-Meshblu-Uuid</code> header with the device UUID and a <code>Content-Type: application/json</code> header.</p>
  <p>When <code>generate_and_forward_meshblu_credentials</code> is true, FreshBlu generates a short-lived token (5-minute expiry, tagged <code>webhook-credential</code>) and sends it as a Base64-encoded <code>uuid:token</code> pair in the <code>Authorization: Bearer</code> header. The receiving server can use this to call back into FreshBlu.</p>

  <h3>Meshblu</h3>
  <p>Re-emits the event as a message from the device to itself. This is useful for triggering downstream logic without leaving the mesh. The <code>MeshbluForwarder</code> struct has no configuration fields.</p>
  <CodeBlock lang="rust" code={`pub struct MeshbluForwarder {}`} />
  <p>The re-emitted message has <code>topic: "forwarder"</code> and the original event payload. This allows handlers to distinguish forwarded events from direct messages.</p>

  <h2>Event Type Mapping</h2>
  <p>Forwarders are organized in the same four categories as subscriptions, each with sent/received pairs:</p>
  <CodeBlock lang="rust" code={`pub struct Forwarders {
    pub broadcast: ForwarderPair,   // sent + received
    pub configure: ForwarderPair,   // sent + received
    pub message: ForwarderPair,     // sent + received
    pub unregister: ForwarderPair,  // sent + received
}

pub struct ForwarderPair {
    pub sent: Vec<ForwarderEntry>,
    pub received: Vec<ForwarderEntry>,
}`} />
  <p>The eight event slots:</p>
  <ul>
    <li><code>message.sent</code> - fires when this device sends a direct message.</li>
    <li><code>message.received</code> - fires when this device receives a direct message.</li>
    <li><code>broadcast.sent</code> - fires when this device sends a broadcast.</li>
    <li><code>broadcast.received</code> - fires when this device receives a broadcast.</li>
    <li><code>configure.sent</code> - fires when this device's config is updated.</li>
    <li><code>configure.received</code> - fires when a config update is directed at this device.</li>
    <li><code>unregister.sent</code> - fires when this device is deleted.</li>
    <li><code>unregister.received</code> - fires when this device's unregistration is observed.</li>
  </ul>

  <h2>Execution Model</h2>
  <p>The <code>WebhookExecutor</code> handles all forwarder execution. Key behaviors:</p>
  <ul>
    <li><strong>Async</strong> - forwarders fire in a spawned task. They do not block the event pipeline or the HTTP response.</li>
    <li><strong>Concurrent webhooks</strong> - all webhook-type forwarders for a single event fire concurrently via <code>join_all</code>.</li>
    <li><strong>Sequential meshblu</strong> - meshblu-type forwarders execute sequentially because they mutate bus state.</li>
    <li><strong>Cap per event</strong> - maximum 10 forwarders per event slot. Entries beyond the cap are ignored.</li>
    <li><strong>Timeout</strong> - webhook HTTP requests time out after 10 seconds.</li>
    <li><strong>Fire and forget</strong> - failed webhooks are logged and counted in Prometheus metrics (<code>WEBHOOKS_FAILED</code>), but do not retry.</li>
  </ul>

  <h2>Loop Detection</h2>
  <p>Meshblu forwarders can create loops: Device A forwards to itself, which triggers another forward. FreshBlu prevents this with two checks:</p>
  <ul>
    <li><strong>Depth limit</strong> - the forwarding chain stops at depth 5.</li>
    <li><strong>Cycle detection</strong> - if a device UUID appears twice in the forwarding chain, the loop is broken.</li>
  </ul>

  <h2>SSRF Protection</h2>
  <p>Webhook URLs are validated before the request is sent. Rejected targets:</p>
  <ul>
    <li>Localhost and loopback addresses (127.0.0.1, ::1, 0.0.0.0).</li>
    <li>Private IP ranges (10.x, 172.16.x, 192.168.x).</li>
    <li>Link-local addresses (169.254.x.x).</li>
    <li>Cloud metadata endpoints (169.254.169.254, metadata.google.internal).</li>
    <li>Internal TLDs (.internal, .local, .localhost).</li>
    <li>Non-HTTP schemes (file://, ftp://, gopher://).</li>
  </ul>
  <p>Only <code>http://</code> and <code>https://</code> URLs with public hosts are allowed.</p>

  <h2>Storage</h2>
  <p>Forwarders are stored as part of the device document under <code>meshblu.forwarders</code>. They are set and updated via <code>PUT /devices/:uuid</code> like any other device property. There is no separate API for managing individual forwarder entries. To add or remove a forwarder, update the entire forwarders block.</p>
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
