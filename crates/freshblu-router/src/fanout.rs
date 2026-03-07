use freshblu_core::subscription::SubscriptionType;
use freshblu_proto::{DeliveryEnvelope, NatsEvent};
use freshblu_store::DynStore;
use redis::AsyncCommands;
use tracing::{debug, warn};
use uuid::Uuid;

/// Resolves subscriptions and routes delivery envelopes to the correct gateway pod.
pub struct Fanout {
    pub store: DynStore,
    pub redis: redis::aio::ConnectionManager,
    pub nats: async_nats::Client,
}

impl Fanout {
    /// Route a direct message to the target device's pod.
    pub async fn route_direct(&self, target: &Uuid, event: NatsEvent, source_pod: &str) {
        if let Some(pod_id) = self.get_pod(target).await {
            self.send_envelope(target, &event, &pod_id, source_pod)
                .await;
        } else {
            debug!("Device {} not online, dropping direct message", target);
        }
    }

    /// Route a broadcast: look up subscribers, find their pods, and deliver.
    pub async fn route_broadcast(&self, emitter: &Uuid, event: NatsEvent, source_pod: &str) {
        let sub_type = SubscriptionType::BroadcastSent;
        self.fanout_to_subscribers(emitter, &sub_type, &event, source_pod)
            .await;
    }

    /// Route a config update to configure.sent subscribers.
    pub async fn route_config(&self, uuid: &Uuid, event: NatsEvent, source_pod: &str) {
        // Deliver to the device itself
        self.route_direct(uuid, event.clone(), source_pod).await;

        // Fan out to configure.sent subscribers
        let sub_type = SubscriptionType::ConfigureSent;
        self.fanout_to_subscribers(uuid, &sub_type, &event, source_pod)
            .await;
    }

    /// Route an unregister event to unregister.sent subscribers.
    pub async fn route_unregister(&self, uuid: &Uuid, event: NatsEvent, source_pod: &str) {
        let sub_type = SubscriptionType::UnregisterSent;
        self.fanout_to_subscribers(uuid, &sub_type, &event, source_pod)
            .await;
    }

    async fn fanout_to_subscribers(
        &self,
        emitter: &Uuid,
        sub_type: &SubscriptionType,
        event: &NatsEvent,
        source_pod: &str,
    ) {
        let subscribers = match self.store.get_subscribers(emitter, sub_type).await {
            Ok(subs) => subs,
            Err(e) => {
                warn!("Failed to get subscribers for {}: {}", emitter, e);
                return;
            }
        };

        for sub_uuid in &subscribers {
            if let Some(pod_id) = self.get_pod(sub_uuid).await {
                self.send_envelope(sub_uuid, event, &pod_id, source_pod)
                    .await;
            }
        }
    }

    async fn get_pod(&self, uuid: &Uuid) -> Option<String> {
        let mut conn = self.redis.clone();
        let key = format!("freshblu:presence:{}", uuid);
        conn.get(&key).await.ok()
    }

    async fn send_envelope(
        &self,
        target: &Uuid,
        event: &NatsEvent,
        pod_id: &str,
        source_pod: &str,
    ) {
        let envelope = DeliveryEnvelope {
            target: *target,
            event: event.clone(),
            source_pod: source_pod.to_string(),
        };
        let subject = freshblu_proto::delivery(pod_id);
        match serde_json::to_vec(&envelope) {
            Ok(payload) => {
                if let Err(e) = self.nats.publish(subject, payload.into()).await {
                    warn!("Failed to publish delivery envelope to {}: {}", pod_id, e);
                }
            }
            Err(e) => {
                warn!("Failed to serialize delivery envelope: {}", e);
            }
        }
    }
}
