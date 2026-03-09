<script>
  import CodeBlock from '$lib/components/ui/CodeBlock.svelte';
</script>

<svelte:head><title>HTTP API Reference - FreshBlu Docs</title></svelte:head>
<div class="doc-page">
  <h1 class="doc-title">HTTP API Reference</h1>
  <p class="doc-intro">Complete reference for all HTTP endpoints. A public instance is available at <code>https://api.freshblu.org</code>. Authentication uses HTTP Basic Auth with <code>uuid:token</code> as username:password. Legacy headers <code>skynet_auth_uuid</code> and <code>skynet_auth_token</code> are also supported. The <code>x-meshblu-as</code> header allows acting as another device when permitted.</p>

  <h2>Status</h2>

  <div class="endpoint">
    <span class="endpoint-method">GET</span>
    <span class="endpoint-path">/status</span>
    <p class="endpoint-desc">Returns server status. No authentication required.</p>
    <CodeBlock lang="json" code={`{
  "meshblu": true,
  "version": "2.0.0",
  "online": true,
  "connections": 42,
  "engine": "freshblu"
}`} />
  </div>

  <div class="endpoint">
    <span class="endpoint-method">GET</span>
    <span class="endpoint-path">/healthcheck</span>
    <p class="endpoint-desc">Verifies store connectivity. Returns 200 if healthy, 503 if the database is unreachable. No authentication required.</p>
    <CodeBlock lang="json" code={`{ "healthy": true }`} />
  </div>

  <div class="endpoint">
    <span class="endpoint-method">GET</span>
    <span class="endpoint-path">/metrics</span>
    <p class="endpoint-desc">Prometheus metrics endpoint. No authentication required.</p>
  </div>

  <h2>Authentication</h2>

  <div class="endpoint">
    <span class="endpoint-method">POST</span>
    <span class="endpoint-path">/authenticate</span>
    <p class="endpoint-desc">Verify device credentials. No auth header required. Credentials are passed in the request body.</p>
    <h3>Request Body</h3>
    <CodeBlock lang="json" code={`{
  "uuid": "device-uuid",
  "token": "device-token"
}`} />
    <h3>Response</h3>
    <CodeBlock lang="json" code={`{ "uuid": "device-uuid" }`} />
  </div>

  <h2>Device Registration</h2>

  <div class="endpoint">
    <span class="endpoint-method">POST</span>
    <span class="endpoint-path">/devices</span>
    <p class="endpoint-desc">Register a new device. When <code>open_registration</code> is enabled (default), no auth is required. When disabled, valid auth credentials must be provided. Also available at <code>/v2/devices</code>.</p>
    <h3>Request Body</h3>
    <CodeBlock lang="json" code={`{
  "name": "my-sensor",
  "type": "sensor",
  "owner": "owner-uuid"
}`} />
    <h3>Response</h3>
    <p class="endpoint-desc">Returns the full device object with a plaintext <code>token</code> field. This is the only time the token is returned in cleartext.</p>
    <CodeBlock lang="json" code={`{
  "uuid": "generated-uuid",
  "token": "plaintext-token",
  "name": "my-sensor",
  "type": "sensor",
  "meshblu": { ... }
}`} />
  </div>

  <h2>Device Management</h2>

  <div class="endpoint">
    <span class="endpoint-method">GET</span>
    <span class="endpoint-path">/devices/:uuid</span>
    <p class="endpoint-desc">Get a device by UUID. Requires <code>discover.view</code> permission on the target device. Returns 404 if not found or not permitted. Also available at <code>/v2/devices/:uuid</code> and <code>/v3/devices/:uuid</code>.</p>
    <h3>Response</h3>
    <CodeBlock lang="json" code={`{
  "uuid": "device-uuid",
  "name": "my-sensor",
  "type": "sensor",
  "online": false,
  "meshblu": { ... }
}`} />
  </div>

  <div class="endpoint">
    <span class="endpoint-method">PUT</span>
    <span class="endpoint-path">/devices/:uuid</span>
    <p class="endpoint-desc">Update device properties. Requires <code>configure.update</code> permission. Emits a <code>config</code> event to the device and all <code>configure.sent</code> subscribers. Fires <code>configure.sent</code> forwarders. Also available at <code>/v2/devices/:uuid</code>.</p>
    <h3>Request Body</h3>
    <CodeBlock lang="json" code={`{
  "name": "updated-name",
  "color": "blue"
}`} />
    <h3>Response</h3>
    <p class="endpoint-desc">Returns the updated device view.</p>
  </div>

  <div class="endpoint">
    <span class="endpoint-method">DELETE</span>
    <span class="endpoint-path">/devices/:uuid</span>
    <p class="endpoint-desc">Unregister (delete) a device. Requires <code>configure.update</code> permission. Notifies <code>unregister.sent</code> subscribers before deletion. Also available at <code>/v2/devices/:uuid</code>.</p>
    <h3>Response</h3>
    <CodeBlock lang="json" code={`{ "uuid": "device-uuid" }`} />
  </div>

  <div class="endpoint">
    <span class="endpoint-method">POST</span>
    <span class="endpoint-path">/devices/search</span>
    <p class="endpoint-desc">Search devices by property filters. Results are filtered to only devices the authenticated actor can discover. Also available at <code>/v2/devices/search</code>.</p>
    <h3>Request Body</h3>
    <CodeBlock lang="json" code={`{
  "type": "sensor",
  "online": true
}`} />
    <h3>Response</h3>
    <p class="endpoint-desc">Returns an array of device views matching the query.</p>
  </div>

  <h2>Identity</h2>

  <div class="endpoint">
    <span class="endpoint-method">GET</span>
    <span class="endpoint-path">/whoami</span>
    <p class="endpoint-desc">Returns the authenticated device's own properties. Also available at <code>/v2/whoami</code>.</p>
  </div>

  <div class="endpoint">
    <span class="endpoint-method">GET</span>
    <span class="endpoint-path">/mydevices</span>
    <p class="endpoint-desc">Returns devices owned by the authenticated device.</p>
  </div>

  <h2>Messaging</h2>

  <div class="endpoint">
    <span class="endpoint-method">POST</span>
    <span class="endpoint-path">/messages</span>
    <p class="endpoint-desc">Send a message to one or more devices. Requires <code>message.from</code> permission on each target device. If <code>devices</code> contains <code>"*"</code>, the message is treated as a broadcast and delivered to all <code>broadcast.sent</code> subscribers. Also available at <code>/v2/messages</code>.</p>
    <h3>Request Body</h3>
    <CodeBlock lang="json" code={`{
  "devices": ["target-uuid"],
  "topic": "temperature",
  "payload": { "value": 22.5, "unit": "C" }
}`} />
    <h3>Response</h3>
    <CodeBlock lang="json" code={`{ "sent": true }`} />
  </div>

  <div class="endpoint">
    <span class="endpoint-method">POST</span>
    <span class="endpoint-path">/broadcasts</span>
    <p class="endpoint-desc">Broadcast a message. Equivalent to sending a message with <code>devices: ["*"]</code>. The <code>devices</code> field in the body is ignored.</p>
    <h3>Request Body</h3>
    <CodeBlock lang="json" code={`{
  "topic": "status-update",
  "payload": { "status": "online" }
}`} />
  </div>

  <h2>Subscriptions</h2>

  <div class="endpoint">
    <span class="endpoint-method">POST</span>
    <span class="endpoint-path">/devices/:uuid/subscriptions</span>
    <p class="endpoint-desc">Create a subscription for the device at <code>:uuid</code>. Requires <code>configure.update</code> permission on the subscriber if acting on behalf of another device. Permission on the emitter is checked based on subscription type.</p>
    <h3>Request Body</h3>
    <CodeBlock lang="json" code={`{
  "emitterUuid": "emitter-device-uuid",
  "subscriberUuid": "ignored-overridden-by-path",
  "type": "broadcast-sent"
}`} />
  </div>

  <div class="endpoint">
    <span class="endpoint-method">GET</span>
    <span class="endpoint-path">/devices/:uuid/subscriptions</span>
    <p class="endpoint-desc">List all subscriptions for a device. Requires <code>configure.update</code> permission if listing another device's subscriptions.</p>
  </div>

  <div class="endpoint">
    <span class="endpoint-method">DELETE</span>
    <span class="endpoint-path">/devices/:uuid/subscriptions/:emitter_uuid/:sub_type</span>
    <p class="endpoint-desc">Delete a specific subscription. The <code>sub_type</code> uses dot notation (e.g., <code>broadcast.sent</code>) or hyphens (e.g., <code>broadcast-sent</code>).</p>
  </div>

  <h2>Tokens</h2>

  <div class="endpoint">
    <span class="endpoint-method">POST</span>
    <span class="endpoint-path">/devices/:uuid/tokens</span>
    <p class="endpoint-desc">Generate a new session token. Requires <code>configure.update</code> permission.</p>
    <h3>Request Body (optional)</h3>
    <CodeBlock lang="json" code={`{
  "tag": "mqtt-session",
  "expiresOn": 1735689600
}`} />
    <h3>Response</h3>
    <CodeBlock lang="json" code={`{
  "uuid": "device-uuid",
  "token": "new-plaintext-token",
  "createdAt": "2026-03-07T00:00:00Z"
}`} />
  </div>

  <div class="endpoint">
    <span class="endpoint-method">DELETE</span>
    <span class="endpoint-path">/devices/:uuid/tokens/:token</span>
    <p class="endpoint-desc">Revoke a specific token. Requires <code>configure.update</code> permission.</p>
    <h3>Response</h3>
    <CodeBlock lang="json" code={`{ "revoked": true }`} />
  </div>

  <div class="endpoint">
    <span class="endpoint-method">POST</span>
    <span class="endpoint-path">/devices/:uuid/token</span>
    <p class="endpoint-desc">Reset token. Revokes all existing tokens and generates a new root token. Requires <code>configure.update</code> permission.</p>
    <h3>Response</h3>
    <CodeBlock lang="json" code={`{
  "uuid": "device-uuid",
  "token": "new-root-token"
}`} />
  </div>

  <div class="endpoint">
    <span class="endpoint-method">POST</span>
    <span class="endpoint-path">/search/tokens</span>
    <p class="endpoint-desc">Search tokens by tag, device, or expiry. Results are scoped to the authenticated device only.</p>
    <h3>Response</h3>
    <CodeBlock lang="json" code={`[
  {
    "uuid": "device-uuid",
    "createdAt": "2026-03-07T00:00:00Z",
    "expiresOn": null,
    "tag": "mqtt-session"
  }
]`} />
  </div>

  <h2>Claim Device</h2>

  <div class="endpoint">
    <span class="endpoint-method">POST</span>
    <span class="endpoint-path">/claimdevice/:uuid</span>
    <p class="endpoint-desc">Claim ownership of a device.</p>
    <h3>Response</h3>
    <CodeBlock lang="json" code={`{
  "uuid": "claimed-device-uuid",
  "owner": "your-uuid"
}`} />
  </div>

  <h2>Public Keys</h2>

  <div class="endpoint">
    <span class="endpoint-method">GET</span>
    <span class="endpoint-path">/devices/:uuid/publickey</span>
    <p class="endpoint-desc">Get a device's public key. No special permission required beyond authentication.</p>
    <h3>Response</h3>
    <CodeBlock lang="json" code={`{
  "uuid": "device-uuid",
  "publicKey": "-----BEGIN PUBLIC KEY-----..."
}`} />
  </div>

  <div class="endpoint">
    <span class="endpoint-method">GET</span>
    <span class="endpoint-path">/publickey</span>
    <p class="endpoint-desc">Get the server's global public key. No authentication required.</p>
    <h3>Response</h3>
    <CodeBlock lang="json" code={`{ "publicKey": "-----BEGIN PUBLIC KEY-----..." }`} />
  </div>

  <h2>Firehose / SSE</h2>

  <div class="endpoint">
    <span class="endpoint-method">GET</span>
    <span class="endpoint-path">/subscribe</span>
    <p class="endpoint-desc">Server-Sent Events stream for the authenticated device. Receives all events (messages, config changes, broadcasts, unregister) in real time. The device is marked online for the duration of the connection. Sets the device offline and disconnects from the bus on stream close.</p>
  </div>

  <h2>WebSocket</h2>

  <div class="endpoint">
    <span class="endpoint-method">GET</span>
    <span class="endpoint-path">/ws</span>
    <p class="endpoint-desc">WebSocket upgrade endpoint. Also available at <code>/socket.io</code> for Meshblu Socket.io client compatibility. See the <a href="/docs/reference/websocket-protocol">WebSocket Protocol</a> reference for the message format.</p>
  </div>

  <h2>Error Responses</h2>
  <p>All error responses follow the same format:</p>
  <CodeBlock lang="json" code={`{ "error": "description of the error" }`} />
  <p>See the <a href="/docs/reference/error-codes">Error Codes</a> reference for all possible error types and their HTTP status codes.</p>

  <h2>Rate Limiting</h2>
  <p>All authenticated endpoints enforce rate limiting. The default is 1200 requests per 60-second window per device. When exceeded, the server returns <code>429 Too Many Requests</code>. Configure limits with the <code>FRESHBLU_RATE_LIMIT</code> and <code>FRESHBLU_RATE_WINDOW</code> environment variables.</p>
