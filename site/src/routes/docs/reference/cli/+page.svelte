<script lang="ts">
  import CodeBlock from '$lib/components/ui/CodeBlock.svelte';
</script>

<svelte:head>
  <title>CLI Reference - FreshBlu</title>
</svelte:head>

<div class="doc-page">
  <h1 class="doc-title">CLI Reference</h1>
  <p>The <code>freshblu-cli</code> tool provides a command-line interface for interacting with FreshBlu servers. It is wire-compatible with <code>meshblu-util</code>.</p>

  <h2>Installation</h2>
  <CodeBlock code="cargo install freshblu-cli" lang="bash" />

  <h2>Global Flags</h2>
  <table class="doc-table">
    <thead>
      <tr><th>Flag</th><th>Env</th><th>Description</th></tr>
    </thead>
    <tbody>
      <tr><td><code>--server &lt;URL&gt;</code></td><td><code>MESHBLU_SERVER</code></td><td>Server URL (default: <code>http://localhost:3000</code>)</td></tr>
      <tr><td><code>--uuid &lt;UUID&gt;</code></td><td><code>MESHBLU_UUID</code></td><td>Device UUID for authentication</td></tr>
      <tr><td><code>--token &lt;TOKEN&gt;</code></td><td><code>MESHBLU_TOKEN</code></td><td>Device token for authentication</td></tr>
      <tr><td><code>--config &lt;PATH&gt;</code></td><td></td><td>Config file path (default: <code>meshblu.json</code>)</td></tr>
      <tr><td><code>--format &lt;FMT&gt;</code></td><td></td><td>Output format: <code>json</code> (default) or <code>table</code></td></tr>
    </tbody>
  </table>

  <h2>Commands</h2>

  <h3><code>register</code></h3>
  <p>Register a new device on the mesh.</p>
  <CodeBlock code={`freshblu register
freshblu register --type sensor --name "temp-01"`} lang="bash" />

  <h3><code>whoami</code></h3>
  <p>Show the authenticated device's information.</p>
  <CodeBlock code="freshblu whoami" lang="bash" />

  <h3><code>get &lt;UUID&gt;</code></h3>
  <p>Retrieve a device by UUID.</p>
  <CodeBlock code="freshblu get 550e8400-e29b-41d4-a716-446655440000" lang="bash" />

  <h3><code>update &lt;UUID&gt; &lt;JSON&gt;</code></h3>
  <p>Update device properties.</p>
  <CodeBlock code={`freshblu update 550e8400-e29b-41d4-a716-446655440000 '{"name":"updated"}'`} lang="bash" />

  <h3><code>unregister &lt;UUID&gt;</code></h3>
  <p>Delete a device from the mesh.</p>
  <CodeBlock code="freshblu unregister 550e8400-e29b-41d4-a716-446655440000" lang="bash" />

  <h3><code>message &lt;JSON&gt;</code></h3>
  <p>Send a message to one or more devices.</p>
  <CodeBlock code={`freshblu message '{"devices":["*"],"payload":{"hello":true}}'`} lang="bash" />

  <h3><code>subscribe &lt;EMITTER_UUID&gt; &lt;TYPE&gt;</code></h3>
  <p>Create a subscription. Types: <code>broadcast.sent</code>, <code>broadcast.received</code>, <code>configure.sent</code>, <code>configure.received</code>, <code>message.sent</code>, <code>message.received</code>, <code>unregister.sent</code>, <code>unregister.received</code>.</p>
  <CodeBlock code={`freshblu subscribe 550e8400-e29b-41d4-a716-446655440000 message.received`} lang="bash" />

  <h3><code>token generate &lt;UUID&gt;</code></h3>
  <p>Generate a new session token for a device.</p>
  <CodeBlock code="freshblu token generate 550e8400-e29b-41d4-a716-446655440000" lang="bash" />

  <h3><code>token revoke &lt;UUID&gt; &lt;TOKEN&gt;</code></h3>
  <p>Revoke a specific token.</p>
  <CodeBlock code="freshblu token revoke 550e8400-e29b-41d4-a716-446655440000 abc123..." lang="bash" />

  <h3><code>status</code></h3>
  <p>Check server health.</p>
  <CodeBlock code={`freshblu status
# {"meshblu":true,"sky":"default"}`} lang="bash" />

  <h2>Configuration File</h2>
  <p>The CLI looks for a <code>meshblu.json</code> file in the current directory:</p>
  <CodeBlock code={`{
  "server": "http://localhost:3000",
  "uuid": "550e8400-e29b-41d4-a716-446655440000",
  "token": "your-device-token"
}`} lang="json" />
  <p>Command-line flags and environment variables override file settings.</p>
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
  .doc-table {
    width: 100%;
    border-collapse: collapse;
    margin-bottom: 16px;
    font-family: var(--font-ui);
    font-size: var(--text-sm);
  }
  .doc-table th {
    text-align: left;
    padding: 8px 12px;
    border-bottom: 2px solid var(--border);
    font-size: var(--text-xs);
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--ink-muted);
  }
  .doc-table td {
    padding: 8px 12px;
    border-bottom: 1px solid var(--border);
    color: var(--ink-soft);
  }
</style>
