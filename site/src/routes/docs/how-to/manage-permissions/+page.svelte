<script lang="ts">
  import CodeBlock from '$lib/components/ui/CodeBlock.svelte';
</script>

<svelte:head><title>Manage Permissions - FreshBlu Docs</title></svelte:head>

<div class="doc-page">
  <h1 class="doc-title">Manage Permissions</h1>
  <p class="doc-intro">Control who can discover, configure, message, and broadcast to your devices using Meshblu v2 whitelists.</p>

  <h2>Permission Categories</h2>
  <p>Every device has a <code>meshblu.whitelists</code> block with four categories:</p>
  <ul>
    <li><strong>discover</strong> &mdash; who can view this device's properties (<code>view</code>, <code>as</code>)</li>
    <li><strong>configure</strong> &mdash; who can update this device and receive config events (<code>update</code>, <code>sent</code>, <code>received</code>, <code>as</code>)</li>
    <li><strong>message</strong> &mdash; who can send/receive direct messages (<code>from</code>, <code>sent</code>, <code>received</code>, <code>as</code>)</li>
    <li><strong>broadcast</strong> &mdash; who can subscribe to broadcast events (<code>sent</code>, <code>received</code>, <code>as</code>)</li>
  </ul>

  <h2>Whitelist Format</h2>
  <p>Each sub-type holds an array of objects with a <code>uuid</code> field. The special value <code>"*"</code> means any device is allowed.</p>
  <CodeBlock lang="json" code={`{
  "discover": {
    "view": [{"uuid": "*"}],
    "as": []
  }
}`} />
  <p>An empty array means nobody except the device itself has access. Self-access is always permitted regardless of whitelists.</p>

  <h2>View Current Permissions</h2>
  <CodeBlock lang="bash" code={`CREDS=$(echo -n "UUID:TOKEN" | base64)

curl http://localhost:3000/devices/UUID \\
  -H "Authorization: Basic $CREDS"`} />
  <p>The <code>meshblu.whitelists</code> block in the response shows all current permissions.</p>

  <h2>Add a UUID to a Whitelist</h2>
  <p>To grant another device permission to send messages to your device, add its UUID to <code>message.from</code>.</p>
  <CodeBlock lang="bash" code={`curl -X PUT http://localhost:3000/devices/MY_UUID \\
  -H "Authorization: Basic $CREDS" \\
  -H "Content-Type: application/json" \\
  -d '{
    "meshblu": {
      "whitelists": {
        "message": {
          "from": [
            {"uuid": "ALLOWED_SENDER_UUID"},
            {"uuid": "ANOTHER_SENDER_UUID"}
          ]
        }
      }
    }
  }'`} />
  <p>This replaces the entire <code>message.from</code> list. Include all UUIDs you want to keep.</p>

  <h2>Grant Discovery Access</h2>
  <p>Let specific devices view your properties:</p>
  <CodeBlock lang="bash" code={`curl -X PUT http://localhost:3000/devices/MY_UUID \\
  -H "Authorization: Basic $CREDS" \\
  -H "Content-Type: application/json" \\
  -d '{
    "meshblu": {
      "whitelists": {
        "discover": {
          "view": [
            {"uuid": "VIEWER_UUID_1"},
            {"uuid": "VIEWER_UUID_2"}
          ]
        }
      }
    }
  }'`} />

  <h2>Grant Configure Access</h2>
  <p>Let another device update your configuration:</p>
  <CodeBlock lang="bash" code={`curl -X PUT http://localhost:3000/devices/MY_UUID \\
  -H "Authorization: Basic $CREDS" \\
  -H "Content-Type: application/json" \\
  -d '{
    "meshblu": {
      "whitelists": {
        "configure": {
          "update": [{"uuid": "ADMIN_UUID"}],
          "sent": [{"uuid": "*"}],
          "received": [{"uuid": "ADMIN_UUID"}]
        }
      }
    }
  }'`} />
  <p>Here, <code>update</code> restricts who can modify the device. <code>sent</code> is set to wildcard so anyone can subscribe to config change events. <code>received</code> restricts who sees config changes directed at this device.</p>

  <h2>Use Wildcard for Public Access</h2>
  <p>Set <code>{`{"uuid": "*"}`}</code> to allow any device:</p>
  <CodeBlock lang="bash" code={`curl -X PUT http://localhost:3000/devices/MY_UUID \\
  -H "Authorization: Basic $CREDS" \\
  -H "Content-Type: application/json" \\
  -d '{
    "meshblu": {
      "whitelists": {
        "discover": { "view": [{"uuid": "*"}] },
        "message": { "from": [{"uuid": "*"}] },
        "broadcast": { "sent": [{"uuid": "*"}] }
      }
    }
  }'`} />
  <p>This makes the device publicly discoverable, open to messages from anyone, and broadcasts visible to all subscribers.</p>

  <h2>Lock Down a Device</h2>
  <p>Remove all access by setting empty arrays:</p>
  <CodeBlock lang="bash" code={`curl -X PUT http://localhost:3000/devices/MY_UUID \\
  -H "Authorization: Basic $CREDS" \\
  -H "Content-Type: application/json" \\
  -d '{
    "meshblu": {
      "whitelists": {
        "discover": { "view": [], "as": [] },
        "configure": { "update": [], "sent": [], "received": [], "as": [] },
        "message": { "from": [], "sent": [], "received": [], "as": [] },
        "broadcast": { "sent": [], "received": [], "as": [] }
      }
    }
  }'`} />
  <p>The device can still access itself. No other device can interact with it.</p>

  <h2>Remove a UUID from a Whitelist</h2>
  <p>Whitelists are replaced on update, not appended. To remove a UUID, resubmit the list without it.</p>
  <CodeBlock lang="bash" code={`# Before: from: [{"uuid": "A"}, {"uuid": "B"}, {"uuid": "C"}]
# Remove B by sending A and C only:

curl -X PUT http://localhost:3000/devices/MY_UUID \\
  -H "Authorization: Basic $CREDS" \\
  -H "Content-Type: application/json" \\
  -d '{
    "meshblu": {
      "whitelists": {
        "message": {
          "from": [{"uuid": "A"}, {"uuid": "C"}]
        }
      }
    }
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
