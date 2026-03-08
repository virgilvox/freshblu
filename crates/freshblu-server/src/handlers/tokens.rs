use axum::{
    extract::{Path, State},
    Json,
};
use freshblu_core::{
    error::FreshBluError, permissions::PermissionChecker, token::GenerateTokenOptions,
};
use std::collections::HashMap;
use uuid::Uuid;

use super::AuthenticatedDevice;
use crate::{ApiError, AppState};

type ApiResult<T> = Result<Json<T>, ApiError>;

// POST /devices/:uuid/tokens
pub async fn generate_token(
    State(state): State<AppState>,
    AuthenticatedDevice(actor, _): AuthenticatedDevice,
    Path(uuid): Path<Uuid>,
    opts: Option<Json<GenerateTokenOptions>>,
) -> ApiResult<serde_json::Value> {
    let device = state
        .store
        .get_device(&uuid)
        .await?
        .ok_or(FreshBluError::NotFound)
        .map_err(ApiError::from)?;

    let checker = PermissionChecker::new(&device.meshblu.whitelists, &actor.uuid, &uuid);

    if !checker.can_configure_update() {
        return Err(FreshBluError::Forbidden.into());
    }

    let (record, plaintext) = state
        .store
        .generate_token(&uuid, opts.map(|j| j.0).unwrap_or_default())
        .await?;

    Ok(Json(serde_json::json!({
        "uuid": uuid,
        "token": plaintext,
        "createdAt": record.created_at,
    })))
}

// DELETE /devices/:uuid/tokens/:token
pub async fn revoke_token(
    State(state): State<AppState>,
    AuthenticatedDevice(actor, _): AuthenticatedDevice,
    Path((uuid, token)): Path<(Uuid, String)>,
) -> ApiResult<serde_json::Value> {
    let device = state
        .store
        .get_device(&uuid)
        .await?
        .ok_or(FreshBluError::NotFound)
        .map_err(ApiError::from)?;

    let checker = PermissionChecker::new(&device.meshblu.whitelists, &actor.uuid, &uuid);

    if !checker.can_configure_update() {
        return Err(FreshBluError::Forbidden.into());
    }

    state.store.revoke_token(&uuid, &token).await?;

    Ok(Json(serde_json::json!({ "revoked": true })))
}

// POST /devices/:uuid/token — revoke all tokens and generate new root token
pub async fn reset_token(
    State(state): State<AppState>,
    AuthenticatedDevice(actor, _): AuthenticatedDevice,
    Path(uuid): Path<Uuid>,
) -> ApiResult<serde_json::Value> {
    let device = state
        .store
        .get_device(&uuid)
        .await?
        .ok_or(FreshBluError::NotFound)
        .map_err(ApiError::from)?;

    let checker = PermissionChecker::new(&device.meshblu.whitelists, &actor.uuid, &uuid);
    if !checker.can_configure_update() {
        return Err(FreshBluError::Forbidden.into());
    }

    let new_token = state.store.reset_token(&uuid).await?;

    Ok(Json(serde_json::json!({
        "uuid": uuid,
        "token": new_token,
    })))
}

// POST /search/tokens — search tokens by tag/device/expiry
// Scoped: only returns tokens belonging to the authenticated device
pub async fn search_tokens(
    State(state): State<AppState>,
    AuthenticatedDevice(actor, _): AuthenticatedDevice,
    Json(mut query): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Vec<serde_json::Value>> {
    // Force the query to be scoped to the authenticated device's UUID
    query.insert(
        "uuid".to_string(),
        serde_json::Value::String(actor.uuid.to_string()),
    );
    let records = state.store.search_tokens(&query).await?;

    let results: Vec<serde_json::Value> = records
        .into_iter()
        .map(|r| {
            serde_json::json!({
                "uuid": r.uuid,
                "createdAt": r.created_at,
                "expiresOn": r.expires_on,
                "tag": r.tag,
            })
        })
        .collect();

    Ok(Json(results))
}
