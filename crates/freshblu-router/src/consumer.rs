use freshblu_proto::NatsEvent;
use futures::StreamExt;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::fanout::Fanout;

/// Consumes events from NATS subjects and routes them via Fanout.
pub async fn run_consumer(fanout: Fanout) -> anyhow::Result<()> {
    let nats = fanout.nats.clone();

    // Subscribe to all freshblu event subjects
    let mut device_sub = nats.subscribe("freshblu.device.>").await?;
    let mut broadcast_sub = nats.subscribe("freshblu.broadcast.>").await?;
    let mut configure_sub = nats.subscribe("freshblu.configure.>").await?;
    let mut system_sub = nats.subscribe("freshblu.system.>").await?;

    info!("Router consumer started, listening on freshblu.* subjects");

    loop {
        tokio::select! {
            Some(msg) = device_sub.next() => {
                // Subject: freshblu.device.{uuid}.inbox
                if let Some(uuid) = extract_uuid_from_subject(&msg.subject, "freshblu.device.", ".inbox") {
                    match serde_json::from_slice::<NatsEvent>(&msg.payload) {
                        Ok(event) => {
                            fanout.route_direct(&uuid, event, "router").await;
                        }
                        Err(e) => warn!("Failed to parse device event: {}", e),
                    }
                }
            }
            Some(msg) = broadcast_sub.next() => {
                // Subject: freshblu.broadcast.{uuid}
                if let Some(uuid) = extract_uuid_from_subject(&msg.subject, "freshblu.broadcast.", "") {
                    match serde_json::from_slice::<NatsEvent>(&msg.payload) {
                        Ok(event) => {
                            fanout.route_broadcast(&uuid, event, "router").await;
                        }
                        Err(e) => warn!("Failed to parse broadcast event: {}", e),
                    }
                }
            }
            Some(msg) = configure_sub.next() => {
                // Subject: freshblu.configure.{uuid}
                if let Some(uuid) = extract_uuid_from_subject(&msg.subject, "freshblu.configure.", "") {
                    match serde_json::from_slice::<NatsEvent>(&msg.payload) {
                        Ok(event) => {
                            fanout.route_config(&uuid, event, "router").await;
                        }
                        Err(e) => warn!("Failed to parse configure event: {}", e),
                    }
                }
            }
            Some(msg) = system_sub.next() => {
                // Subject: freshblu.system.unregister
                match serde_json::from_slice::<NatsEvent>(&msg.payload) {
                    Ok(event) => {
                        if let NatsEvent::Unregister { uuid } = &event {
                            let uuid = *uuid;
                            fanout.route_unregister(&uuid, event, "router").await;
                        }
                    }
                    Err(e) => warn!("Failed to parse system event: {}", e),
                }
            }
            else => {
                error!("All NATS subscriptions closed");
                break;
            }
        }
    }

    Ok(())
}

fn extract_uuid_from_subject(subject: &str, prefix: &str, suffix: &str) -> Option<Uuid> {
    let rest = subject.strip_prefix(prefix)?;
    let uuid_str = if suffix.is_empty() {
        rest
    } else {
        rest.strip_suffix(suffix)?
    };
    Uuid::parse_str(uuid_str).ok()
}

