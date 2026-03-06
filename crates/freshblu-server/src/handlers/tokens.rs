use axum::{
    extract::{Path, State},
    Json,
};
use freshblu_core::{
    error::FreshBluError,
    permissions::PermissionChecker,
    token::GenerateTokenOptions,
};
use uuid::Uuid;

use super::AuthenticatedDevice;
use crate::AppState;

type ApiResult<T> = Result<Json<T>, FreshBluError>;

// POST /devices/:uuid/tokens
pub async fn generate_token(
    State(state): State<AppState>,
    AuthenticatedDevice(actor, _): AuthenticatedDevice,
    Path(uuid): Path<Uuid>,
    Json(opts): Json<Option<GenerateTokenOptions>>,
) -> ApiResult<serde_json::Value> {
    let device = state
        .store
        .get_device(&uuid)
        .await?
        .ok_or(FreshBluError::NotFound)?;

    let checker = PermissionChecker::new(
        &device.meshblu.whitelists,
        &actor.uuid,
        &uuid,
    );

    if !checker.can_configure_update() {
        return Err(FreshBluError::Forbidden);
    }

    let (record, plaintext) = state
        .store
        .generate_token(&uuid, opts.unwrap_or_default())
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
        .ok_or(FreshBluError::NotFound)?;

    let checker = PermissionChecker::new(
        &device.meshblu.whitelists,
        &actor.uuid,
        &uuid,
    );

    if !checker.can_configure_update() {
        return Err(FreshBluError::Forbidden);
    }

    state.store.revoke_token(&uuid, &token).await?;

    Ok(Json(serde_json::json!({ "revoked": true })))
}
