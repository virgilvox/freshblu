<script lang="ts">
  import Button from '$lib/components/ui/Button.svelte';
  import Badge from '$lib/components/ui/Badge.svelte';
  import Card from '$lib/components/ui/Card.svelte';
  import Logo from '$lib/components/brand/Logo.svelte';
  import CodeBlock from '$lib/components/ui/CodeBlock.svelte';
</script>

<svelte:head>
  <title>FreshBlu - IoT Messaging Platform</title>
</svelte:head>

<!-- Hero -->
<section class="hero">
  <div class="hero-inner">
    <div class="hero-eyebrow">Meshblu-Compatible IoT Mesh</div>
    <h1 class="hero-title">FRESH<span>BLU</span></h1>
    <p class="hero-subtitle">Signal-driven device messaging. Built in Rust.</p>
    <div class="hero-actions">
      <Button href="/docs/tutorials/getting-started" size="lg">Get Started</Button>
      <Button href="/playground" variant="ghost" size="lg">
        <i class="fa-solid fa-terminal"></i>
        Playground
      </Button>
    </div>
    <div class="hero-meta">
      <span class="hero-meta-item">Public Server <span>api.freshblu.org</span></span>
      <span class="hero-meta-item">Protocol <span>HTTP / WS / MQTT</span></span>
      <span class="hero-meta-item">Runtime <span>Rust + Axum</span></span>
    </div>
  </div>
  <div class="hero-mark">
    <Logo size={180} />
  </div>
</section>

<!-- Protocol badges -->
<section class="protocols">
  <div class="protocol-trace">
    <span class="proto-tag">HTTP REST</span>
    <span class="proto-line"></span>
    <span class="proto-tag">WebSocket</span>
    <span class="proto-line"></span>
    <span class="proto-tag">MQTT 3.1.1</span>
    <span class="proto-line"></span>
    <span class="proto-dot"></span>
  </div>
</section>

<!-- Features -->
<section class="features">
  <div class="section-header">
    <span class="section-num">01</span>
    <span class="section-title">Capabilities</span>
  </div>
  <div class="card-grid">
    <Card variant="pulse" meta="Messaging">
      <p class="card-text">Send messages between devices over HTTP, WebSocket, or MQTT. Subscriptions route events automatically.</p>
    </Card>
    <Card variant="signal" meta="Permissions">
      <p class="card-text">Fine-grained whitelists control who can discover, configure, send to, and receive from each device.</p>
    </Card>
    <Card variant="pulse" meta="Multi-Protocol">
      <p class="card-text">Connect via REST API, persistent WebSocket, or MQTT broker. All protocols share the same device mesh.</p>
    </Card>
    <Card meta="Subscriptions">
      <p class="card-text">Subscribe to message.received, message.sent, configure.received, and more. Fan-out across connected clients.</p>
    </Card>
    <Card meta="Tokens">
      <p class="card-text">Generate multiple session tokens per device. Revoke individually or reset all. bcrypt-hashed storage.</p>
    </Card>
    <Card meta="Webhooks">
      <p class="card-text">Configure HTTP forwarders per event type. Outbound webhooks fire on message delivery, config changes, and more.</p>
    </Card>
  </div>
</section>

<!-- Quick start -->
<section class="quickstart">
  <div class="section-header">
    <span class="section-num">02</span>
    <span class="section-title">Quick Start</span>
  </div>
  <div class="qs-steps">
    <div class="qs-step">
      <span class="qs-num">01</span>
      <span class="qs-label">Register a device and save the credentials</span>
      <CodeBlock lang="bash" code={`# Register a new device on the public server
curl -s -X POST https://api.freshblu.org/devices | jq .

# Response:
# { "uuid": "d0a1f3b2-...", "token": "a8c3e9...", "meshblu": { ... } }

# Save the uuid and token - the token is shown only once
UUID="your-uuid-here"
TOKEN="your-token-here"
CREDS=$(echo -n "$UUID:$TOKEN" | base64)`} />
    </div>
    <div class="qs-step">
      <span class="qs-num">02</span>
      <span class="qs-label">Send a message to another device</span>
      <CodeBlock lang="bash" code={`curl -X POST https://api.freshblu.org/messages \\
  -H "Authorization: Basic $CREDS" \\
  -H "Content-Type: application/json" \\
  -d '{"devices": ["TARGET_UUID"], "payload": {"hello": "world"}}'`} />
    </div>
    <div class="qs-step">
      <span class="qs-num">03</span>
      <span class="qs-label">Listen via WebSocket</span>
      <CodeBlock lang="javascript" code={`const ws = new WebSocket('wss://api.freshblu.org/ws');
ws.send(JSON.stringify({event: 'identity', uuid: 'YOUR_UUID', token: 'YOUR_TOKEN'}));
ws.onmessage = (e) => console.log(JSON.parse(e.data));`} />
    </div>
  </div>
