use axum::{
    extract::State,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
};
use std::convert::Infallible;
use std::sync::Arc;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;
use uuid::Uuid;

use super::AuthenticatedDevice;
use crate::AppState;

/// Guard that cleans up bus/store state when dropped.
struct SubscribeGuard {
    uuid: Uuid,
    state: AppState,
}

impl Drop for SubscribeGuard {
    fn drop(&mut self) {
        let uuid = self.uuid;
        let state = self.state.clone();
        tokio::spawn(async move {
            let _ = state.store.set_online(&uuid, false).await;
            state.bus.disconnect(&uuid);
        });
    }
}

/// GET /subscribe — SSE stream of all events for the authenticated device
pub async fn subscribe(
    State(state): State<AppState>,
    AuthenticatedDevice(actor, _): AuthenticatedDevice,
) -> impl IntoResponse {
    let _ = state.store.set_online(&actor.uuid, true).await;
    let rx = state.bus.connect(actor.uuid);

    // Arc guard so the closure can own a clone
    let guard = Arc::new(SubscribeGuard {
        uuid: actor.uuid,
        state: state.clone(),
    });

    let stream = BroadcastStream::new(rx).filter_map(move |result| {
        let _guard = guard.clone(); // prevent drop until stream ends
        match result {
            Ok(event) => match serde_json::to_string(&event) {
                Ok(json) => Some(Ok::<_, Infallible>(Event::default().data(json))),
                Err(_) => None,
            },
            Err(_) => None,
        }
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}
