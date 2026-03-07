use axum::{
    extract::{Path, State},
    Json,
};
use freshblu_core::{
    device::{DeviceView, RegisterParams},
    error::FreshBluError,
    message::DeviceEvent,
    permissions::PermissionChecker,
    subscription::SubscriptionType,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use uuid::Uuid;

use super::AuthenticatedDevice;
use crate::{ApiError, AppState};

type ApiResult<T> = Result<Json<T>, ApiError>;

// POST /devices
pub async fn register(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(params): Json<RegisterParams>,
) -> ApiResult<Value> {
    // Enforce open_registration flag
    if !state.config.open_registration {
        // If registration is closed, require valid auth credentials
        let has_auth = headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(freshblu_core::auth::parse_basic_auth)
            .is_some();
        let has_legacy_auth =
            headers.get("skynet_auth_uuid").is_some() && headers.get("skynet_auth_token").is_some();
        if !has_auth && !has_legacy_auth {
            return Err(FreshBluError::Forbidden.into());
        }
    }

    let (device, plaintext_token) = state.store.register(params).await?;

    // Return device + plaintext token (only time token is visible)
    let mut resp = serde_json::to_value(&device)
        .map_err(|e| ApiError::from(FreshBluError::Internal(e.to_string())))?;
    resp["token"] = Value::String(plaintext_token);

    Ok(Json(resp))
}

// GET /devices/:uuid
pub async fn get_device(
    State(state): State<AppState>,
    AuthenticatedDevice(actor, as_uuid): AuthenticatedDevice,
    Path(uuid): Path<Uuid>,
) -> ApiResult<DeviceView> {
    // Verify x-meshblu-as permission
    if let Some(ref as_u) = as_uuid {
        let as_device = state
            .store
            .get_device(as_u)
            .await?
            .ok_or(FreshBluError::NotFound)
            .map_err(ApiError::from)?;
        let checker = PermissionChecker::new(&as_device.meshblu.whitelists, &actor.uuid, as_u);
        if !checker.can_discover_as() {
            return Err(FreshBluError::Forbidden.into());
        }
    }

    let device = state
        .store
        .get_device(&uuid)
        .await?
        .ok_or(FreshBluError::NotFound)
        .map_err(ApiError::from)?;

    let effective_actor = as_uuid.unwrap_or(actor.uuid);

    // Check permission: can actor discover/view this device?
    let checker = PermissionChecker::new(&device.meshblu.whitelists, &effective_actor, &uuid);

    if !checker.can_discover_view() {
        return Err(FreshBluError::NotFound.into());
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
    // Verify x-meshblu-as permission
    if let Some(ref as_u) = as_uuid {
        let as_device = state
            .store
            .get_device(as_u)
            .await?
            .ok_or(FreshBluError::NotFound)
            .map_err(ApiError::from)?;
        let checker = PermissionChecker::new(&as_device.meshblu.whitelists, &actor.uuid, as_u);
        if !checker.can_configure_as() {
            return Err(FreshBluError::Forbidden.into());
        }
    }

    let device = state
        .store
        .get_device(&uuid)
        .await?
        .ok_or(FreshBluError::NotFound)
        .map_err(ApiError::from)?;

    let effective_actor = as_uuid.unwrap_or(actor.uuid);

    let checker = PermissionChecker::new(&device.meshblu.whitelists, &effective_actor, &uuid);

    if !checker.can_configure_update() {
        return Err(FreshBluError::Forbidden.into());
    }

    let updated = state.store.update_device(&uuid, body).await?;
    let view = updated.to_view();

    // Emit configure event to all configure.sent subscribers
    let config_event = DeviceEvent::Config {
        device: Box::new(view.clone()),
    };
    let subscribers = state
        .store
        .get_subscribers(&uuid, &SubscriptionType::ConfigureSent)
        .await
        .unwrap_or_default();

    for sub_uuid in subscribers {
        let _ = state.bus.publish(&sub_uuid, config_event.clone()).await;
    }

    // Also deliver to device itself if connected
    let _ = state.bus.publish(&uuid, config_event).await;

    Ok(Json(view))
}

// DELETE /devices/:uuid
pub async fn unregister(
    State(state): State<AppState>,
    AuthenticatedDevice(actor, as_uuid): AuthenticatedDevice,
    Path(uuid): Path<Uuid>,
) -> ApiResult<Value> {
    // Verify x-meshblu-as permission
    if let Some(ref as_u) = as_uuid {
        let as_device = state
            .store
            .get_device(as_u)
            .await?
            .ok_or(FreshBluError::NotFound)
            .map_err(ApiError::from)?;
        let checker = PermissionChecker::new(&as_device.meshblu.whitelists, &actor.uuid, as_u);
        if !checker.can_configure_as() {
            return Err(FreshBluError::Forbidden.into());
        }
    }

    let device = state
        .store
        .get_device(&uuid)
        .await?
        .ok_or(FreshBluError::NotFound)
        .map_err(ApiError::from)?;

    let effective_actor = as_uuid.unwrap_or(actor.uuid);
    let checker = PermissionChecker::new(&device.meshblu.whitelists, &effective_actor, &uuid);

    if !checker.can_configure_update() {
        return Err(FreshBluError::Forbidden.into());
    }

    // Notify subscribers
    let unreg_event = DeviceEvent::Unregistered { uuid };
    let subs = state
        .store
        .get_subscribers(&uuid, &SubscriptionType::UnregisterSent)
        .await
        .unwrap_or_default();
    for sub_uuid in subs {
        let _ = state.bus.publish(&sub_uuid, unreg_event.clone()).await;
    }

    state.store.unregister(&uuid).await?;
    state.bus.disconnect(&uuid);

    Ok(Json(json!({ "uuid": uuid })))
}

// POST /devices/search
pub async fn search(
    State(state): State<AppState>,
    AuthenticatedDevice(actor, as_uuid): AuthenticatedDevice,
    Json(query): Json<HashMap<String, Value>>,
) -> ApiResult<Vec<DeviceView>> {
    // Verify x-meshblu-as permission
    if let Some(ref as_u) = as_uuid {
        let as_device = state
            .store
            .get_device(as_u)
            .await?
            .ok_or(FreshBluError::NotFound)
            .map_err(ApiError::from)?;
        let checker = PermissionChecker::new(&as_device.meshblu.whitelists, &actor.uuid, as_u);
        if !checker.can_discover_as() {
            return Err(FreshBluError::Forbidden.into());
        }
    }

    let effective_actor = as_uuid.unwrap_or(actor.uuid);
    let all = state.store.search_devices(&query).await?;

    // Filter to only devices this actor can discover
    let visible: Vec<DeviceView> = all
        .into_iter()
        .filter(|d| {
            let checker = PermissionChecker::new(&d.meshblu.whitelists, &effective_actor, &d.uuid);
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
        .ok_or(FreshBluError::NotFound)
        .map_err(ApiError::from)?;
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
        .ok_or(FreshBluError::NotFound)
        .map_err(ApiError::from)?;
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
            .ok_or_else(|| ApiError::from(FreshBluError::Validation("uuid required".into())))?;
        let token = body
            .get("token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ApiError::from(FreshBluError::Validation("token required".into())))?;

        let uuid = Uuid::parse_str(uuid_str)
            .map_err(|_| ApiError::from(FreshBluError::Validation("invalid uuid".into())))?;

        let device = state
            .store
            .authenticate(&uuid, token)
            .await?
            .ok_or_else(|| ApiError::from(FreshBluError::Unauthorized))?;

        Ok(Json(json!({ "uuid": device.uuid })))
    }
}
