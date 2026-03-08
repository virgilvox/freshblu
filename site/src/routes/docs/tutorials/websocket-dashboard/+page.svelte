<script lang="ts">
  import CodeBlock from '$lib/components/ui/CodeBlock.svelte';
</script>

<svelte:head><title>WebSocket Dashboard - FreshBlu Docs</title></svelte:head>

<div class="doc-page">
  <h1 class="doc-title">WebSocket Dashboard</h1>
  <p class="doc-intro">Build a browser dashboard that receives device messages in real time and sends commands back. Plain HTML and JavaScript, no framework required.</p>

  <h2>1. Create the HTML Page</h2>
  <p>Start with a minimal page. A list for incoming messages. An input and button for sending.</p>
  <CodeBlock lang="html" code={`<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>FreshBlu Dashboard</title>
  <style>
    body { font-family: monospace; background: #0a0a0a; color: #ccc; padding: 24px; }
    #messages { list-style: none; padding: 0; max-height: 400px; overflow-y: auto; }
    #messages li { padding: 4px 0; border-bottom: 1px solid #222; font-size: 14px; }
    .controls { margin-top: 16px; }
    input, button { font-family: monospace; font-size: 14px; padding: 6px 10px; }
    button { cursor: pointer; background: #1a1a2e; color: #00e5ff; border: 1px solid #00e5ff; }
    input { background: #111; color: #ccc; border: 1px solid #333; width: 300px; }
    .status { color: #666; font-size: 12px; margin-bottom: 12px; }
  </style>
</head>
<body>
  <h1>Dashboard</h1>
  <div id="status" class="status">disconnected</div>
  <ul id="messages"></ul>
  <div class="controls">
    <input id="target" placeholder="target uuid" />
    <input id="payload" placeholder='{"cmd": "on"}' />
    <button id="send">Send</button>
  </div>
  <script src="dashboard.js"></script>
</body>
</html>`} />

  <h2>2. Connect the WebSocket</h2>
  <p>Create a <code>dashboard.js</code> file. Open a connection to the FreshBlu WebSocket endpoint.</p>
  <CodeBlock lang="javascript" code={`const UUID  = 'YOUR_UUID';
const TOKEN = 'YOUR_TOKEN';

const ws = new WebSocket('ws://localhost:3000/ws');
const statusEl = document.getElementById('status');

ws.onopen = () => {
  statusEl.textContent = 'connected, authenticating...';
};

ws.onerror = () => {
  statusEl.textContent = 'connection error';
};

ws.onclose = () => {
  statusEl.textContent = 'disconnected';
};`} />

  <h2>3. Send the Identity Message</h2>
  <p>On open, send an <code>identity</code> action with your device credentials. The server validates them before accepting further frames.</p>
  <CodeBlock lang="javascript" code={`ws.onopen = () => {
  statusEl.textContent = 'authenticating...';
  ws.send(JSON.stringify({
    action: 'identity',
    uuid: UUID,
    token: TOKEN
  }));
};`} />

  <h2>4. Handle the Ready Event</h2>
  <p>The server sends a <code>ready</code> action after successful authentication. Until you receive it, the connection is not usable.</p>
  <CodeBlock lang="javascript" code={`let authenticated = false;

ws.onmessage = (event) => {
  const msg = JSON.parse(event.data);

  if (msg.action === 'ready') {
    authenticated = true;
    statusEl.textContent = 'ready';
    return;
  }

  // handle other actions below
};`} />

  <h2>5. Display Incoming Messages</h2>
  <p>Append each incoming message to the list. Show the sender and payload.</p>
  <CodeBlock lang="javascript" code={`const messageList = document.getElementById('messages');

ws.onmessage = (event) => {
  const msg = JSON.parse(event.data);

  if (msg.action === 'ready') {
    authenticated = true;
    statusEl.textContent = 'ready';
    return;
  }

  if (msg.action === 'message') {
    const li = document.createElement('li');
    const time = new Date().toLocaleTimeString();
    li.textContent = time + ' [' + msg.fromUuid + '] '
      + JSON.stringify(msg.payload);
    messageList.prepend(li);
  }
};`} />

  <h2>6. Add the Send Button</h2>
  <p>Read the target UUID and payload from the inputs. Send a message action through the WebSocket.</p>
  <CodeBlock lang="javascript" code={`const targetInput  = document.getElementById('target');
const payloadInput = document.getElementById('payload');
const sendBtn      = document.getElementById('send');

sendBtn.addEventListener('click', () => {
  if (!authenticated) return;

  const target = targetInput.value.trim();
  if (!target) return;

  let payload;
  try {
    payload = JSON.parse(payloadInput.value);
  } catch {
    payload = { text: payloadInput.value };
  }

  ws.send(JSON.stringify({
    action: 'message',
    devices: [target],
    payload: payload
  }));

  payloadInput.value = '';
});`} />
  <p>The full dashboard is two files. Open the HTML in a browser, fill in your credentials in <code>dashboard.js</code>, and you have a live view of your device mesh.</p>
</div>

<style>
  .doc-page { max-width: 740px; }
  .doc-title { font-family: var(--font-display); font-size: var(--text-2xl); font-weight: 700; letter-spacing: 0.04em; text-transform: uppercase; margin-bottom: 8px; }
  .doc-intro { font-size: var(--text-sm); color: var(--ink-soft); line-height: var(--leading-relaxed); margin-bottom: 40px; }
  h2 { font-family: var(--font-display); font-size: var(--text-lg); font-weight: 700; letter-spacing: 0.04em; margin: 40px 0 16px; padding-bottom: 8px; border-bottom: 1px solid var(--border); }
  p { font-size: var(--text-sm); color: var(--ink-soft); line-height: var(--leading-relaxed); margin-bottom: 16px; }
  code { font-family: var(--font-body); font-size: var(--text-sm); color: var(--pulse); }
</style>