</div>

<style>
  .doc-page { max-width: 740px; }
  .doc-title { font-family: var(--font-display); font-size: var(--text-2xl); font-weight: 700; letter-spacing: 0.04em; text-transform: uppercase; margin-bottom: 8px; }
  .doc-intro { font-size: var(--text-sm); color: var(--ink-soft); line-height: var(--leading-relaxed); margin-bottom: 40px; }
  h2 { font-family: var(--font-display); font-size: var(--text-lg); font-weight: 700; letter-spacing: 0.04em; margin: 40px 0 16px; padding-bottom: 8px; border-bottom: 1px solid var(--border); }
  h3 { font-family: var(--font-display); font-size: var(--text-md); font-weight: 600; margin: 24px 0 8px; }
  p { font-size: var(--text-sm); color: var(--ink-soft); line-height: var(--leading-relaxed); margin-bottom: 16px; }
  code { font-family: var(--font-body); font-size: var(--text-sm); color: var(--pulse); }
  a { color: var(--pulse); text-decoration: none; }
  a:hover { text-decoration: underline; }
  .endpoint { margin-bottom: 32px; padding: 16px; border: 1px solid var(--border); background: var(--void-lift); }
  .endpoint-method { font-family: var(--font-display); font-size: var(--text-sm); font-weight: 700; letter-spacing: 0.1em; text-transform: uppercase; color: var(--signal); }
  .endpoint-path { font-family: var(--font-body); font-size: var(--text-sm); color: var(--pulse); margin-left: 8px; }
  .endpoint-desc { font-size: var(--text-sm); color: var(--ink-soft); margin-top: 8px; }
</style>
