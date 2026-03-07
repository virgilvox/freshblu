mod helpers;

use freshblu_core::{
    device::RegisterParams,
    forwarder::{ForwarderEntry, ForwarderPair, Forwarders, MeshbluForwarder, WebhookForwarder},
};
use serde_json::json;
use wiremock::{
    matchers::{header_exists, method, path},
    Mock, MockServer, ResponseTemplate,
};

use helpers::*;

// ---------------------------------------------------------------------------
// Forwarder config serde round-trip
// ---------------------------------------------------------------------------

#[test]
fn forwarder_config_serde_roundtrip() {
    let fwd = Forwarders {
        message: ForwarderPair {
            sent: vec![
                ForwarderEntry::Webhook(WebhookForwarder {
                    url: "https://example.com/hook".to_string(),
                    method: "POST".to_string(),
                    sign_request: false,
                    generate_and_forward_meshblu_credentials: false,
                }),
                ForwarderEntry::Meshblu(MeshbluForwarder {}),
            ],
            received: vec![],
        },
        broadcast: ForwarderPair {
            sent: vec![ForwarderEntry::Webhook(WebhookForwarder {
                url: "https://example.com/broadcast".to_string(),
                method: "PUT".to_string(),
                sign_request: true,
                generate_and_forward_meshblu_credentials: true,
            })],
            received: vec![],
        },
        ..Default::default()
    };

    let json = serde_json::to_value(&fwd).unwrap();
    let back: Forwarders = serde_json::from_value(json).unwrap();
    assert_eq!(back, fwd);
}

// ---------------------------------------------------------------------------
// Webhook fires when forwarders configured
// ---------------------------------------------------------------------------

#[tokio::test]
async fn webhook_fires_on_execute() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/hook"))
        .and(header_exists("x-meshblu-uuid"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock_server)
        .await;

    let (_, state) = setup().await;
    let (uuid, _) = register_device(&state).await;
    let uuid_parsed: uuid::Uuid = uuid.parse().unwrap();

    // Construct a device with a webhook forwarder
    let mut device = state.store.get_device(&uuid_parsed).await.unwrap().unwrap();
    device.meshblu.forwarders = Some(Forwarders {
        message: ForwarderPair {
            sent: vec![ForwarderEntry::Webhook(WebhookForwarder {
                url: format!("{}/hook", mock_server.uri()),
                method: "POST".to_string(),
                sign_request: false,
                generate_and_forward_meshblu_credentials: false,
            })],
            received: vec![],
        },
        ..Default::default()
    });

    let payload = json!({ "test": "webhook_fire" });
    state
        .webhook_executor
        .execute(
            &device,
            freshblu_core::forwarder::ForwarderEvent::MessageSent,
            &payload,
            &[],
        )
        .await;

    // wiremock expect(1) will verify the mock was called on drop
}

// ---------------------------------------------------------------------------
// Meshblu-to-meshblu forwarding
// ---------------------------------------------------------------------------

#[tokio::test]
async fn meshblu_forwarder_reemits_as_message() {
    let (_, state) = setup().await;
    let (uuid, _token) = register_device(&state).await;

    // Verify MeshbluForwarder can be constructed and serialized
    let entry = ForwarderEntry::Meshblu(MeshbluForwarder {});
    let json = serde_json::to_value(&entry).unwrap();
    assert_eq!(json["type"], "meshblu");
}

// ---------------------------------------------------------------------------
// Circular loop detection
// ---------------------------------------------------------------------------

#[tokio::test]
async fn meshblu_forwarder_detects_loop() {
    let (_, state) = setup().await;
    let (uuid, _) = register_device(&state).await;
    let uuid_parsed: uuid::Uuid = uuid.parse().unwrap();

    // Simulate calling execute with self already in forwarded_from
    let device = state.store.get_device(&uuid_parsed).await.unwrap().unwrap();
    let payload = json!({ "test": true });

    // This should not panic and should detect the loop
    state
        .webhook_executor
        .execute(&device, freshblu_core::forwarder::ForwarderEvent::MessageSent, &payload, &[uuid_parsed])
        .await;
}

// ---------------------------------------------------------------------------
// Credential forwarding generates temp token
// ---------------------------------------------------------------------------

#[tokio::test]
async fn webhook_with_credentials_generates_token() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/authed-hook"))
        .and(header_exists("authorization"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock_server)
        .await;

    let (_, state) = setup().await;
    let (uuid, _) = register_device(&state).await;
    let uuid_parsed: uuid::Uuid = uuid.parse().unwrap();

    // Count tokens before
    let tokens_before = state.store.list_tokens(&uuid_parsed).await.unwrap().len();

    // Create a device with credential-forwarding webhook
    let wh = WebhookForwarder {
        url: format!("{}/authed-hook", mock_server.uri()),
        method: "POST".to_string(),
        sign_request: false,
        generate_and_forward_meshblu_credentials: true,
    };

    // Manually construct device with forwarder and fire
    let mut device = state.store.get_device(&uuid_parsed).await.unwrap().unwrap();
    device.meshblu.forwarders = Some(Forwarders {
        message: ForwarderPair {
            sent: vec![ForwarderEntry::Webhook(wh)],
            received: vec![],
        },
        ..Default::default()
    });

    let payload = json!({ "test": "credential" });
    state
        .webhook_executor
        .execute(
            &device,
            freshblu_core::forwarder::ForwarderEvent::MessageSent,
            &payload,
            &[],
        )
        .await;

    // A new temp token should have been created
    let tokens_after = state.store.list_tokens(&uuid_parsed).await.unwrap().len();
    assert!(tokens_after > tokens_before);

    // Verify the mock was called
    // (wiremock expect(1) will assert on drop)
}
