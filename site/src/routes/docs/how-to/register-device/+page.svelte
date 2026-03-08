<script lang="ts">
  import CodeBlock from '$lib/components/ui/CodeBlock.svelte';
</script>

<svelte:head><title>Register a Device - FreshBlu Docs</title></svelte:head>

<div class="doc-page">
  <h1 class="doc-title">Register a Device</h1>
  <p class="doc-intro">Create a new device with optional properties, type, and initial permissions. The response includes a plaintext token shown only once.</p>

  <h2>Basic Registration</h2>
  <p>POST to <code>/devices</code> with an empty body. The server assigns a UUID and generates a token.</p>
  <CodeBlock lang="bash" code={`curl -X POST http://localhost:3000/devices \\
  -H "Content-Type: application/json" \\
  -d '{}'`} />
  <p>Response:</p>
  <CodeBlock lang="json" code={`{
  "uuid": "d0a1f3b2-...",
  "token": "a8c3e9...",
  "online": false,
  "meshblu": {
    "whitelists": { ... }
  }
}`} />
  <p>Store both values immediately. The token is never returned again.</p>

  <h2>Set Device Type</h2>
  <p>Pass a <code>type</code> field to classify the device. This is freeform. Common values include <code>device:sensor</code>, <code>device:gateway</code>, or <code>device:controller</code>.</p>
  <CodeBlock lang="bash" code={`curl -X POST http://localhost:3000/devices \\
  -H "Content-Type: application/json" \\
  -d '{
    "type": "device:temperature-sensor"
  }'`} />

  <h2>Set Custom Properties</h2>
  <p>Any additional JSON fields in the body become device properties. These are stored and returned on subsequent queries.</p>
  <CodeBlock lang="bash" code={`curl -X POST http://localhost:3000/devices \\
  -H "Content-Type: application/json" \\
  -d '{
    "type": "device:sensor",
    "name": "Living Room Thermostat",
    "location": "building-a",
    "firmware": "2.1.0"
  }'`} />

  <h2>Set Initial Permissions</h2>
  <p>By default, new devices get open whitelists (wildcard <code>*</code> on all permission lists). To lock down a device at registration, pass the <code>meshblu.whitelists</code> block.</p>
  <CodeBlock lang="bash" code={`curl -X POST http://localhost:3000/devices \\
  -H "Content-Type: application/json" \\
  -d '{
    "type": "device:sensor",
    "meshblu": {
      "whitelists": {
        "discover": {
          "view": [{"uuid": "*"}]
        },
        "configure": {
          "update": [{"uuid": "OWNER_UUID"}],
          "sent": [{"uuid": "OWNER_UUID"}],
          "received": [{"uuid": "OWNER_UUID"}]
        },
        "message": {
          "from": [{"uuid": "OWNER_UUID"}],
          "sent": [{"uuid": "OWNER_UUID"}],
          "received": [{"uuid": "OWNER_UUID"}]
        },
        "broadcast": {
          "sent": [{"uuid": "*"}],
          "received": []
        }
      }
    }
  }'`} />
  <p>This example creates a sensor that anyone can discover, but only <code>OWNER_UUID</code> can configure or message. Broadcasts are open. See the <a href="/docs/explanation/permission-model">permission model</a> for full details.</p>

  <h2>Closed Registration</h2>
  <p>When the server runs with <code>open_registration</code> disabled, you must authenticate to register new devices. Pass credentials via Basic auth.</p>
  <CodeBlock lang="bash" code={`CREDS=$(echo -n "EXISTING_UUID:EXISTING_TOKEN" | base64)

curl -X POST http://localhost:3000/devices \\
  -H "Authorization: Basic $CREDS" \\
  -H "Content-Type: application/json" \\
  -d '{"type": "device:new-sensor"}'`} />

  <h2>Verify Registration</h2>
  <p>Confirm the device was created by calling <code>/whoami</code> with the new credentials.</p>
  <CodeBlock lang="bash" code={`NEW_CREDS=$(echo -n "NEW_UUID:NEW_TOKEN" | base64)

curl http://localhost:3000/whoami \\
  -H "Authorization: Basic $NEW_CREDS"`} />
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
