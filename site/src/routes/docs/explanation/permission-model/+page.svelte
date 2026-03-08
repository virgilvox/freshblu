<script lang="ts">
  import CodeBlock from '$lib/components/ui/CodeBlock.svelte';
</script>

<svelte:head><title>Permission Model - FreshBlu Docs</title></svelte:head>

<div class="doc-page">
  <h1 class="doc-title">Permission Model</h1>
  <p class="doc-intro">FreshBlu uses the Meshblu v2.0 whitelist system. Every device has a whitelists block that controls who can do what. This page explains the design rationale and the semantics of each permission type.</p>

  <h2>Why Whitelists</h2>
  <p>Meshblu v1 used a simple owner/public model. Any device could message any other device. This made it easy to get started but impossible to lock down in production. V2 introduced per-operation whitelists: each device declares exactly which other devices may interact with it and in what way.</p>
  <p>The whitelist approach is deny-by-default. If a UUID is not in the list, the operation is rejected. There is no global admin override. This pushes access control to the edges, where each device owner makes their own decisions.</p>

  <h2>The Four Categories</h2>
  <p>Permissions are organized into four categories, each with sub-types.</p>

  <h3>discover</h3>
  <p>Controls visibility. Can another device see that this device exists?</p>
  <ul>
    <li><code>view</code> - checked on <code>GET /devices/:uuid</code> and <code>POST /devices/search</code>. If the caller is not in this list, the device appears to not exist (returns 404, not 403).</li>
    <li><code>as</code> - checked when a caller uses the <code>x-meshblu-as</code> header to discover on behalf of this device.</li>
  </ul>

  <h3>configure</h3>
  <p>Controls modification and config event distribution.</p>
  <ul>
    <li><code>update</code> - checked on <code>PUT /devices/:uuid</code> and <code>DELETE /devices/:uuid</code>. Guards who can change the device document.</li>
    <li><code>sent</code> - controls who can subscribe to config change events that this device emits. When the device is updated, subscribers in this list receive the new config.</li>
    <li><code>received</code> - controls who can subscribe to config change events directed at this device.</li>
    <li><code>as</code> - checked when a caller uses <code>x-meshblu-as</code> to configure on behalf of this device.</li>
  </ul>

  <h3>message</h3>
  <p>Controls direct messaging.</p>
  <ul>
    <li><code>from</code> - checked on the target device when a message is sent to it. This is the gatekeeper for inbound messages. If the sender is not in this list, the message is silently dropped.</li>
    <li><code>sent</code> - controls who can subscribe to messages sent FROM this device. Allows third parties to monitor outbound traffic.</li>
    <li><code>received</code> - controls who can subscribe to messages received BY this device. Allows third parties to monitor inbound traffic.</li>
    <li><code>as</code> - checked when a caller uses <code>x-meshblu-as</code> to send messages on behalf of this device.</li>
  </ul>

  <h3>broadcast</h3>
  <p>Controls broadcast event distribution.</p>
  <ul>
    <li><code>sent</code> - controls who can subscribe to broadcasts from this device. This is checked when creating a <code>broadcast-sent</code> subscription.</li>
    <li><code>received</code> - controls who can subscribe to broadcasts received by this device.</li>
    <li><code>as</code> - checked when a caller uses <code>x-meshblu-as</code> to broadcast on behalf of this device.</li>
  </ul>

  <h2>Self-Access</h2>
  <p>A device always has full access to itself. The <code>PermissionChecker</code> short-circuits all checks when the actor UUID equals the device UUID. No whitelist entry is needed.</p>
  <CodeBlock lang="rust" code={`fn is_self(&self) -> bool {
    self.actor == self.device_uuid
}

pub fn can_discover_view(&self) -> bool {
    self.is_self() || check_whitelist(&self.device.discover.view, self.actor)
}`} />

  <h2>Wildcard</h2>
  <p>The UUID value <code>"*"</code> matches any device. Adding <code>{`{"uuid": "*"}`}</code> to a whitelist makes that operation public. The <code>Whitelists::open()</code> constructor sets wildcard on all operation types except <code>as</code> sub-types.</p>

  <h2>Empty List Semantics</h2>
  <p>An empty array means no external device has access. Combined with the self-access rule, only the device itself can perform the operation. This is the most restrictive setting. The <code>Whitelists::default()</code> constructor produces empty lists everywhere, locking the device completely.</p>

  <h2>Private Constructor</h2>
  <p><code>Whitelists::private(owner)</code> creates whitelists where only the specified owner UUID has access. The device itself still has self-access. This is useful when a controller device registers sensors that only it should manage.</p>

  <h2>When Each Check Applies</h2>
  <ul>
    <li><code>discover.view</code> - <code>GET /devices/:uuid</code>, <code>POST /devices/search</code> (filters results), <code>GET /whoami</code> (implicitly passes, always self).</li>
    <li><code>discover.as</code> - <code>GET /devices/:uuid</code> with <code>x-meshblu-as</code> header.</li>
    <li><code>configure.update</code> - <code>PUT /devices/:uuid</code>, <code>DELETE /devices/:uuid</code>, creating subscriptions for another device.</li>
    <li><code>configure.as</code> - <code>PUT /devices/:uuid</code> or <code>DELETE /devices/:uuid</code> with <code>x-meshblu-as</code> header.</li>
    <li><code>message.from</code> - <code>POST /messages</code> (checked on each target device), WebSocket message action.</li>
    <li><code>message.as</code> - <code>POST /messages</code> with <code>x-meshblu-as</code> header.</li>
    <li><code>broadcast.sent</code> - creating a <code>broadcast-sent</code> subscription.</li>
    <li><code>broadcast.as</code> - <code>POST /messages</code> with <code>devices: ["*"]</code> and <code>x-meshblu-as</code> header.</li>
  </ul>

  <h2>Silent Rejection</h2>
  <p>Some permission failures are silent. When sending a direct message to multiple devices, targets that deny <code>message.from</code> are skipped without error. The sender receives <code>{`{"sent": true}`}</code> regardless. Discovery failures return 404 instead of 403, hiding the existence of the device from unauthorized callers.</p>
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
