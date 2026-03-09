# freshblu-client

Rust client SDK for the [FreshBlu](https://github.com/virgilvox/freshblu) IoT messaging platform.

## Features

- **HTTP client** (default) — register devices, send messages, manage subscriptions and tokens
- **WebSocket client** — real-time event streaming with `tokio` async runtime

## Installation

```toml
[dependencies]
freshblu-client = "1.0"
```

Enable WebSocket support:

```toml
[dependencies]
freshblu-client = { version = "1.0", features = ["ws"] }
```

## Quick Start

```rust
use freshblu_client::FreshBluClient;

#[tokio::main]
async fn main() -> Result<(), freshblu_client::Error> {
    let mut client = FreshBluClient::new("https://api.freshblu.org");

    // Register a device
    let device = client.register(serde_json::json!({"type": "sensor"})).await?;
    client.set_credentials(device.uuid, device.token.clone());

    // Send a message
    client.message(&["target-uuid"], serde_json::json!({"temp": 22.5})).await?;

    Ok(())
}
```

## HTTP API

All HTTP methods are available when the `http` feature is enabled (default):

```rust
let mut client = FreshBluClient::new("https://api.freshblu.org");

// Device management
let device = client.register(serde_json::json!({"type": "sensor"})).await?;
client.set_credentials(device.uuid, device.token.clone());
let me = client.whoami().await?;
let dev = client.get_device(&uuid).await?;
let updated = client.update_device(&uuid, serde_json::json!({"name": "new"})).await?;
client.unregister(&uuid).await?;
let results = client.search(serde_json::json!({"type": "sensor"})).await?;
let mine = client.my_devices().await?;

// Messaging
client.message(&["target-uuid"], serde_json::json!({"temp": 22.5})).await?;
client.broadcast(serde_json::json!({"alert": true})).await?;

// Subscriptions
use freshblu_client::SubscriptionType;
client.create_subscription(&sub_uuid, &emit_uuid, SubscriptionType::MessageReceived).await?;
let subs = client.subscriptions(&sub_uuid).await?;
client.delete_subscription(&sub_uuid, &emit_uuid, SubscriptionType::MessageReceived).await?;

// Tokens
let new_token = client.generate_token(&uuid).await?;
client.revoke_token(&uuid, "token-value").await?;
let reset = client.reset_token(&uuid).await?;

// Status
let status = client.status().await?;
```

## WebSocket Client

Enable the `ws` feature for real-time event streaming:

```rust
use freshblu_client::{FreshBluClient, ws::FreshBluWs};

let client = FreshBluClient::new("https://api.freshblu.org");
// ... set credentials ...

let ws = FreshBluWs::connect(&client).await?;
let mut rx = ws.subscribe();

// Listen for events
tokio::spawn(async move {
    while let Ok(event) = rx.recv().await {
        println!("{}: {:?}", event.event, event.data);
    }
});

// Send messages
ws.send_message(&["target-uuid"], serde_json::json!({"temp": 22.5})).await?;
ws.send_broadcast(serde_json::json!({"alert": true})).await?;
```

## Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `http`  | Yes     | HTTP client via `reqwest` |
| `ws`    | No      | WebSocket client via `tokio-tungstenite` |

## License

MIT OR Apache-2.0
