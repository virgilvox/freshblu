use axum::{
    extract::{Path, Query, State},
    Json,
};
use freshblu_core::{
    device::{DeviceView, RegisterParams, UpdateParams},
    error::FreshBluError,
    message::DeviceEvent,
    permissions::PermissionChecker,
    subscription::SubscriptionType,
};
use serde::Deserialize;
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use uuid::Uuid;

use super::AuthenticatedDevice;
use crate::AppState;

type ApiResult<T> = Result<Json<T>, FreshBluError>;

// POST /devices
pub async fn register(
    State(state): State<AppState>,
    Json(params): Json<RegisterParams>,
) -> ApiResult<Value> {
    let (device, plaintext_token) = state.store.register(params).await?;

    // Return device + plaintext token (only time token is visible)
    let mut resp = serde_json::to_value(&device).map_err(|e| {
        FreshBluError::Internal(e.to_string())
    })?;
    resp["token"] = Value::String(plaintext_token);

    Ok(Json(resp))
}

// GET /devices/:uuid
pub async fn get_device(
    State(state): State<AppState>,
    AuthenticatedDevice(actor, as_uuid): AuthenticatedDevice,
    Path(uuid): Path<Uuid>,
) -> ApiResult<DeviceView> {
    let device = state
        .store
        .get_device(&uuid)
        .await?
        .ok_or(FreshBluError::NotFound)?;

    let effective_actor = as_uuid.unwrap_or(actor.uuid);

    // Check permission: can actor discover/view this device?
    let checker = PermissionChecker::new(
        &device.meshblu.whitelists,
        &effective_actor,
        &uuid,
    );

    if !checker.can_discover_view() {
        // Meshblu convention: can't distinguish "not found" from "no permission"
        return Err(FreshBluError::NotFound);
    }

    Ok(Json(device.to_view()))
}

// PUT /devices/:uuid
pub async fn update_device(
    State(state): State<AppState>,
    AuthenticatedDevice(actor, as_uuid): AuthenticatedDevice,
    Path(uuid): Path<Uuid>,
    Json(body): Json<HashMap<String, Value>>,
) -> ApiResult<DeviceView> {
    let device = state
        .store
        .get_device(&uuid)
        .await?
        .ok_or(FreshBluError::NotFound)?;

    let effective_actor = as_uuid.unwrap_or(actor.uuid);

    let checker = PermissionChecker::new(
        &device.meshblu.whitelists,
        &effective_actor,
        &uuid,
    );

    if !checker.can_configure_update() {
        return Err(FreshBluError::Forbidden);
    }

    let updated = state.store.update_device(&uuid, body).await?;
    let view = updated.to_view();

    // Emit configure event to all configure.sent subscribers
    let config_event = DeviceEvent::Config { device: view.clone() };
    let subscribers = state
        .store
        .get_subscribers(&uuid, &SubscriptionType::ConfigureSent)
        .await
        .unwrap_or_default();

    for sub_uuid in subscribers {
        state.hub.deliver(&sub_uuid, config_event.clone());
    }

    // Also deliver to device itself if connected
    state.hub.deliver(&uuid, config_event);

    Ok(Json(view))
}

// DELETE /devices/:uuid
pub async fn unregister(
    State(state): State<AppState>,
    AuthenticatedDevice(actor, as_uuid): AuthenticatedDevice,
    Path(uuid): Path<Uuid>,
) -> ApiResult<Value> {
    let device = state
        .store
        .get_device(&uuid)
        .await?
        .ok_or(FreshBluError::NotFound)?;

    let effective_actor = as_uuid.unwrap_or(actor.uuid);
    let checker = PermissionChecker::new(
        &device.meshblu.whitelists,
        &effective_actor,
        &uuid,
    );

    if !checker.can_configure_update() {
        return Err(FreshBluError::Forbidden);
    }

    // Notify subscribers
    let unreg_event = DeviceEvent::Unregistered { uuid };
    let subs = state
        .store
        .get_subscribers(&uuid, &SubscriptionType::UnregisterSent)
        .await
        .unwrap_or_default();
    for sub_uuid in subs {
        state.hub.deliver(&sub_uuid, unreg_event.clone());
    }

    state.store.unregister(&uuid).await?;
    state.hub.disconnect(&uuid);

    Ok(Json(json!({ "uuid": uuid })))
}

// POST /devices/search
pub async fn search(
    State(state): State<AppState>,
    AuthenticatedDevice(actor, as_uuid): AuthenticatedDevice,
    Json(query): Json<HashMap<String, Value>>,
) -> ApiResult<Vec<DeviceView>> {
    let effective_actor = as_uuid.unwrap_or(actor.uuid);
    let all = state.store.search_devices(&query).await?;

    // Filter to only devices this actor can discover
    let visible: Vec<DeviceView> = all
        .into_iter()
        .filter(|d| {
            let checker = PermissionChecker::new(
                &d.meshblu.whitelists,
                &effective_actor,
                &d.uuid,
            );
            checker.can_discover_view()
        })
        .collect();

    Ok(Json(visible))
}

// GET /whoami
pub async fn whoami(
    State(state): State<AppState>,
    AuthenticatedDevice(actor, _): AuthenticatedDevice,
) -> ApiResult<DeviceView> {
    let device = state
        .store
        .get_device(&actor.uuid)
        .await?
        .ok_or(FreshBluError::NotFound)?;
    Ok(Json(device.to_view()))
}

// GET /mydevices
pub async fn my_devices(
    State(state): State<AppState>,
    AuthenticatedDevice(actor, _): AuthenticatedDevice,
) -> ApiResult<Vec<DeviceView>> {
    // Return all devices owned by this actor (where they're the creator)
    // Simplified: just return the device itself
    let device = state
        .store
        .get_device(&actor.uuid)
        .await?
        .ok_or(FreshBluError::NotFound)?;
    Ok(Json(vec![device.to_view()]))
}

pub mod auth {
    use super::*;

    // POST /authenticate - verify credentials
    pub async fn authenticate(
        State(state): State<AppState>,
        Json(body): Json<HashMap<String, Value>>,
    ) -> ApiResult<Value> {
        let uuid_str = body
            .get("uuid")
            .and_then(|v| v.as_str())
            .ok_or(FreshBluError::Validation("uuid required".into()))?;
        let token = body
            .get("token")
            .and_then(|v| v.as_str())
            .ok_or(FreshBluError::Validation("token required".into()))?;

        let uuid = Uuid::parse_str(uuid_str)
            .map_err(|_| FreshBluError::Validation("invalid uuid".into()))?;

        let device = state
            .store
            .authenticate(&uuid, token)
            .await?
            .ok_or(FreshBluError::Unauthorized)?;

        Ok(Json(json!({ "uuid": device.uuid })))
    }
}
