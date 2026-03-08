use std::collections::HashMap;

use async_trait::async_trait;
use freshblu_core::{
    device::{Device, DeviceView, RegisterParams},
    error::Result,
    subscription::{CreateSubscriptionParams, Subscription, SubscriptionType},
    token::{GenerateTokenOptions, TokenRecord},
};
use redis::AsyncCommands;
use serde_json::Value;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::store::{DeviceStore, DynStore};

const AUTH_TTL: u64 = 300; // 5 minutes
const DEVICE_TTL: u64 = 60; // 60 seconds
const SUBS_TTL: u64 = 60; // 60 seconds

/// A caching decorator that wraps any `DynStore` with Redis caching.
/// Caches auth results, device lookups, and subscriber lists.
pub struct CachedStore {
    inner: DynStore,
    redis: redis::aio::ConnectionManager,
}

impl CachedStore {
    pub async fn new(inner: DynStore, redis_url: &str) -> anyhow::Result<Self> {
        let client = redis::Client::open(redis_url)?;
        let redis = redis::aio::ConnectionManager::new(client).await?;
        Ok(Self { inner, redis })
    }

    fn auth_key(uuid: &Uuid, token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        let hash = hex::encode(hasher.finalize());
        format!("freshblu:auth:{}:{}", uuid, hash)
    }

    fn device_key(uuid: &Uuid) -> String {
        format!("freshblu:device:{}", uuid)
    }

    fn subs_key(emitter: &Uuid, sub_type: &SubscriptionType) -> String {
        format!("freshblu:subs:{}:{}", emitter, sub_type)
    }

    fn auth_pattern(uuid: &Uuid) -> String {
        format!("freshblu:auth:{}:*", uuid)
    }

    async fn invalidate_device(&self, uuid: &Uuid) {
        let mut conn = self.redis.clone();
        let key = Self::device_key(uuid);
        let _: std::result::Result<(), _> = conn.del::<_, ()>(&key).await;
    }

    async fn invalidate_auth(&self, uuid: &Uuid) {
        let pattern = Self::auth_pattern(uuid);
        self.scan_and_delete(&pattern).await;
    }

    async fn invalidate_subs(&self, uuid: &Uuid) {
        let pattern = format!("freshblu:subs:{}:*", uuid);
        self.scan_and_delete(&pattern).await;
    }

    /// Use SCAN instead of KEYS to avoid blocking Redis on large keyspaces.
    async fn scan_and_delete(&self, pattern: &str) {
        let mut conn = self.redis.clone();
        let mut cursor: u64 = 0;
        loop {
            let result: std::result::Result<(u64, Vec<String>), _> = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(pattern)
                .arg("COUNT")
                .arg(100)
                .query_async(&mut conn)
                .await;
            match result {
                Ok((next_cursor, keys)) => {
                    for key in keys {
                        let _: std::result::Result<(), _> = conn.del::<_, ()>(&key).await;
                    }
                    if next_cursor == 0 {
                        break;
                    }
                    cursor = next_cursor;
                }
                Err(_) => break,
            }
        }
    }
}

#[async_trait]
impl DeviceStore for CachedStore {
    async fn register(&self, params: RegisterParams) -> Result<(Device, String)> {
        self.inner.register(params).await
    }

    async fn get_device(&self, uuid: &Uuid) -> Result<Option<Device>> {
        let mut conn = self.redis.clone();
        let key = Self::device_key(uuid);

        // Check cache
        let cached: std::result::Result<Option<String>, _> = conn.get(&key).await;
        if let Ok(Some(json)) = cached {
            if let Ok(device) = serde_json::from_str::<Device>(&json) {
                return Ok(Some(device));
            }
        }

        // Cache miss — fetch from inner store
        let result = self.inner.get_device(uuid).await?;
        if let Some(ref device) = result {
            if let Ok(json) = serde_json::to_string(device) {
                let _: std::result::Result<(), _> =
                    conn.set_ex::<_, _, ()>(&key, &json, DEVICE_TTL).await;
            }
        }
        Ok(result)
    }

    async fn update_device(
        &self,
        uuid: &Uuid,
        properties: HashMap<String, Value>,
    ) -> Result<Device> {
        let result = self.inner.update_device(uuid, properties).await?;
        self.invalidate_device(uuid).await;
        Ok(result)
    }

    async fn set_online(&self, uuid: &Uuid, online: bool) -> Result<()> {
        let result = self.inner.set_online(uuid, online).await?;
        self.invalidate_device(uuid).await;
        Ok(result)
    }

    async fn unregister(&self, uuid: &Uuid) -> Result<()> {
        let result = self.inner.unregister(uuid).await?;
        self.invalidate_device(uuid).await;
        self.invalidate_auth(uuid).await;
        self.invalidate_subs(uuid).await;
        Ok(result)
    }

