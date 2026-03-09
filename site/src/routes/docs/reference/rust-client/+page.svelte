<script lang="ts">
  import CodeBlock from '$lib/components/ui/CodeBlock.svelte';
</script>

<svelte:head>
  <title>Rust Client - FreshBlu</title>
</svelte:head>

<div class="doc-page">
  <h1 class="doc-title">Rust Client</h1>
  <p>The <code>freshblu-client</code> crate provides an async Rust SDK for FreshBlu with HTTP and WebSocket support.</p>

  <h2>Installation</h2>
  <CodeBlock code={`# HTTP only (default)
cargo add freshblu-client

# HTTP + WebSocket
cargo add freshblu-client --features ws`} lang="bash" />

  <p>Or add to your <code>Cargo.toml</code>:</p>
  <CodeBlock code={`[dependencies]
freshblu-client = "1.0"

# With WebSocket support
freshblu-client = { version = "1.0", features = ["ws"] }`} lang="toml" />

  <h2>HTTP Client</h2>

  <h3>Constructor &amp; Authentication</h3>
  <CodeBlock code={`use freshblu_client::FreshBluClient;

let mut client = FreshBluClient::new("https://api.freshblu.org");

// Set credentials after registering or from saved config
client.set_credentials(uuid, token);

// Check current credentials
if let Some((uuid, token)) = client.credentials() {
    println!("Authenticated as {}", uuid);
}`} lang="rust" />

  <h3>Devices</h3>
  <CodeBlock code={`// Register a new device
let device = client.register(serde_json::json!({"type": "sensor"})).await?;
client.set_credentials(device.uuid, device.token.clone());

// Get authenticated device info
let me = client.whoami().await?;

// Get a device by UUID
let dev = client.get_device(&uuid).await?;

// Update device properties
let updated = client.update_device(&uuid, serde_json::json!({
    "name": "temp-sensor-01"
})).await?;

// Delete a device
client.unregister(&uuid).await?;

// Search for devices
let results = client.search(serde_json::json!({"type": "sensor"})).await?;

// List devices you own
let mine = client.my_devices().await?;

// Claim an unclaimed device
let claimed = client.claim_device(&uuid).await?;`} lang="rust" />

  <h3>Messages</h3>
  <CodeBlock code={`// Send to specific devices
client.message(
    &["target-uuid-1", "target-uuid-2"],
    serde_json::json!({"temp": 22.5})
).await?;

// Broadcast to all subscribers
client.broadcast(serde_json::json!({"alert": "high temp"})).await?;`} lang="rust" />

  <h3>Subscriptions</h3>
  <CodeBlock code={`use freshblu_client::SubscriptionType;

// Subscribe to events
client.create_subscription(
    &subscriber_uuid,
    &emitter_uuid,
    SubscriptionType::MessageReceived,
).await?;

// List subscriptions
let subs = client.subscriptions(&subscriber_uuid).await?;

// Delete a subscription
client.delete_subscription(
    &subscriber_uuid,
    &emitter_uuid,
    SubscriptionType::MessageReceived,
).await?;`} lang="rust" />

  <h3>Tokens</h3>
  <CodeBlock code={`// Generate a new token
let new_token = client.generate_token(&uuid).await?;

// Revoke a specific token
client.revoke_token(&uuid, "token-value").await?;

// Reset all tokens (returns a new one)
let reset = client.reset_token(&uuid).await?;`} lang="rust" />

  <h3>Server Status</h3>
  <CodeBlock code={`let status = client.status().await?;
println!("Server online: {}", status.meshblu);`} lang="rust" />

  <h2>WebSocket Client</h2>
  <p>Enable the <code>ws</code> feature for real-time event streaming via <code>FreshBluWs</code>.</p>

  <h3>Connection</h3>
  <CodeBlock code={`use freshblu_client::FreshBluWs;

// Client must have credentials set
let ws = FreshBluWs::connect(&client).await?;`} lang="rust" />

  <h3>Receiving Events</h3>
  <CodeBlock code={`let mut rx = ws.subscribe();

tokio::spawn(async move {
    while let Ok(event) = rx.recv().await {
        match event.event.as_str() {
            "message" => println!("Message: {:?}", event.data),
            "broadcast" => println!("Broadcast: {:?}", event.data),
            _ => println!("{}: {:?}", event.event, event.data),
        }
    }
});`} lang="rust" />

  <h3>Sending</h3>
  <CodeBlock code={`// Send a message to specific devices
ws.send_message(&["target-uuid"], serde_json::json!({"temp": 22.5})).await?;

// Broadcast
ws.send_broadcast(serde_json::json!({"alert": true})).await?;

// Subscribe to another device's events over WS
ws.subscribe_ws(&emitter_uuid, "message.received").await?;

// Send a raw JSON command
ws.send(serde_json::json!({"event": "ping"})).await?;`} lang="rust" />

  <h2>Error Handling</h2>
  <CodeBlock code={`use freshblu_client::Error;

match client.whoami().await {
    Ok(device) => println!("I am {}", device.uuid),
    Err(Error::Http { status: 401, .. }) => eprintln!("Not authenticated"),
    Err(Error::Http { status, message }) => eprintln!("HTTP {}: {}", status, message),
    Err(Error::Request(e)) => eprintln!("Network error: {}", e),
    Err(e) => eprintln!("Error: {}", e),
}`} lang="rust" />

  <h2>Complete Example</h2>
  <CodeBlock code={`use freshblu_client::{FreshBluClient, FreshBluWs};

#[tokio::main]
async fn main() -> Result<(), freshblu_client::Error> {
    let mut client = FreshBluClient::new("https://api.freshblu.org");

    // Register two devices
    let sender = client.register(serde_json::json!({"type": "sender"})).await?;
    let receiver = client.register(serde_json::json!({"type": "receiver"})).await?;

    // Receiver subscribes to sender's messages
    client.set_credentials(receiver.uuid, receiver.token.clone());
    client.create_subscription(
        &receiver.uuid,
        &sender.uuid,
        freshblu_client::SubscriptionType::MessageReceived,
    ).await?;

    // Receiver connects via WebSocket
    let ws = FreshBluWs::connect(&client).await?;
    let mut rx = ws.subscribe();

    tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            if event.event == "message" {
                println!("Received: {:?}", event.data["payload"]);
            }
        }
    });

    // Sender sends a message
    client.set_credentials(sender.uuid, sender.token.clone());
    client.message(
        &[&receiver.uuid.to_string()],
        serde_json::json!({"hello": "world"}),
    ).await?;

    Ok(())
}`} lang="rust" />

  <h2>Feature Flags</h2>
  <table class="doc-table">
    <thead>
      <tr><th>Feature</th><th>Default</th><th>Description</th></tr>
    </thead>
    <tbody>
      <tr><td><code>http</code></td><td>Yes</td><td>HTTP client via <code>reqwest</code></td></tr>
      <tr><td><code>ws</code></td><td>No</td><td>WebSocket client via <code>tokio-tungstenite</code></td></tr>
    </tbody>
  </table>
