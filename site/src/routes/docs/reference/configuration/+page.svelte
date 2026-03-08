<svelte:head><title>Configuration Reference - FreshBlu Docs</title></svelte:head>
<div class="doc-page">
  <h1 class="doc-title">Configuration</h1>
  <p class="doc-intro">FreshBlu is configured entirely through environment variables. A <code>.env</code> file in the working directory is loaded automatically via dotenvy. All variables have sensible defaults for local development.</p>

  <h2>Server</h2>

  <table class="config-table">
    <thead>
      <tr>
        <th>Variable</th>
        <th>Default</th>
        <th>Description</th>
      </tr>
    </thead>
    <tbody>
      <tr>
        <td><code>FRESHBLU_HTTP_PORT</code></td>
        <td><code>3000</code></td>
        <td>HTTP server listen port.</td>
      </tr>
      <tr>
        <td><code>FRESHBLU_MQTT_PORT</code></td>
        <td><code>1883</code></td>
        <td>MQTT broker listen port.</td>
      </tr>
      <tr>
        <td><code>RUST_LOG</code></td>
        <td><code>info</code></td>
        <td>Log level filter. Supports standard tracing directives (e.g., <code>debug</code>, <code>freshblu_server=trace</code>).</td>
      </tr>
    </tbody>
  </table>

  <h2>Database</h2>

  <table class="config-table">
    <thead>
      <tr>
        <th>Variable</th>
        <th>Default</th>
        <th>Description</th>
      </tr>
    </thead>
    <tbody>
      <tr>
        <td><code>DATABASE_URL</code></td>
        <td><code>sqlite:freshblu.db</code></td>
        <td>Database connection string. Use <code>sqlite:path</code> for SQLite or <code>postgresql://user:pass@host/db</code> for PostgreSQL. The store backend is selected automatically based on the URL prefix.</td>
      </tr>
    </tbody>
  </table>

  <h2>Security</h2>

  <table class="config-table">
    <thead>
      <tr>
        <th>Variable</th>
        <th>Default</th>
        <th>Description</th>
      </tr>
    </thead>
    <tbody>
      <tr>
        <td><code>FRESHBLU_PEPPER</code></td>
        <td><code>change-me-in-production</code></td>
        <td>Bcrypt pepper appended to tokens before hashing. Change this in production. Changing it invalidates all existing tokens.</td>
      </tr>
      <tr>
        <td><code>FRESHBLU_OPEN_REGISTRATION</code></td>
        <td><code>true</code></td>
        <td>Allow unauthenticated device registration. Set to <code>false</code> to require valid credentials for <code>POST /devices</code>.</td>
      </tr>
      <tr>
        <td><code>FRESHBLU_PUBLIC_KEY</code></td>
        <td>None</td>
        <td>Server public key in PEM format. Returned by <code>GET /publickey</code>.</td>
      </tr>
      <tr>
        <td><code>FRESHBLU_PRIVATE_KEY</code></td>
        <td>None</td>
        <td>Server private key in PEM format. Used for signing operations.</td>
      </tr>
    </tbody>
  </table>

  <h2>Rate Limiting</h2>

  <table class="config-table">
    <thead>
      <tr>
        <th>Variable</th>
        <th>Default</th>
        <th>Description</th>
      </tr>
    </thead>
    <tbody>
      <tr>
        <td><code>FRESHBLU_RATE_LIMIT</code></td>
        <td><code>1200</code></td>
        <td>Maximum requests per window per device.</td>
      </tr>
      <tr>
        <td><code>FRESHBLU_RATE_WINDOW</code></td>
        <td><code>60</code></td>
        <td>Rate limit window duration in seconds.</td>
      </tr>
    </tbody>
  </table>

  <h2>Messages</h2>

  <table class="config-table">
    <thead>
      <tr>
        <th>Variable</th>
        <th>Default</th>
        <th>Description</th>
      </tr>
    </thead>
    <tbody>
      <tr>
        <td><code>FRESHBLU_MAX_MESSAGE_SIZE</code></td>
        <td><code>1048576</code></td>
        <td>Maximum message size in bytes (payload + extra fields combined). Default is 1 MB. Messages exceeding this limit return 413 over HTTP or are silently dropped over WebSocket.</td>
      </tr>
    </tbody>
  </table>

  <h2>Horizontal Scaling</h2>

  <table class="config-table">
    <thead>
      <tr>
        <th>Variable</th>
        <th>Default</th>
        <th>Description</th>
      </tr>
    </thead>
    <tbody>
      <tr>
        <td><code>NATS_URL</code></td>
        <td>None</td>
        <td>NATS server URL (e.g., <code>nats://localhost:4222</code>). When set, the server uses NatsBus for cross-pod message delivery instead of the in-process LocalBus.</td>
      </tr>
      <tr>
        <td><code>REDIS_URL</code></td>
        <td>None</td>
        <td>Redis server URL (e.g., <code>redis://localhost:6379</code>). When set, enables the CachedStore layer for device caching and Redis-based presence tracking.</td>
      </tr>
      <tr>
        <td><code>POD_ID</code></td>
        <td>Hostname or random 8-char ID</td>
        <td>Unique identifier for this server instance. Used for NATS delivery routing. Defaults to the <code>HOSTNAME</code> environment variable, or a random 8-character string if neither is set.</td>
      </tr>
    </tbody>
  </table>

  <h2>Deployment Modes</h2>
  <h3>Single instance (development)</h3>
  <p>No additional configuration needed. The server uses SQLite, LocalBus, and in-memory presence by default.</p>

  <h3>Multi-instance (production)</h3>
  <p>Set <code>DATABASE_URL</code> to PostgreSQL, <code>NATS_URL</code> to your NATS cluster, and <code>REDIS_URL</code> to Redis. Each instance needs a unique <code>POD_ID</code> (typically set automatically by the container orchestrator via hostname).</p>
</div>

<style>
  .doc-page { max-width: 740px; }
  .doc-title { font-family: var(--font-display); font-size: var(--text-2xl); font-weight: 700; letter-spacing: 0.04em; text-transform: uppercase; margin-bottom: 8px; }
  .doc-intro { font-size: var(--text-sm); color: var(--ink-soft); line-height: var(--leading-relaxed); margin-bottom: 40px; }
  h2 { font-family: var(--font-display); font-size: var(--text-lg); font-weight: 700; letter-spacing: 0.04em; margin: 40px 0 16px; padding-bottom: 8px; border-bottom: 1px solid var(--border); }
  h3 { font-family: var(--font-display); font-size: var(--text-md); font-weight: 600; margin: 24px 0 8px; }
  p { font-size: var(--text-sm); color: var(--ink-soft); line-height: var(--leading-relaxed); margin-bottom: 16px; }
  code { font-family: var(--font-body); font-size: var(--text-sm); color: var(--pulse); }
  .config-table { width: 100%; border-collapse: collapse; margin-bottom: 24px; }
  .config-table th { font-family: var(--font-ui); font-size: 9px; letter-spacing: 0.15em; text-transform: uppercase; color: var(--ink-muted); text-align: left; padding: 8px 12px; border-bottom: 1px solid var(--border); }
  .config-table td { font-family: var(--font-ui); font-size: var(--text-xs); padding: 10px 12px; border-bottom: 1px solid var(--border); color: var(--ink-soft); }
  .config-table td code { color: var(--pulse); }
</style>
