use axum::{extract::State, Json};
use freshblu_core::{
    error::FreshBluError,
    message::{DeviceEvent, Message, MessageMetadata, SendMessageParams},
    permissions::PermissionChecker,
    subscription::SubscriptionType,
};
use uuid::Uuid;

use super::AuthenticatedDevice;
use crate::AppState;

type ApiResult<T> = Result<Json<T>, FreshBluError>;

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
            .ok_or(FreshBluError::NotFound)?;
        let checker = PermissionChecker::new(
            &target.meshblu.whitelists,
            &actor.uuid,
            as_u,
        );
        if !checker.can_message_as() {
            return Err(FreshBluError::Forbidden);
        }
    }

    let is_broadcast = params.is_broadcast();

    let message = Message {
        devices: params.devices.clone(),
        from_uuid: Some(sender_uuid),
        topic: params.topic.clone(),
        payload: params.payload.clone(),
        metadata: None,
        extra: params.extra.clone(),
    };

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
        let msg_event = DeviceEvent::Message(message.clone());
        state.hub.deliver(&target_uuid, msg_event.clone());

        // Fan out: message.received subscribers of target
        let received_subs = state
            .store
            .get_subscribers(&target_uuid, &SubscriptionType::MessageReceived)
            .await
            .unwrap_or_default();
        for sub_uuid in received_subs {
            state.hub.deliver(&sub_uuid, msg_event.clone());
        }

        // Fan out: message.sent subscribers of sender
        let sent_subs = state
            .store
            .get_subscribers(&sender_uuid, &SubscriptionType::MessageSent)
            .await
            .unwrap_or_default();
        for sub_uuid in sent_subs {
            state.hub.deliver(&sub_uuid, msg_event.clone());
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

        let broadcast_event = DeviceEvent::Broadcast(message.clone());

        for sub_uuid in &broadcast_subs {
            // Verify sub still has permission
            if let Ok(Some(sub_device)) = state.store.get_device(sub_uuid).await {
                let checker = PermissionChecker::new(
                    &sub_device.meshblu.whitelists,
                    &sender_uuid,
                    sub_uuid,
                );
                // They subscribed, permission was valid at sub time - deliver
                state.hub.deliver(sub_uuid, broadcast_event.clone());

                // Also fan out to broadcast.received subscribers of each recipient
                let br_subs = state
                    .store
                    .get_subscribers(sub_uuid, &SubscriptionType::BroadcastReceived)
                    .await
                    .unwrap_or_default();
                for br_sub in br_subs {
                    state.hub.deliver(&br_sub, broadcast_event.clone());
                }
            }
        }
    }

    Ok(Json(serde_json::json!({ "sent": true })))
}
