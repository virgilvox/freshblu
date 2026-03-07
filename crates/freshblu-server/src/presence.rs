use redis::AsyncCommands;
use tracing::warn;
use uuid::Uuid;

const PRESENCE_TTL: u64 = 30; // seconds
const HEARTBEAT_INTERVAL: u64 = 15; // seconds

/// Redis-based presence tracker for device-to-pod mapping.
/// Each connected device has a key `freshblu:presence:{uuid}` = `{pod_id}` with a TTL.
/// If the pod crashes, the TTL expires and the device is considered offline.
#[derive(Clone)]
pub struct PresenceTracker {
    redis: redis::aio::ConnectionManager,
    pod_id: String,
}

impl PresenceTracker {
    pub fn new(redis: redis::aio::ConnectionManager, pod_id: String) -> Self {
        Self { redis, pod_id }
    }

    fn key(uuid: &Uuid) -> String {
        format!("freshblu:presence:{}", uuid)
    }

    /// Register a device as present on this pod. Sets key with TTL.
    pub async fn register(&self, uuid: &Uuid) -> anyhow::Result<()> {
        let mut conn = self.redis.clone();
        let key = Self::key(uuid);
        conn.set_ex::<_, _, ()>(&key, &self.pod_id, PRESENCE_TTL).await?;
        Ok(())
    }

    /// Refresh the TTL for a device's presence key.
    pub async fn heartbeat(&self, uuid: &Uuid) -> anyhow::Result<()> {
        let mut conn = self.redis.clone();
        let key = Self::key(uuid);
        conn.expire::<_, ()>(&key, PRESENCE_TTL as i64).await?;
        Ok(())
    }

    /// Remove a device's presence key.
    pub async fn unregister(&self, uuid: &Uuid) -> anyhow::Result<()> {
        let mut conn = self.redis.clone();
        let key = Self::key(uuid);
        conn.del::<_, ()>(&key).await?;
        Ok(())
    }

    /// Look up which pod a device is on. Returns None if offline.
    pub async fn get_pod(&self, uuid: &Uuid) -> anyhow::Result<Option<String>> {
        let mut conn = self.redis.clone();
        let key = Self::key(uuid);
        let result: Option<String> = conn.get(&key).await?;
        Ok(result)
    }

    /// Spawn a background heartbeat task that refreshes the presence key every 15s.
    /// Returns a handle that can be aborted on disconnect.
    pub fn spawn_heartbeat(&self, uuid: Uuid) -> tokio::task::JoinHandle<()> {
        let tracker = self.clone();
        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(std::time::Duration::from_secs(HEARTBEAT_INTERVAL));
            loop {
                interval.tick().await;
                if let Err(e) = tracker.heartbeat(&uuid).await {
                    warn!("Presence heartbeat failed for {}: {}", uuid, e);
                    break;
                }
            }
        })
    }
}
