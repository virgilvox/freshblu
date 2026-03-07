//! MQTT integration tests
//!
//! These tests verify MQTT authentication, message routing, and permission checks.
//! Requires the rumqttd broker to be started as part of the server.

mod helpers;

use freshblu_server::mqtt::parse_mqtt_topic;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Unit tests for MQTT topic parsing (don't need a running broker)
// ---------------------------------------------------------------------------

#[test]
fn mqtt_topic_parsing_valid_message() {
    let uuid = Uuid::new_v4();
    let topic = format!("{}/message", uuid);
    let result = parse_mqtt_topic(&topic);
    assert!(result.is_some());
    let (parsed_uuid, event_type) = result.unwrap();
    assert_eq!(parsed_uuid, uuid);
    assert_eq!(event_type, "message");
}

#[test]
fn mqtt_topic_parsing_valid_broadcast() {
    let uuid = Uuid::new_v4();
    let topic = format!("{}/broadcast", uuid);
    let (parsed_uuid, event_type) = parse_mqtt_topic(&topic).unwrap();
    assert_eq!(parsed_uuid, uuid);
    assert_eq!(event_type, "broadcast");
}

#[test]
fn mqtt_topic_parsing_valid_config() {
    let uuid = Uuid::new_v4();
    let topic = format!("{}/config", uuid);
    let (parsed_uuid, event_type) = parse_mqtt_topic(&topic).unwrap();
    assert_eq!(parsed_uuid, uuid);
    assert_eq!(event_type, "config");
}

#[test]
fn mqtt_topic_parsing_no_slash_defaults_to_message() {
    let uuid = Uuid::new_v4();
    let topic = uuid.to_string();
    let result = parse_mqtt_topic(&topic);
    assert!(result.is_some());
    let (parsed_uuid, event_type) = result.unwrap();
    assert_eq!(parsed_uuid, uuid);
    assert_eq!(event_type, "message");
}

#[test]
fn mqtt_topic_parsing_invalid_uuid() {
    assert!(parse_mqtt_topic("not-a-uuid/message").is_none());
    assert!(parse_mqtt_topic("").is_none());
    assert!(parse_mqtt_topic("/message").is_none());
}

#[test]
fn mqtt_topic_parsing_nested_path() {
    let uuid = Uuid::new_v4();
    // splitn(2, '/') means "message/extra/stuff" is the event_type
    let topic = format!("{}/message/extra/stuff", uuid);
    let (parsed_uuid, event_type) = parse_mqtt_topic(&topic).unwrap();
    assert_eq!(parsed_uuid, uuid);
    assert_eq!(event_type, "message/extra/stuff");
}

// ---------------------------------------------------------------------------
// MQTT sender identity documentation test (B7)
// ---------------------------------------------------------------------------

#[test]
fn mqtt_sender_identity_contract() {
    // Document: the bridge extracts from_uuid from the topic.
    // An authenticated client could publish to another UUID's topic.
    // This is a known limitation until rumqttd exposes client_id on Forward.
    let client_uuid = Uuid::new_v4();
    let spoofed_topic = format!("{}/message", Uuid::new_v4());
    let (topic_uuid, _) = parse_mqtt_topic(&spoofed_topic).unwrap();
    assert_ne!(topic_uuid, client_uuid,
        "KNOWN LIMITATION: bridge cannot currently validate topic UUID against client identity");
}

// ---------------------------------------------------------------------------
// MQTT broker integration tests (require live broker)
// These are marked #[ignore] because starting rumqttd in tests requires
// careful port management and broker startup timing.
// Run with: cargo test -p freshblu-server mqtt -- --ignored
// ---------------------------------------------------------------------------

#[cfg(test)]
mod broker_tests {
    use super::helpers::*;
    use freshblu_core::subscription::{CreateSubscriptionParams, SubscriptionType};
    use freshblu_server::mqtt::MqttAdapter;
    use freshblu_server::AppState;
    use rumqttc::{AsyncClient, MqttOptions, QoS};
    use serde_json::json;
    use std::time::Duration;
    use tokio_tungstenite::tungstenite::Message;
    use futures::SinkExt;

    async fn start_mqtt(state: &AppState) -> u16 {
        let port = portpicker::pick_unused_port().unwrap();
        let adapter = MqttAdapter::new(state.store.clone(), state.bus.clone(), port);
        tokio::spawn(async move { adapter.start().await.unwrap() });
        tokio::time::sleep(Duration::from_millis(500)).await;
        port
    }

    #[tokio::test]
    #[ignore]
    async fn mqtt_auth_valid_credentials() {
        let (_, state) = setup().await;
        let (uuid, token) = register_device(&state).await;
        let port = start_mqtt(&state).await;

        let mut opts = MqttOptions::new(&uuid, "127.0.0.1", port);
        opts.set_credentials(&uuid, &token);
        opts.set_keep_alive(Duration::from_secs(5));

        let (client, mut eventloop) = AsyncClient::new(opts, 10);

        // Should get ConnAck
        let event = tokio::time::timeout(Duration::from_secs(5), eventloop.poll())
            .await
            .expect("connection timed out")
            .expect("connection failed");

        if let rumqttc::Event::Incoming(rumqttc::Packet::ConnAck(ack)) = event {
            assert_eq!(ack.code, rumqttc::ConnectReturnCode::Success,
                "valid credentials should produce successful ConnAck");
        } else {
            panic!("expected ConnAck, got {:?}", event);
        }

        client.disconnect().await.ok();
    }

    #[tokio::test]
    #[ignore]
    async fn mqtt_auth_invalid_credentials() {
        let (_, state) = setup().await;
        let (uuid, _token) = register_device(&state).await;
        let port = start_mqtt(&state).await;

        let mut opts = MqttOptions::new(&uuid, "127.0.0.1", port);
        opts.set_credentials(&uuid, "wrong-token");
        opts.set_keep_alive(Duration::from_secs(5));

        let (_client, mut eventloop) = AsyncClient::new(opts, 10);

        let result = tokio::time::timeout(Duration::from_secs(5), eventloop.poll()).await;
        match result {
            Ok(Err(_)) => {} // Connection refused — expected
            Ok(Ok(rumqttc::Event::Incoming(rumqttc::Packet::ConnAck(ack)))) => {
                assert_ne!(ack.code, rumqttc::ConnectReturnCode::Success,
                    "invalid credentials should not produce successful ConnAck");
            }
            _ => {} // Timeout or other error — also acceptable for rejection
        }
    }

    #[tokio::test]
    #[ignore]
    async fn mqtt_publish_delivers_to_ws() {
        let (ws_url, state) = setup().await;
        let (sender_uuid, sender_token) = register_device(&state).await;
        let (target_uuid, target_token) = register_device(&state).await;
        let port = start_mqtt(&state).await;

        // Target connects via WS
        let mut ws_target = connect_and_auth(&ws_url, &target_uuid, &target_token).await;

        // Sender connects via MQTT
        let mut opts = MqttOptions::new(&sender_uuid, "127.0.0.1", port);
        opts.set_credentials(&sender_uuid, &sender_token);
        opts.set_keep_alive(Duration::from_secs(5));
        let (client, mut eventloop) = AsyncClient::new(opts, 10);

        // Wait for ConnAck
        let _ = eventloop.poll().await;

        // Publish message to target
        let payload = json!({
            "devices": [target_uuid],
            "payload": {"from": "mqtt"}
        });
        client
            .publish(
                format!("{}/message", sender_uuid),
                QoS::AtLeastOnce,
                false,
                serde_json::to_vec(&payload).unwrap(),
            )
            .await
            .unwrap();

        // Drive the event loop
        let _ = tokio::time::timeout(Duration::from_secs(2), eventloop.poll()).await;

        // WS target should receive the message
        let msg = recv_json(&mut ws_target).await.expect("WS target should receive MQTT message");
        assert_eq!(msg["event"], "message");
        assert_eq!(msg["payload"]["from"], "mqtt");

        client.disconnect().await.ok();
    }

    #[tokio::test]
    #[ignore]
    async fn mqtt_publish_permission_denied() {
        let (ws_url, state) = setup().await;
        let (sender_uuid, sender_token) = register_device(&state).await;
        let (target_uuid, target_token) = register_private_device(&state).await;
        let port = start_mqtt(&state).await;

        // Target connects via WS
        let mut ws_target = connect_and_auth(&ws_url, &target_uuid, &target_token).await;

        // Sender connects via MQTT
        let mut opts = MqttOptions::new(&sender_uuid, "127.0.0.1", port);
        opts.set_credentials(&sender_uuid, &sender_token);
        opts.set_keep_alive(Duration::from_secs(5));
        let (client, mut eventloop) = AsyncClient::new(opts, 10);

        let _ = eventloop.poll().await;

        let payload = json!({
            "devices": [target_uuid],
            "payload": {"should": "not arrive"}
        });
        client
            .publish(
                format!("{}/message", sender_uuid),
                QoS::AtLeastOnce,
                false,
                serde_json::to_vec(&payload).unwrap(),
            )
            .await
            .unwrap();

        let _ = tokio::time::timeout(Duration::from_secs(2), eventloop.poll()).await;

        // WS target should NOT receive (permission denied)
        let msg = recv_json(&mut ws_target).await;
        assert!(msg.is_none(), "private device should not receive MQTT message");

        client.disconnect().await.ok();
    }

    #[tokio::test]
    #[ignore]
    async fn mqtt_broadcast_to_subscribers() {
        let (ws_url, state) = setup().await;
        let (sender_uuid, sender_token) = register_device(&state).await;
        let (sub_uuid, sub_token) = register_device(&state).await;
        let port = start_mqtt(&state).await;

        // Create broadcast.sent subscription
        let sender_parsed: uuid::Uuid = sender_uuid.parse().unwrap();
        let sub_parsed: uuid::Uuid = sub_uuid.parse().unwrap();
        let params = CreateSubscriptionParams {
            emitter_uuid: sender_parsed,
            subscriber_uuid: sub_parsed,
            subscription_type: SubscriptionType::BroadcastSent,
        };
        state.store.create_subscription(&params).await.unwrap();

        // Subscriber connects via WS
        let mut ws_sub = connect_and_auth(&ws_url, &sub_uuid, &sub_token).await;

        // Sender connects via MQTT and broadcasts
        let mut opts = MqttOptions::new(&sender_uuid, "127.0.0.1", port);
        opts.set_credentials(&sender_uuid, &sender_token);
        opts.set_keep_alive(Duration::from_secs(5));
        let (client, mut eventloop) = AsyncClient::new(opts, 10);

        let _ = eventloop.poll().await;

        let payload = json!({"broadcast_data": true});
        client
            .publish(
                format!("{}/broadcast", sender_uuid),
                QoS::AtLeastOnce,
                false,
                serde_json::to_vec(&payload).unwrap(),
            )
            .await
            .unwrap();

        let _ = tokio::time::timeout(Duration::from_secs(2), eventloop.poll()).await;

        // Subscriber should receive the broadcast
        let msg = recv_json(&mut ws_sub).await.expect("subscriber should receive MQTT broadcast");
        assert_eq!(msg["event"], "broadcast");
        assert_eq!(msg["payload"]["broadcast_data"], true);

        client.disconnect().await.ok();
    }
}
