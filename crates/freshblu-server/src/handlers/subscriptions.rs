use axum::{
    extract::{Path, State},
    Json,
};
use freshblu_core::{
    error::FreshBluError,
    permissions::PermissionChecker,
    subscription::{CreateSubscriptionParams, DeleteSubscriptionParams, Subscription, SubscriptionType},
};
use std::str::FromStr;
use uuid::Uuid;

use super::AuthenticatedDevice;
use crate::AppState;

type ApiResult<T> = Result<Json<T>, FreshBluError>;

// POST /devices/:uuid/subscriptions
pub async fn create_subscription(
    State(state): State<AppState>,
    AuthenticatedDevice(actor, _): AuthenticatedDevice,
    Path(subscriber_uuid): Path<Uuid>,
    Json(params): Json<CreateSubscriptionParams>,
) -> ApiResult<Subscription> {
    // Verify actor can create subscriptions for subscriber_uuid
    if actor.uuid != subscriber_uuid {
        // Must have configure.update permission to create subs for another device
        let sub_device = state
            .store
            .get_device(&subscriber_uuid)
            .await?
            .ok_or(FreshBluError::NotFound)?;
        let checker = PermissionChecker::new(
            &sub_device.meshblu.whitelists,
            &actor.uuid,
            &subscriber_uuid,
        );
        if !checker.can_configure_update() {
            return Err(FreshBluError::Forbidden);
        }
    }

    // Verify permission to subscribe to emitter's events
    let emitter_device = state
        .store
        .get_device(&params.emitter_uuid)
        .await?
        .ok_or(FreshBluError::NotFound)?;

    let checker = PermissionChecker::new(
        &emitter_device.meshblu.whitelists,
        &subscriber_uuid,
        &params.emitter_uuid,
    );

    let allowed = match &params.subscription_type {
        SubscriptionType::BroadcastSent => checker.can_broadcast_sent(),
        SubscriptionType::BroadcastReceived => checker.can_broadcast_received(),
        SubscriptionType::MessageSent => checker.can_message_sent(),
        SubscriptionType::MessageReceived => checker.can_message_received(),
        SubscriptionType::ConfigureSent => checker.can_configure_sent(),
        SubscriptionType::ConfigureReceived => checker.can_configure_received(),
        SubscriptionType::UnregisterSent | SubscriptionType::UnregisterReceived => {
            // Anyone can subscribe to unregister events if they can discover
            checker.can_discover_view()
        }
    };

    if !allowed {
        return Err(FreshBluError::Forbidden);
    }

    let sub = state.store.create_subscription(&params).await?;
    Ok(Json(sub))
}

// GET /devices/:uuid/subscriptions
pub async fn list_subscriptions(
    State(state): State<AppState>,
    AuthenticatedDevice(actor, _): AuthenticatedDevice,
    Path(subscriber_uuid): Path<Uuid>,
) -> ApiResult<Vec<Subscription>> {
    // Can only list your own subscriptions (or with configure.update permission)
    if actor.uuid != subscriber_uuid {
        let device = state
            .store
            .get_device(&subscriber_uuid)
            .await?
            .ok_or(FreshBluError::NotFound)?;
        let checker = PermissionChecker::new(
            &device.meshblu.whitelists,
            &actor.uuid,
            &subscriber_uuid,
        );
        if !checker.can_configure_update() {
            return Err(FreshBluError::Forbidden);
        }
    }

    let subs = state.store.get_subscriptions(&subscriber_uuid).await?;
    Ok(Json(subs))
}

// DELETE /devices/:uuid/subscriptions/:emitter_uuid/:sub_type
pub async fn delete_subscription(
    State(state): State<AppState>,
    AuthenticatedDevice(actor, _): AuthenticatedDevice,
    Path((subscriber_uuid, emitter_uuid, sub_type_str)): Path<(Uuid, Uuid, String)>,
) -> ApiResult<serde_json::Value> {
    if actor.uuid != subscriber_uuid {
        let device = state
            .store
            .get_device(&subscriber_uuid)
            .await?
            .ok_or(FreshBluError::NotFound)?;
        let checker = PermissionChecker::new(
            &device.meshblu.whitelists,
            &actor.uuid,
            &subscriber_uuid,
        );
        if !checker.can_configure_update() {
            return Err(FreshBluError::Forbidden);
        }
    }

    let sub_type = SubscriptionType::from_str(&sub_type_str.replace("-", "."))
        .map_err(|_| FreshBluError::Validation("invalid subscription type".into()))?;

    state
        .store
        .delete_subscription(&subscriber_uuid, Some(&emitter_uuid), Some(&sub_type))
        .await?;

    Ok(Json(serde_json::json!({ "deleted": true })))
}
