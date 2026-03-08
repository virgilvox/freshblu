use axum::{extract::State, Json};
use freshblu_core::{
    error::FreshBluError,
    forwarder::ForwarderEvent,
    message::{DeviceEvent, Message, SendMessageParams},
    permissions::PermissionChecker,
    subscription::SubscriptionType,
};
use std::sync::Arc;
use uuid::Uuid;

use super::AuthenticatedDevice;
use crate::metrics::MESSAGES_SENT;
use crate::{ApiError, AppState};

type ApiResult<T> = Result<Json<T>, ApiError>;

// POST /messages
pub async fn send_message(
    State(state): State<AppState>,
    AuthenticatedDevice(actor, as_uuid): AuthenticatedDevice,
    Json(params): Json<SendMessageParams>,
) -> ApiResult<serde_json::Value> {
    // Message size validation — checks payload + extra fields
    {
        let payload_size = params
            .payload
            .as_ref()
            .and_then(|p| serde_json::to_string(p).ok())
            .map(|s| s.len())
            .unwrap_or(0);
        let extra_size = if params.extra.is_empty() {
            0
        } else {
            serde_json::to_string(&params.extra)
                .map(|s| s.len())
                .unwrap_or(0)
        };
        if payload_size + extra_size > state.config.max_message_size {
            return Err(FreshBluError::MessageTooLarge.into());
        }
    }

    let sender_uuid = as_uuid.unwrap_or(actor.uuid);

    // If acting as another device, verify permission
    if let Some(ref as_u) = as_uuid {
        let target = state
            .store
            .get_device(as_u)
            .await?
            .ok_or(FreshBluError::NotFound)
            .map_err(ApiError::from)?;
        let checker = PermissionChecker::new(&target.meshblu.whitelists, &actor.uuid, as_u);
        if !checker.can_message_as() {
            return Err(FreshBluError::Forbidden.into());
        }
        // If broadcasting as another device, also check broadcast.as
        if params.devices.iter().any(|d| d == "*") && !checker.can_broadcast_as() {
            return Err(FreshBluError::Forbidden.into());
        }
    }

    let is_broadcast = params.is_broadcast();

    let message = Arc::new(Message {
        devices: params.devices.clone(),
        from_uuid: Some(sender_uuid),
        topic: params.topic.clone(),
        payload: params.payload.clone(),
        metadata: None,
        extra: params.extra.clone(),
    });

    // --- Direct messages to specific device UUIDs ---
    for device_id in &params.devices {
        if device_id == "*" {
            continue; // handled as broadcast below
        }

        let target_uuid = match Uuid::parse_str(device_id) {
            Ok(u) => u,
            Err(_) => continue,
        };

        let target_device = match state.store.get_device(&target_uuid).await {
            Ok(Some(d)) => d,
            _ => continue,
        };

        // Check: can sender send to target?
        let checker = PermissionChecker::new(
            &target_device.meshblu.whitelists,
            &sender_uuid,
            &target_uuid,
        );

        if !checker.can_message_from() {
            continue; // silently skip unauthorized targets
        }

        // Deliver to target device
        let msg_event = DeviceEvent::Message((*message).clone());
        let _ = state.bus.publish(&target_uuid, msg_event.clone()).await;

        // Fan out: message.received subscribers of target
        let received_subs = state
            .store
            .get_subscribers(&target_uuid, &SubscriptionType::MessageReceived)
            .await
            .unwrap_or_default();
        if !received_subs.is_empty() {
            let _ = state
                .bus
                .publish_many(&received_subs, msg_event.clone())
                .await;
        }

        // Fan out: message.sent subscribers of sender
        let sent_subs = state
            .store
            .get_subscribers(&sender_uuid, &SubscriptionType::MessageSent)
            .await
            .unwrap_or_default();
        if !sent_subs.is_empty() {
            let _ = state.bus.publish_many(&sent_subs, msg_event.clone()).await;
        }
    }

    // --- Broadcast to all subscribers ---
    if is_broadcast {
        // Get all devices subscribed to this sender's broadcast.sent
        let broadcast_subs = state
            .store
            .get_subscribers(&sender_uuid, &SubscriptionType::BroadcastSent)
            .await
            .unwrap_or_default();

        let broadcast_event = DeviceEvent::Broadcast((*message).clone());

        // Deliver to all broadcast.sent subscribers
        // Subscriptions have FK constraints with CASCADE delete, so stale entries
        // are cleaned up automatically when devices are unregistered.
        if !broadcast_subs.is_empty() {
            let _ = state
                .bus
                .publish_many(&broadcast_subs, broadcast_event.clone())
                .await;
        }

        // Fan out to broadcast.received subscribers of each recipient
        for sub_uuid in &broadcast_subs {
            let br_subs = state
                .store
                .get_subscribers(sub_uuid, &SubscriptionType::BroadcastReceived)
                .await
                .unwrap_or_default();
            if !br_subs.is_empty() {
                let _ = state
                    .bus
                    .publish_many(&br_subs, broadcast_event.clone())
                    .await;
            }
        }
    }

    // Fire forwarders for sender (message.sent)
    if let Ok(Some(sender_device)) = state.store.get_device(&sender_uuid).await {
        let payload = serde_json::to_value(&*message).unwrap_or_default();
        let executor = state.webhook_executor.clone();
        let dev = sender_device.clone();
        tokio::spawn(async move {
            executor
                .execute(&dev, ForwarderEvent::MessageSent, &payload, &[])
                .await;
        });
    }

    MESSAGES_SENT.inc();
    Ok(Json(serde_json::json!({ "sent": true })))
}

/// Broadcast-specific params where `devices` is optional (defaults to ["*"])
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BroadcastParams {
    #[serde(default)]
    pub devices: Option<Vec<String>>,
    pub topic: Option<String>,
    pub payload: Option<serde_json::Value>,
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

// POST /broadcasts — broadcast wrapper (forces devices: ["*"])
pub async fn broadcast(
    State(state): State<AppState>,
    AuthenticatedDevice(actor, as_uuid): AuthenticatedDevice,
    Json(params): Json<BroadcastParams>,
) -> ApiResult<serde_json::Value> {
    let msg_params = SendMessageParams {
        devices: vec!["*".to_string()],
        topic: params.topic,
        payload: params.payload,
        extra: params.extra,
    };
    send_message(
        State(state),
        AuthenticatedDevice(actor, as_uuid),
        Json(msg_params),
    )
    .await
}
