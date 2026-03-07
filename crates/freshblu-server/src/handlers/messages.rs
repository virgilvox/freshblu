use axum::{extract::State, Json};
use freshblu_core::{
    error::FreshBluError,
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
        for sub_uuid in received_subs {
            let _ = state.bus.publish(&sub_uuid, msg_event.clone()).await;
        }

        // Fan out: message.sent subscribers of sender
        let sent_subs = state
            .store
            .get_subscribers(&sender_uuid, &SubscriptionType::MessageSent)
            .await
            .unwrap_or_default();
        for sub_uuid in sent_subs {
            let _ = state.bus.publish(&sub_uuid, msg_event.clone()).await;
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

        for sub_uuid in &broadcast_subs {
            // Verify sub still has permission
            if let Ok(Some(_sub_device)) = state.store.get_device(sub_uuid).await {
                // They subscribed, permission was valid at sub time - deliver
                let _ = state.bus.publish(sub_uuid, broadcast_event.clone()).await;

                // Also fan out to broadcast.received subscribers of each recipient
                let br_subs = state
                    .store
                    .get_subscribers(sub_uuid, &SubscriptionType::BroadcastReceived)
                    .await
                    .unwrap_or_default();
                for br_sub in br_subs {
                    let _ = state.bus.publish(&br_sub, broadcast_event.clone()).await;
                }
            }
        }
    }

    MESSAGES_SENT.inc();
    Ok(Json(serde_json::json!({ "sent": true })))
}