</section>

<!-- Stack -->
<section class="status-section">
  <div class="section-header">
    <span class="section-num">03</span>
    <span class="section-title">Stack</span>
  </div>
  <div class="badge-row">
    <Badge variant="pulse">Rust</Badge>
    <Badge variant="pulse">Axum</Badge>
    <Badge variant="online">PostgreSQL</Badge>
    <Badge variant="online">Redis</Badge>
    <Badge variant="pending">NATS</Badge>
    <Badge variant="muted">SQLite</Badge>
    <Badge variant="muted">Arduino SDK</Badge>
    <Badge variant="muted">WASM Client</Badge>
  </div>
</section>

<style>
  .hero {
    padding: 80px 40px 64px;
    border-bottom: 1px solid var(--border);
    position: relative;
    overflow: hidden;
    max-width: 1200px;
    margin: 0 auto;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 40px;
  }
  .hero-inner { flex: 1; }
  .hero-mark {
    flex-shrink: 0;
    opacity: 0.8;
  }
  .hero-eyebrow {
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    letter-spacing: 0.2em;
    text-transform: uppercase;
    color: var(--pulse);
    margin-bottom: 16px;
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .hero-eyebrow::before {
    content: '';
    display: block;
    width: 24px;
    height: 1px;
    background: var(--pulse);
  }
  .hero-title {
    font-family: var(--font-display);
    font-size: clamp(48px, 8vw, 80px);
    font-weight: 700;
    line-height: 1.0;
    letter-spacing: 0.03em;
    margin-bottom: 6px;
  }
  .hero-title span { color: var(--pulse); }
  .hero-subtitle {
    font-family: var(--font-display);
    font-size: clamp(18px, 3vw, 28px);
    font-weight: 300;
    color: var(--ink-soft);
    letter-spacing: 0.08em;
    margin-bottom: 32px;
  }
  .hero-actions {
    display: flex;
    gap: 12px;
    margin-bottom: 40px;
    flex-wrap: wrap;
  }
  .hero-meta {
    display: flex;
    gap: 32px;
    flex-wrap: wrap;
  }
  .hero-meta-item {
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--ink-muted);
  }
  .hero-meta-item span { color: var(--ink-soft); }

  .protocols {
    max-width: 1200px;
    margin: 0 auto;
    padding: 32px 40px;
  }
  .protocol-trace {
    display: flex;
    align-items: center;
  }
  .proto-tag {
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    letter-spacing: 0.1em;
    text-transform: uppercase;
    background: var(--pulse-dim);
    color: var(--pulse);
    padding: 4px 10px;
    border: 1px solid var(--pulse);
    white-space: nowrap;
  }
  .proto-line {
    flex: 1;
    height: 1px;
    background: var(--pulse);
    opacity: 0.4;
  }
  .proto-dot {
    width: 8px;
    height: 8px;
    background: var(--pulse);
  }

  .features, .quickstart, .status-section {
    max-width: 1200px;
    margin: 0 auto;
    padding: 80px 40px 0;
  }
  .section-header {
    display: flex;
    align-items: baseline;
    gap: 16px;
    margin-bottom: 40px;
    padding-bottom: 16px;
    border-bottom: 1px solid var(--border);
  }
  .section-num {
    font-family: var(--font-display);
    font-size: var(--text-xs);
    font-weight: 700;
    letter-spacing: 0.15em;
    color: var(--pulse);
    text-transform: uppercase;
  }
  .section-title {
    font-family: var(--font-display);
    font-size: var(--text-2xl);
    font-weight: 700;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }
  .card-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 12px;
  }
  .card-text {
    font-size: var(--text-sm);
    color: var(--ink-soft);
    line-height: var(--leading-relaxed);
  }

  .qs-steps {
    display: flex;
    flex-direction: column;
    gap: 32px;
  }
  .qs-step {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .qs-num {
    font-family: var(--font-display);
    font-size: var(--text-xs);
    font-weight: 700;
    letter-spacing: 0.15em;
    color: var(--pulse);
  }
  .qs-label {
    font-family: var(--font-display);
    font-size: var(--text-md);
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .badge-row {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }

  .status-section {
    padding-bottom: 80px;
  }

  @media (max-width: 900px) {
    .card-grid { grid-template-columns: 1fr; }
    .hero { flex-direction: column; padding: 48px 24px; }
    .hero-mark { display: none; }
  }
  @media (max-width: 600px) {
    .features, .quickstart, .status-section, .protocols {
      padding-left: 16px;
      padding-right: 16px;
    }
  }
</style>
