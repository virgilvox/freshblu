use uuid::Uuid;

/// NATS subject for delivering messages to a specific device's inbox.
pub fn device_inbox(uuid: &Uuid) -> String {
    format!("freshblu.device.{}.inbox", uuid)
}

/// NATS subject for broadcast events from a device.
pub fn broadcast(uuid: &Uuid) -> String {
    format!("freshblu.broadcast.{}", uuid)
}

/// NATS subject for configure events on a device.
pub fn configure(uuid: &Uuid) -> String {
    format!("freshblu.configure.{}", uuid)
}

/// NATS subject for system-wide unregister events.
pub fn system_unregister() -> String {
    "freshblu.system.unregister".to_string()
}

/// NATS subject for delivering envelopes to a specific gateway pod.
pub fn delivery(pod_id: &str) -> String {
    format!("freshblu.delivery.{}", pod_id)
}

/// NATS subject for presence updates for a device.
pub fn presence(uuid: &Uuid) -> String {
    format!("freshblu.presence.{}", uuid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subject_formats() {
        let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        assert_eq!(
            device_inbox(&uuid),
            "freshblu.device.550e8400-e29b-41d4-a716-446655440000.inbox"
        );
        assert_eq!(
            broadcast(&uuid),
            "freshblu.broadcast.550e8400-e29b-41d4-a716-446655440000"
        );
        assert_eq!(
            configure(&uuid),
            "freshblu.configure.550e8400-e29b-41d4-a716-446655440000"
        );
        assert_eq!(system_unregister(), "freshblu.system.unregister");
        assert_eq!(delivery("pod-1"), "freshblu.delivery.pod-1");
        assert_eq!(
            presence(&uuid),
            "freshblu.presence.550e8400-e29b-41d4-a716-446655440000"
        );
    }
}
