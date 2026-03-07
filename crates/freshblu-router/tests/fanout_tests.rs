//! Router and proto component tests

use freshblu_core::device::{DeviceView, MeshbluMeta};
use freshblu_core::message::Message;
use freshblu_core::permissions::Whitelists;
use freshblu_proto::{DeliveryEnvelope, NatsEvent};
use std::collections::HashMap;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Subject builder tests
// ---------------------------------------------------------------------------

#[test]
fn subject_builders_device_inbox() {
    let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
    assert_eq!(
        freshblu_proto::device_inbox(&uuid),
        "freshblu.device.550e8400-e29b-41d4-a716-446655440000.inbox"
    );
}

#[test]
fn subject_builders_broadcast() {
    let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
    assert_eq!(
        freshblu_proto::broadcast(&uuid),
        "freshblu.broadcast.550e8400-e29b-41d4-a716-446655440000"
    );
}

#[test]
fn subject_builders_configure() {
    let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
    assert_eq!(
        freshblu_proto::configure(&uuid),
        "freshblu.configure.550e8400-e29b-41d4-a716-446655440000"
    );
}

#[test]
fn subject_builders_delivery() {
    assert_eq!(freshblu_proto::delivery("pod-42"), "freshblu.delivery.pod-42");
}

#[test]
fn subject_builders_presence() {
    let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
    assert_eq!(
        freshblu_proto::presence(&uuid),
        "freshblu.presence.550e8400-e29b-41d4-a716-446655440000"
    );
}

#[test]
fn subject_builders_system_unregister() {
    assert_eq!(freshblu_proto::system_unregister(), "freshblu.system.unregister");
}

// ---------------------------------------------------------------------------
// Envelope roundtrip tests for all NatsEvent variants
// ---------------------------------------------------------------------------

fn make_msg() -> Message {
    Message {
        devices: vec!["target".into()],
        from_uuid: Some(Uuid::new_v4()),
        topic: Some("test-topic".into()),
        payload: Some(serde_json::json!({"key": "value"})),
        metadata: None,
        extra: HashMap::new(),
    }
}

fn make_device_view(uuid: Uuid) -> DeviceView {
    DeviceView {
        uuid,
        online: true,
        device_type: Some("test".into()),
        meshblu: MeshbluMeta::new(Whitelists::open()),
        properties: HashMap::new(),
    }
}

#[test]
fn envelope_roundtrip_message() {
    let target = Uuid::new_v4();
    let from = Uuid::new_v4();
    let envelope = DeliveryEnvelope {
        target,
        event: NatsEvent::Message {
            from,
            msg: make_msg(),
        },
        source_pod: "pod-1".into(),
    };
    let json = serde_json::to_string(&envelope).unwrap();
    let decoded: DeliveryEnvelope = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.target, target);
    assert_eq!(decoded.source_pod, "pod-1");
    match &decoded.event {
        NatsEvent::Message { from: f, msg } => {
            assert_eq!(*f, from);
            assert_eq!(msg.topic, Some("test-topic".into()));
        }
        _ => panic!("expected Message variant"),
    }
}

#[test]
fn envelope_roundtrip_broadcast() {
    let target = Uuid::new_v4();
    let from = Uuid::new_v4();
    let envelope = DeliveryEnvelope {
        target,
        event: NatsEvent::Broadcast {
            from,
            msg: make_msg(),
        },
        source_pod: "pod-2".into(),
    };
    let json = serde_json::to_string(&envelope).unwrap();
    let decoded: DeliveryEnvelope = serde_json::from_str(&json).unwrap();
    assert!(matches!(decoded.event, NatsEvent::Broadcast { .. }));
}

#[test]
fn envelope_roundtrip_config_update() {
    let uuid = Uuid::new_v4();
    let envelope = DeliveryEnvelope {
        target: uuid,
        event: NatsEvent::ConfigUpdate {
            uuid,
            device: make_device_view(uuid),
        },
        source_pod: "pod-3".into(),
    };
    let json = serde_json::to_string(&envelope).unwrap();
    let decoded: DeliveryEnvelope = serde_json::from_str(&json).unwrap();
    match &decoded.event {
        NatsEvent::ConfigUpdate {
            uuid: u,
            device: dv,
        } => {
            assert_eq!(*u, uuid);
            assert_eq!(dv.uuid, uuid);
        }
        _ => panic!("expected ConfigUpdate variant"),
    }
}

#[test]
fn envelope_roundtrip_unregister() {
    let uuid = Uuid::new_v4();
    let envelope = DeliveryEnvelope {
        target: uuid,
        event: NatsEvent::Unregister { uuid },
        source_pod: "pod-4".into(),
    };
    let json = serde_json::to_string(&envelope).unwrap();
    let decoded: DeliveryEnvelope = serde_json::from_str(&json).unwrap();
    match &decoded.event {
        NatsEvent::Unregister { uuid: u } => assert_eq!(*u, uuid),
        _ => panic!("expected Unregister variant"),
    }
}

// ---------------------------------------------------------------------------
// extract_uuid_from_subject tests (function is not pub, so we test the
// behavior via the subject format expectations)
// ---------------------------------------------------------------------------

#[test]
fn extract_uuid_device_inbox_format() {
    let uuid = Uuid::new_v4();
    let subject = freshblu_proto::device_inbox(&uuid);
    // Should be: freshblu.device.{uuid}.inbox
    assert!(subject.starts_with("freshblu.device."));
    assert!(subject.ends_with(".inbox"));

    // Extract the UUID part
    let stripped = subject
        .strip_prefix("freshblu.device.")
        .unwrap()
        .strip_suffix(".inbox")
        .unwrap();
    let parsed = Uuid::parse_str(stripped).unwrap();
    assert_eq!(parsed, uuid);
}

#[test]
fn extract_uuid_broadcast_format() {
    let uuid = Uuid::new_v4();
    let subject = freshblu_proto::broadcast(&uuid);
    let stripped = subject.strip_prefix("freshblu.broadcast.").unwrap();
    let parsed = Uuid::parse_str(stripped).unwrap();
    assert_eq!(parsed, uuid);
}

#[test]
fn extract_uuid_invalid_prefix() {
    // If someone tries to parse a non-freshblu subject, it shouldn't work
    let result = Uuid::parse_str("not.a.uuid");
    assert!(result.is_err());
}
