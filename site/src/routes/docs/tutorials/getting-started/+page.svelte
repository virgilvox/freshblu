<script lang="ts">
  import CodeBlock from '$lib/components/ui/CodeBlock.svelte';
</script>

<svelte:head><title>Getting Started - FreshBlu Docs</title></svelte:head>

<div class="doc-page">
  <h1 class="doc-title">Getting Started</h1>
  <p class="doc-intro">Register a device, query it, send a message, and listen over WebSocket. Four steps, five minutes. A public instance is available at <code>https://api.freshblu.org</code>, or you can run your own server locally.</p>

  <h2>1. Register a Device</h2>
  <p>POST to <code>/devices</code> with no body. The server returns a UUID and a single-use token.</p>
  <CodeBlock lang="bash" code={`curl -X POST https://api.freshblu.org/devices`} />
  <p>Response:</p>
  <CodeBlock lang="json" code={`{
  "uuid": "d0a1f3b2-...",
  "token": "a8c3e9..."
}`} />
  <p>Save both values. The token is shown only once.</p>

  <h2>2. Check Your Device</h2>
  <p>Base64-encode the credentials as <code>uuid:token</code> and pass them in the Authorization header.</p>
  <CodeBlock lang="bash" code={`# encode credentials
CREDS=$(echo -n "UUID:TOKEN" | base64)

curl https://api.freshblu.org/devices/UUID \\
  -H "Authorization: Basic $CREDS"`} />
  <p>You will get back the full device document, including its type, online status, and whitelist configuration.</p>

  <h2>3. Send a Message</h2>
  <p>Post a JSON body to <code>/messages</code>. The <code>devices</code> array lists target UUIDs. The <code>payload</code> is freeform.</p>
  <CodeBlock lang="bash" code={`curl -X POST https://api.freshblu.org/messages \\
  -H "Authorization: Basic $CREDS" \\
  -H "Content-Type: application/json" \\
  -d '{
    "devices": ["TARGET_UUID"],
    "payload": {"temp": 22.5}
  }'`} />
  <p>The server routes the message through the subscription system. If the target is online and permits messages from your device, it receives the payload immediately.</p>

  <h2>4. Listen via WebSocket</h2>
  <p>Open a WebSocket connection, send an <code>identity</code> frame, wait for <code>ready</code>, then listen for incoming messages.</p>
  <CodeBlock lang="javascript" code={`const ws = new WebSocket('wss://api.freshblu.org/ws');

ws.onopen = () => {
  // authenticate
  ws.send(JSON.stringify({
    event: 'identity',
    uuid: 'YOUR_UUID',
    token: 'YOUR_TOKEN'
  }));
};

ws.onmessage = (e) => {
  const msg = JSON.parse(e.data);

  if (msg.event === 'ready') {
    console.log('connected and authenticated');
    return;
  }

  if (msg.event === 'message') {
    console.log('received:', msg.payload);
  }
};

ws.onerror = (err) => console.error('ws error', err);
ws.onclose = () => console.log('disconnected');`} />
  <p>The connection stays open until you close it or the server shuts down. Messages arrive as they are sent. No polling required.</p>
</div>

<style>
  .doc-page { max-width: 740px; }
  .doc-title { font-family: var(--font-display); font-size: var(--text-2xl); font-weight: 700; letter-spacing: 0.04em; text-transform: uppercase; margin-bottom: 8px; }
  .doc-intro { font-size: var(--text-sm); color: var(--ink-soft); line-height: var(--leading-relaxed); margin-bottom: 40px; }
  h2 { font-family: var(--font-display); font-size: var(--text-lg); font-weight: 700; letter-spacing: 0.04em; margin: 40px 0 16px; padding-bottom: 8px; border-bottom: 1px solid var(--border); }
  p { font-size: var(--text-sm); color: var(--ink-soft); line-height: var(--leading-relaxed); margin-bottom: 16px; }
  code { font-family: var(--font-body); font-size: var(--text-sm); color: var(--pulse); }
</style>
