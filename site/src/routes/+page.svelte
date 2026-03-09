<script lang="ts">
  import Button from '$lib/components/ui/Button.svelte';
  import Badge from '$lib/components/ui/Badge.svelte';
  import Card from '$lib/components/ui/Card.svelte';
  import Logo from '$lib/components/brand/Logo.svelte';
  import CodeBlock from '$lib/components/ui/CodeBlock.svelte';
  import LanguageTabs from '$lib/components/ui/LanguageTabs.svelte';

  const quickStartTabs = [
    {
      label: 'JavaScript',
      lang: 'javascript',
      code: `<script src="https://unpkg.com/freshblu/dist/index.global.js"><\/script>
<script>
  const client = new FreshBluHttp('https://api.freshblu.org');

  // Register a device
  const device = await client.register({ type: 'sensor' });
  console.log(device.uuid, device.token);

  // Send a message
  client.setCredentials(device.uuid, device.token);
  await client.message({ devices: ['TARGET'], payload: { temp: 22.5 } });

  // Listen via WebSocket
  const ws = new FreshBlu('https://api.freshblu.org');
  ws.setCredentials(device.uuid, device.token);
  ws.on('message', (e) => console.log(e.payload));
  await ws.connect();
<\/script>`
    },
    {
      label: 'Node.js',
      lang: 'javascript',
      code: `import { FreshBlu, FreshBluHttp } from 'freshblu';

const client = new FreshBluHttp('https://api.freshblu.org');

// Register a device
const device = await client.register({ type: 'sensor' });
client.setCredentials(device.uuid, device.token);

// Send a message
await client.message({ devices: ['TARGET'], payload: { temp: 22.5 } });

// Listen via WebSocket
const ws = new FreshBlu('https://api.freshblu.org');
ws.setCredentials(device.uuid, device.token);
ws.on('message', (e) => console.log(e.payload));
await ws.connect();`
    },
    {
      label: 'Python',
      lang: 'python',
      code: `from freshblu import FreshBluHttp

client = FreshBluHttp("https://api.freshblu.org")

# Register a device
device = client.register({"type": "sensor"})
client.set_credentials(device["uuid"], device["token"])

# Send a message
client.message({"devices": ["TARGET"], "payload": {"temp": 22.5}})

# Listen via WebSocket
from freshblu import FreshBlu
ws = FreshBlu("https://api.freshblu.org")
ws.set_credentials(device["uuid"], device["token"])
ws.on("message", lambda e: print(e["payload"]))
ws.connect()`
    },
    {
      label: 'Rust',
      lang: 'rust',
      code: `use freshblu_client::FreshBluClient;

#[tokio::main]
async fn main() -> Result<(), freshblu_client::Error> {
    let mut client = FreshBluClient::new("https://api.freshblu.org");

    // Register a device
    let device = client.register(serde_json::json!({"type": "sensor"})).await?;
    client.set_credentials(device.uuid, device.token.clone());

    // Send a message
    client.message(&["TARGET"], serde_json::json!({"temp": 22.5})).await?;

    Ok(())
}`
    },
    {
      label: 'curl',
      lang: 'bash',
      code: `# Register
curl -s -X POST https://api.freshblu.org/devices | jq .

# Send a message
CREDS=$(echo -n "$UUID:$TOKEN" | base64)
curl -X POST https://api.freshblu.org/messages \\
  -H "Authorization: Basic $CREDS" \\
  -H "Content-Type: application/json" \\
  -d '{"devices": ["TARGET"], "payload": {"temp": 22.5}}'`
    },
    {
      label: 'CLI',
      lang: 'bash',
      code: `# Install via npm
npm install -g freshblu-cli

# Or install via cargo
cargo install freshblu-cli

# Register
freshblu register --type sensor --name "temp-01"

# Send a message
freshblu message -d '{"devices":["TARGET"],"payload":{"temp": 22.5}}'

# Start a local server
freshblu server --port 3000`
    },
    {
      label: 'Arduino',
      lang: 'cpp',
      code: `#include <WiFi.h>
#include <FreshBlu.h>

WiFiClient http, mqtt;
FreshBlu blu(http, mqtt, "api.freshblu.org");

void setup() {
  WiFi.begin("SSID", "PASS");
  StaticJsonDocument<256> props;
  props["type"] = "sensor";
  blu.begin(props);
}

void loop() {
  blu.loop();
  StaticJsonDocument<128> msg;
  msg["temp"] = analogRead(A0) * 0.1;
  blu.sendMessage("TARGET_UUID", msg);
  delay(5000);
}`
    }
  ];
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
  <div class="install-strip">
    <code>npm install freshblu</code>
    <span class="install-or">or</span>
    <code>pip install freshblu</code>
    <span class="install-or">or</span>
    <code>cargo add freshblu-client</code>
    <span class="install-links">
      <a href="/docs/reference/javascript-client" class="install-link">JS docs &rarr;</a>
      <a href="/docs/reference/python-client" class="install-link">Python docs &rarr;</a>
      <a href="/docs/reference/rust-client" class="install-link">Rust docs &rarr;</a>
    </span>
  </div>
  <LanguageTabs tabs={quickStartTabs} />
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

  .install-strip {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-wrap: wrap;
    margin-bottom: 24px;
    font-family: var(--font-body);
    font-size: var(--text-sm);
    color: var(--ink-soft);
  }
  .install-strip code {
    background: var(--void);
    border: 1px solid var(--border);
    padding: 4px 10px;
    color: var(--signal);
    font-size: var(--text-sm);
  }
  .install-or {
    color: var(--ink-muted);
    font-size: var(--text-xs);
    text-transform: uppercase;
    letter-spacing: 0.1em;
  }
  .install-links {
    display: flex;
    gap: 16px;
    margin-left: auto;
  }
  .install-link {
    font-family: var(--font-ui);
    font-size: var(--text-xs);
    letter-spacing: 0.08em;
    color: var(--pulse);
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
