pub mod auth;
pub mod devices;
pub mod messages;
pub mod status;
pub mod subscriptions;
pub mod tokens;

use axum::{
    extract::FromRequestParts,
    http::request::Parts,
    response::{IntoResponse, Response},
};
use freshblu_core::{auth::parse_basic_auth, device::Device, error::FreshBluError};
use uuid::Uuid;

use crate::{ApiError, AppState};

/// Extractor: authenticated device from HTTP Basic Auth header (uuid:token)
pub struct AuthenticatedDevice(pub Device, pub Option<Uuid>);

#[axum::async_trait]
impl FromRequestParts<AppState> for AuthenticatedDevice {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let headers = &parts.headers;

        // Try Authorization header first
        let creds = headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(parse_basic_auth);

        // Fallback: skynet_auth header (legacy)
        let creds = creds.or_else(|| {
            headers
                .get("skynet_auth_uuid")
                .zip(headers.get("skynet_auth_token"))
                .and_then(|(u, t)| {
                    Some((
                        u.to_str().ok()?.to_string(),
                        t.to_str().ok()?.to_string(),
                    ))
                })
        });

        let (uuid_str, token) = creds.ok_or_else(|| {
            ApiError::from(FreshBluError::Unauthorized).into_response()
        })?;

        let uuid = Uuid::parse_str(&uuid_str).map_err(|_| {
            ApiError::from(FreshBluError::Unauthorized).into_response()
        })?;

        let device = state
            .store
            .authenticate(&uuid, &token)
            .await
            .map_err(|e| ApiError::from(e).into_response())?
            .ok_or_else(|| ApiError::from(FreshBluError::Unauthorized).into_response())?;

        // Check x-meshblu-as header for acting as another device
        let as_uuid = headers
            .get("x-meshblu-as")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| Uuid::parse_str(v).ok());

        Ok(AuthenticatedDevice(device, as_uuid))
    }
}