</div>

<style>
  .doc-page {
    max-width: 760px;
  }
  .doc-title {
    font-family: var(--font-display);
    font-size: var(--text-3xl);
    font-weight: 700;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    margin-bottom: 16px;
  }
  h2 {
    font-family: var(--font-display);
    font-size: var(--text-xl);
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    margin: 32px 0 12px;
    padding-top: 16px;
    border-top: 1px solid var(--border);
  }
  h3 {
    font-family: var(--font-display);
    font-size: var(--text-base);
    font-weight: 700;
    letter-spacing: 0.06em;
    margin: 24px 0 8px;
  }
  p {
    font-size: var(--text-sm);
    color: var(--ink-soft);
    line-height: var(--leading-relaxed);
    margin-bottom: 12px;
  }
  code {
    font-family: var(--font-body);
    font-size: 0.9em;
    color: var(--pulse);
    background: var(--void-lift);
    padding: 1px 5px;
  }
  .doc-table {
    width: 100%;
    border-collapse: collapse;
    margin-bottom: 16px;
    font-family: var(--font-ui);
    font-size: var(--text-sm);
  }
  .doc-table th {
    text-align: left;
    padding: 8px 12px;
    border-bottom: 2px solid var(--border);
    font-size: var(--text-xs);
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--ink-muted);
  }
  .doc-table td {
    padding: 8px 12px;
    border-bottom: 1px solid var(--border);
    color: var(--ink-soft);
  }
</style>