    async fn search_devices(&self, filters: &HashMap<String, Value>) -> Result<Vec<DeviceView>> {
        // Search is not cached — too many permutations
        self.inner.search_devices(filters).await
    }

    async fn find_by_owner(&self, owner: &Uuid) -> Result<Vec<DeviceView>> {
        self.inner.find_by_owner(owner).await
    }

    async fn authenticate(&self, uuid: &Uuid, token: &str) -> Result<Option<Device>> {
        let mut conn = self.redis.clone();
        let key = Self::auth_key(uuid, token);

        // Check cache
        let cached: std::result::Result<Option<String>, _> = conn.get(&key).await;
        if let Ok(Some(val)) = cached {
            if val == "1" {
                return self.get_device(uuid).await;
            }
        }

        // Cache miss — authenticate against inner store
        let result = self.inner.authenticate(uuid, token).await?;
        if result.is_some() {
            let _: std::result::Result<(), _> = conn.set_ex::<_, _, ()>(&key, "1", AUTH_TTL).await;
        }
        Ok(result)
    }

    async fn generate_token(
        &self,
        uuid: &Uuid,
        opts: GenerateTokenOptions,
    ) -> Result<(TokenRecord, String)> {
        let result = self.inner.generate_token(uuid, opts).await?;
        // Don't invalidate auth cache — new tokens don't affect existing cached results
        Ok(result)
    }

    async fn revoke_token(&self, uuid: &Uuid, token: &str) -> Result<()> {
        let result = self.inner.revoke_token(uuid, token).await?;
        self.invalidate_auth(uuid).await;
        Ok(result)
    }

    async fn revoke_tokens_by_query(
        &self,
        uuid: &Uuid,
        query: HashMap<String, Value>,
    ) -> Result<()> {
        let result = self.inner.revoke_tokens_by_query(uuid, query).await?;
        self.invalidate_auth(uuid).await;
        Ok(result)
    }

    async fn list_tokens(&self, uuid: &Uuid) -> Result<Vec<TokenRecord>> {
        self.inner.list_tokens(uuid).await
    }

    async fn claim_device(&self, uuid: &Uuid, owner: &Uuid) -> Result<Device> {
        let result = self.inner.claim_device(uuid, owner).await?;
        self.invalidate_device(uuid).await;
        Ok(result)
    }

    async fn reset_token(&self, uuid: &Uuid) -> Result<String> {
        let result = self.inner.reset_token(uuid).await?;
        self.invalidate_auth(uuid).await;
        Ok(result)
    }

    async fn search_tokens(&self, query: &HashMap<String, Value>) -> Result<Vec<TokenRecord>> {
        self.inner.search_tokens(query).await
    }

    async fn create_subscription(&self, params: &CreateSubscriptionParams) -> Result<Subscription> {
        let result = self.inner.create_subscription(params).await?;
        self.invalidate_subs(&params.emitter_uuid).await;
        Ok(result)
    }

    async fn delete_subscription(
        &self,
        subscriber_uuid: &Uuid,
        emitter_uuid: Option<&Uuid>,
        sub_type: Option<&SubscriptionType>,
    ) -> Result<()> {
        let result = self
            .inner
            .delete_subscription(subscriber_uuid, emitter_uuid, sub_type)
            .await?;
        if let Some(emitter) = emitter_uuid {
            self.invalidate_subs(emitter).await;
        }
        Ok(result)
    }

    async fn get_subscriptions(&self, subscriber_uuid: &Uuid) -> Result<Vec<Subscription>> {
        self.inner.get_subscriptions(subscriber_uuid).await
    }

    async fn get_subscribers(
        &self,
        emitter_uuid: &Uuid,
        sub_type: &SubscriptionType,
    ) -> Result<Vec<Uuid>> {
        let mut conn = self.redis.clone();
        let key = Self::subs_key(emitter_uuid, sub_type);

        // Check cache
        let cached: std::result::Result<Option<String>, _> = conn.get(&key).await;
        if let Ok(Some(json)) = cached {
            if let Ok(uuids) = serde_json::from_str::<Vec<Uuid>>(&json) {
                return Ok(uuids);
            }
        }

        // Cache miss
        let result = self.inner.get_subscribers(emitter_uuid, sub_type).await?;
        if let Ok(json) = serde_json::to_string(&result) {
            let _: std::result::Result<(), _> =
                conn.set_ex::<_, _, ()>(&key, &json, SUBS_TTL).await;
        }
        Ok(result)
    }
}

// hex encoding helper (avoid pulling in hex crate)
mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes
            .as_ref()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect()
    }
}
