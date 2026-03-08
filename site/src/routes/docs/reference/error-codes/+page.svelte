<svelte:head><title>Error Codes - FreshBlu Docs</title></svelte:head>
<div class="doc-page">
  <h1 class="doc-title">Error Codes</h1>
  <p class="doc-intro">All API errors return a JSON body with an <code>error</code> field containing a human-readable message. The HTTP status code indicates the error category.</p>

  <h2>Error Types</h2>

  <table class="config-table">
    <thead>
      <tr>
        <th>Error</th>
        <th>HTTP Status</th>
        <th>Message</th>
        <th>Description</th>
      </tr>
    </thead>
    <tbody>
      <tr>
        <td><code>Unauthorized</code></td>
        <td>401</td>
        <td><code>unauthorized</code></td>
        <td>Missing or invalid authentication credentials. Check your UUID and token.</td>
      </tr>
      <tr>
        <td><code>InvalidToken</code></td>
        <td>401</td>
        <td><code>invalid token</code></td>
        <td>The token exists but is invalid (expired, revoked, or malformed).</td>
      </tr>
      <tr>
        <td><code>Forbidden</code></td>
        <td>403</td>
        <td><code>forbidden</code></td>
        <td>The authenticated device does not have permission for this operation. Check the target device's whitelists.</td>
      </tr>
      <tr>
        <td><code>NotFound</code></td>
        <td>404</td>
        <td><code>device not found</code></td>
        <td>The requested device does not exist. Also returned when the device exists but the caller lacks <code>discover.view</code> permission.</td>
      </tr>
      <tr>
        <td><code>Conflict</code></td>
        <td>409</td>
        <td><code>device already exists</code></td>
        <td>A device with this UUID already exists. This can occur if a specific UUID is provided during registration.</td>
      </tr>
      <tr>
        <td><code>MessageTooLarge</code></td>
        <td>413</td>
        <td><code>message too large</code></td>
        <td>The combined size of payload and extra fields exceeds <code>max_message_size</code> (default 1 MB).</td>
      </tr>
      <tr>
        <td><code>Validation</code></td>
        <td>422</td>
        <td><code>validation error: {'{detail}'}</code></td>
        <td>Request body failed validation. The message includes specifics (e.g., "uuid required", "invalid uuid", "invalid subscription type").</td>
      </tr>
      <tr>
        <td><code>RateLimitExceeded</code></td>
        <td>429</td>
        <td><code>rate limit exceeded</code></td>
        <td>Too many requests in the current window. Default limit is 1200 requests per 60 seconds per device.</td>
      </tr>
      <tr>
        <td><code>Storage</code></td>
        <td>500</td>
        <td><code>storage error: {'{detail}'}</code></td>
        <td>Database or storage layer failure. Check server logs for details.</td>
      </tr>
      <tr>
        <td><code>Internal</code></td>
        <td>500</td>
        <td><code>internal error: {'{detail}'}</code></td>
        <td>Unexpected server error. Check server logs for details.</td>
      </tr>
    </tbody>
  </table>

  <h2>Response Format</h2>
  <p>All errors return a JSON object with a single <code>error</code> key:</p>
  <pre class="error-example"><code>{`{ "error": "device not found" }`}</code></pre>

  <h2>WebSocket Errors</h2>
  <p>Over WebSocket, errors are delivered as JSON messages with an <code>event</code> field set to <code>"error"</code>:</p>
  <pre class="error-example"><code>{`{ "event": "error", "message": "forbidden: insufficient permission to subscribe" }`}</code></pre>
  <p>Authentication failures use the <code>notReady</code> event instead:</p>
  <pre class="error-example"><code>{`{ "event": "notReady", "reason": "unauthorized" }`}</code></pre>

  <h2>Silent Failures</h2>
  <p>Some operations fail silently by design:</p>
  <ul>
    <li>Messages to devices where <code>message.from</code> is denied are silently dropped.</li>
    <li>Oversized WebSocket messages are silently dropped.</li>
    <li>Messages to unparseable UUIDs in the <code>devices</code> array are skipped.</li>
  </ul>
</div>

<style>
  .doc-page { max-width: 740px; }
  .doc-title { font-family: var(--font-display); font-size: var(--text-2xl); font-weight: 700; letter-spacing: 0.04em; text-transform: uppercase; margin-bottom: 8px; }
  .doc-intro { font-size: var(--text-sm); color: var(--ink-soft); line-height: var(--leading-relaxed); margin-bottom: 40px; }
  h2 { font-family: var(--font-display); font-size: var(--text-lg); font-weight: 700; letter-spacing: 0.04em; margin: 40px 0 16px; padding-bottom: 8px; border-bottom: 1px solid var(--border); }
  h3 { font-family: var(--font-display); font-size: var(--text-md); font-weight: 600; margin: 24px 0 8px; }
  p { font-size: var(--text-sm); color: var(--ink-soft); line-height: var(--leading-relaxed); margin-bottom: 16px; }
  ul { font-size: var(--text-sm); color: var(--ink-soft); line-height: var(--leading-relaxed); margin-bottom: 16px; padding-left: 20px; }
  li { margin-bottom: 8px; }
  code { font-family: var(--font-body); font-size: var(--text-sm); color: var(--pulse); }
  .error-example { background: var(--void); border: 1px solid var(--border); border-left: 3px solid var(--pulse); padding: 12px 16px; margin-bottom: 16px; }
  .error-example code { color: var(--ink-soft); }
  .config-table { width: 100%; border-collapse: collapse; margin-bottom: 24px; }
  .config-table th { font-family: var(--font-ui); font-size: 9px; letter-spacing: 0.15em; text-transform: uppercase; color: var(--ink-muted); text-align: left; padding: 8px 12px; border-bottom: 1px solid var(--border); }
  .config-table td { font-family: var(--font-ui); font-size: var(--text-xs); padding: 10px 12px; border-bottom: 1px solid var(--border); color: var(--ink-soft); }
  .config-table td code { color: var(--pulse); }
</style>
