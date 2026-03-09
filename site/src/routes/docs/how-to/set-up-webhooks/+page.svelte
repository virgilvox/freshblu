<script lang="ts">
  import CodeBlock from '$lib/components/ui/CodeBlock.svelte';
</script>

<svelte:head><title>Set Up Webhooks - FreshBlu Docs</title></svelte:head>

<div class="doc-page">
  <h1 class="doc-title">Set Up Webhooks</h1>
  <p class="doc-intro">Configure forwarders on a device to fire HTTP webhooks when events occur. Webhooks let external services react to messages, config changes, broadcasts, and unregistrations.</p>

  <h2>How Forwarders Work</h2>
  <p>Forwarders are stored on the device document under <code>meshblu.forwarders</code>. Each event category has a <code>sent</code> and <code>received</code> array. When an event fires, FreshBlu iterates the matching array and executes each forwarder entry.</p>
  <p>There are two forwarder types:</p>
  <ul>
    <li><strong>webhook</strong> - sends an HTTP request to an external URL</li>
    <li><strong>meshblu</strong> - re-emits the event as a message back into the bus</li>
  </ul>

  <h2>Add a Webhook Forwarder</h2>
  <p>Update the device to add a webhook that fires on every outgoing message:</p>
  <CodeBlock lang="bash" code={`CREDS=$(echo -n "UUID:TOKEN" | base64)

curl -X PUT https://api.freshblu.org/devices/MY_UUID \\
  -H "Authorization: Basic $CREDS" \\
  -H "Content-Type: application/json" \\
  -d '{
    "meshblu": {
      "forwarders": {
        "message": {
          "sent": [
            {
              "type": "webhook",
              "url": "https://hooks.example.com/freshblu",
              "method": "POST",
              "signRequest": false,
              "generateAndForwardMeshbluCredentials": false
            }
          ]
        }
      }
    }
  }'`} />
  <p>When this device sends a message, FreshBlu POSTs the message payload to the webhook URL.</p>

  <h2>Webhook Request Format</h2>
  <p>FreshBlu sends the event payload as JSON in the request body. Two headers are always included:</p>
  <ul>
    <li><code>X-Meshblu-Uuid</code> - the UUID of the device that owns the forwarder</li>
    <li><code>Content-Type: application/json</code></li>
  </ul>

  <h2>Forward Credentials</h2>
  <p>Set <code>generateAndForwardMeshbluCredentials</code> to <code>true</code> to include a short-lived token in the webhook request. FreshBlu generates a token that expires in 5 minutes and sends it in the <code>Authorization</code> header as a Base64-encoded <code>uuid:token</code> pair.</p>
  <CodeBlock lang="json" code={`{
  "type": "webhook",
  "url": "https://hooks.example.com/freshblu",
  "method": "POST",
  "signRequest": false,
  "generateAndForwardMeshbluCredentials": true
}`} />
  <p>The receiving server can use these credentials to call back into FreshBlu on behalf of the device.</p>

  <h2>Supported HTTP Methods</h2>
  <p>Set the <code>method</code> field to <code>GET</code>, <code>POST</code>, <code>PUT</code>, or <code>DELETE</code>. Defaults to <code>POST</code> if omitted.</p>

  <h2>Event Categories</h2>
  <p>Forwarders can be attached to any combination of these event pairs:</p>
  <ul>
    <li><code>message.sent</code> / <code>message.received</code></li>
    <li><code>broadcast.sent</code> / <code>broadcast.received</code></li>
    <li><code>configure.sent</code> / <code>configure.received</code></li>
    <li><code>unregister.sent</code> / <code>unregister.received</code></li>
  </ul>

  <h3>Fire on Config Changes</h3>
  <CodeBlock lang="bash" code={`curl -X PUT https://api.freshblu.org/devices/MY_UUID \\
  -H "Authorization: Basic $CREDS" \\
  -H "Content-Type: application/json" \\
  -d '{
    "meshblu": {
      "forwarders": {
        "configure": {
          "sent": [
            {
              "type": "webhook",
              "url": "https://hooks.example.com/config-changed",
              "method": "POST"
            }
          ]
        }
      }
    }
  }'`} />

  <h3>Fire on Unregistration</h3>
  <CodeBlock lang="bash" code={`curl -X PUT https://api.freshblu.org/devices/MY_UUID \\
  -H "Authorization: Basic $CREDS" \\
  -H "Content-Type: application/json" \\
  -d '{
    "meshblu": {
      "forwarders": {
        "unregister": {
          "sent": [
            {
              "type": "webhook",
              "url": "https://hooks.example.com/device-removed",
              "method": "POST"
            }
          ]
        }
      }
    }
  }'`} />

  <h2>Multiple Forwarders</h2>
  <p>Each event slot accepts an array. You can attach up to 10 forwarders per event type.</p>
  <CodeBlock lang="json" code={`{
  "message": {
    "sent": [
      {"type": "webhook", "url": "https://hooks.a.com/msg"},
      {"type": "webhook", "url": "https://hooks.b.com/msg"},
      {"type": "meshblu"}
    ]
  }
}`} />

  <h2>Meshblu Forwarders</h2>
  <p>A meshblu-type forwarder re-emits the event as a message from the device to itself. This is useful for chaining logic across multiple devices. Loop detection prevents circular forwarding. The maximum chain depth is 5.</p>
  <CodeBlock lang="json" code={`{
  "message": {
    "received": [
      {"type": "meshblu"}
    ]
  }
}`} />

  <h2>SSRF Protection</h2>
  <p>FreshBlu rejects webhook URLs that target localhost, private IP ranges (10.x, 172.16.x, 192.168.x), link-local addresses, cloud metadata endpoints (169.254.169.254), and internal TLDs (.internal, .local, .localhost). Only <code>http://</code> and <code>https://</code> schemes are allowed.</p>
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
