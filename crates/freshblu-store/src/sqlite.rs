use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use async_trait::async_trait;
use freshblu_core::{
    auth::{compute_device_hash, generate_token, hash_token, verify_token},
    device::{Device, DeviceView, MeshbluMeta, RegisterParams, WhitelistEntry},
    error::{FreshBluError, Result},
    permissions::Whitelists,
    subscription::{CreateSubscriptionParams, Subscription, SubscriptionType},
    token::{GenerateTokenOptions, TokenRecord},
};
use serde_json::Value;
use sqlx::{sqlite::SqlitePool, Row};
use uuid::Uuid;

use crate::store::DeviceStore;

pub struct SqliteStore {
    pool: SqlitePool,
}

impl SqliteStore {
    pub async fn new(database_url: &str) -> anyhow::Result<Self> {
        let pool = SqlitePool::connect(database_url).await?;
        let store = Self { pool };
        store.migrate().await?;
        Ok(store)
    }

    async fn migrate(&self) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS devices (
                uuid TEXT PRIMARY KEY,
                data TEXT NOT NULL,
                online INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT
            );

            CREATE TABLE IF NOT EXISTS tokens (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                device_uuid TEXT NOT NULL,
                hash TEXT NOT NULL,
                created_at TEXT NOT NULL,
                expires_on INTEGER,
                tag TEXT,
                FOREIGN KEY(device_uuid) REFERENCES devices(uuid) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_tokens_device ON tokens(device_uuid);

            CREATE TABLE IF NOT EXISTS subscriptions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                emitter_uuid TEXT NOT NULL,
                subscriber_uuid TEXT NOT NULL,
                subscription_type TEXT NOT NULL,
                UNIQUE(emitter_uuid, subscriber_uuid, subscription_type)
            );

            CREATE INDEX IF NOT EXISTS idx_subs_emitter ON subscriptions(emitter_uuid, subscription_type);
            CREATE INDEX IF NOT EXISTS idx_subs_subscriber ON subscriptions(subscriber_uuid);
            "#,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    fn deserialize_device(data: &str) -> Result<Device> {
        serde_json::from_str(data)
            .map_err(|e| FreshBluError::Storage(format!("deserialize error: {}", e)))
    }

    fn serialize_device(device: &Device) -> Result<String> {
        serde_json::to_string(device)
            .map_err(|e| FreshBluError::Storage(format!("serialize error: {}", e)))
    }
}

#[async_trait]
impl DeviceStore for SqliteStore {
    async fn register(&self, params: RegisterParams) -> Result<(Device, String)> {
        let uuid = Uuid::new_v4();
        let plaintext_token = generate_token();
        let token_hash = hash_token(&plaintext_token)?;

        let whitelists = params
            .meshblu
            .and_then(|m| m.whitelists)
            .unwrap_or_else(Whitelists::open);

        let mut device = Device::new(params.properties, whitelists);
        device.uuid = uuid;
        if let Some(t) = &params.device_type {
            device.device_type = Some(t.clone());
        }

        let device_json = Self::serialize_device(&device)?;
        let hash = compute_device_hash(&device_json);
        device.meshblu.hash = hash;

        let final_json = Self::serialize_device(&device)?;
        let created_at = device.meshblu.created_at.to_rfc3339();
        let uuid_str = uuid.to_string();

        sqlx::query(
            "INSERT INTO devices (uuid, data, online, created_at) VALUES (?, ?, 0, ?)",
        )
        .bind(&uuid_str)
        .bind(&final_json)
        .bind(&created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| FreshBluError::Storage(e.to_string()))?;

        // Store the primary token
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO tokens (device_uuid, hash, created_at) VALUES (?, ?, ?)",
        )
        .bind(&uuid_str)
        .bind(&token_hash)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| FreshBluError::Storage(e.to_string()))?;

        Ok((device, plaintext_token))
    }

    async fn get_device(&self, uuid: &Uuid) -> Result<Option<Device>> {
        let row = sqlx::query("SELECT data FROM devices WHERE uuid = ?")
            .bind(uuid.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| FreshBluError::Storage(e.to_string()))?;

        match row {
            Some(r) => {
                let data: String = r.get("data");
                let device = Self::deserialize_device(&data)?;
                Ok(Some(device))
            }
            None => Ok(None),
        }
    }

    async fn update_device(&self, uuid: &Uuid, properties: HashMap<String, Value>) -> Result<Device> {
        let mut device = self
            .get_device(uuid)
            .await?
            .ok_or(FreshBluError::NotFound)?;

        // Merge properties (don't allow overwriting meshblu system fields)
        for (k, v) in properties {
            match k.as_str() {
                "uuid" | "token" | "meshblu" => {}
                _ => {
                    device.properties.insert(k, v);
                }
            }
        }

        device.meshblu.updated_at = Some(chrono::Utc::now());
        let device_json = Self::serialize_device(&device)?;
        let hash = compute_device_hash(&device_json);
        device.meshblu.hash = hash;

        let final_json = Self::serialize_device(&device)?;
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query("UPDATE devices SET data = ?, updated_at = ? WHERE uuid = ?")
            .bind(&final_json)
            .bind(&now)
            .bind(uuid.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| FreshBluError::Storage(e.to_string()))?;

        Ok(device)
    }

    async fn set_online(&self, uuid: &Uuid, online: bool) -> Result<()> {
        sqlx::query("UPDATE devices SET online = ? WHERE uuid = ?")
            .bind(online as i64)
            .bind(uuid.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| FreshBluError::Storage(e.to_string()))?;
        Ok(())
    }

    async fn unregister(&self, uuid: &Uuid) -> Result<()> {
        sqlx::query("DELETE FROM devices WHERE uuid = ?")
            .bind(uuid.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| FreshBluError::Storage(e.to_string()))?;
        Ok(())
    }

    async fn search_devices(&self, filters: &HashMap<String, Value>) -> Result<Vec<DeviceView>> {
        // For SQLite JSON: use json_extract to filter
        // Simple implementation: fetch all and filter in memory
        // Production: build proper SQL JSON queries
        let rows = sqlx::query("SELECT data, online FROM devices")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| FreshBluError::Storage(e.to_string()))?;

        let mut results = Vec::new();
        for row in rows {
            let data: String = row.get("data");
            let online: i64 = row.get("online");
            if let Ok(mut device) = Self::deserialize_device(&data) {
                device.online = online != 0;
                let view = device.to_view();

                // Check filters
                let matches = filters.iter().all(|(k, v)| {
                    match k.as_str() {
                        "online" => {
                            let want_online = v == "true";
                            view.online == want_online
                        }
                        "type" => view
                            .device_type
                            .as_ref()
                            .map(|t| Value::String(t.clone()) == *v)
                            .unwrap_or(false),
                        _ => view
                            .properties
                            .get(k)
                            .map(|pv| pv == v)
                            .unwrap_or(false),
                    }
                });

                if matches {
                    results.push(view);
                }
            }
        }

        Ok(results)
    }

    async fn authenticate(&self, uuid: &Uuid, token: &str) -> Result<Option<Device>> {
        // Get all non-expired tokens for this device
        let now_ts = chrono::Utc::now().timestamp();
        let rows = sqlx::query(
            "SELECT hash FROM tokens WHERE device_uuid = ? AND (expires_on IS NULL OR expires_on > ?)",
        )
        .bind(uuid.to_string())
        .bind(now_ts)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| FreshBluError::Storage(e.to_string()))?;

        for row in &rows {
            let hash: String = row.get("hash");
            if verify_token(token, &hash) {
                return self.get_device(uuid).await;
            }
        }
        Ok(None)
    }

    async fn generate_token(&self, uuid: &Uuid, opts: GenerateTokenOptions) -> Result<(TokenRecord, String)> {
        let plaintext = generate_token();
        let hash = hash_token(&plaintext)?;
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT INTO tokens (device_uuid, hash, created_at, expires_on, tag) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(uuid.to_string())
        .bind(&hash)
        .bind(&now)
        .bind(opts.expires_on)
        .bind(&opts.tag)
        .execute(&self.pool)
        .await
        .map_err(|e| FreshBluError::Storage(e.to_string()))?;

        let record = TokenRecord {
            uuid: *uuid,
            hash,
            created_at: chrono::Utc::now(),
            expires_on: opts.expires_on,
            tag: opts.tag,
        };

        Ok((record, plaintext))
    }

    async fn revoke_token(&self, uuid: &Uuid, token: &str) -> Result<()> {
        // Find the matching hash and delete it
        let rows = sqlx::query("SELECT id, hash FROM tokens WHERE device_uuid = ?")
            .bind(uuid.to_string())
            .fetch_all(&self.pool)
            .await
            .map_err(|e| FreshBluError::Storage(e.to_string()))?;

        for row in rows {
            let id: i64 = row.get("id");
            let hash: String = row.get("hash");
            if verify_token(token, &hash) {
                sqlx::query("DELETE FROM tokens WHERE id = ?")
                    .bind(id)
                    .execute(&self.pool)
                    .await
                    .map_err(|e| FreshBluError::Storage(e.to_string()))?;
                return Ok(());
            }
        }

        Err(FreshBluError::NotFound)
    }

    async fn revoke_tokens_by_query(&self, uuid: &Uuid, query: HashMap<String, Value>) -> Result<()> {
        if let Some(tag) = query.get("tag").and_then(|v| v.as_str()) {
            sqlx::query("DELETE FROM tokens WHERE device_uuid = ? AND tag = ?")
                .bind(uuid.to_string())
                .bind(tag)
                .execute(&self.pool)
                .await
                .map_err(|e| FreshBluError::Storage(e.to_string()))?;
        } else if let Some(exp) = query.get("expiresOn").and_then(|v| v.as_i64()) {
            sqlx::query("DELETE FROM tokens WHERE device_uuid = ? AND expires_on = ?")
                .bind(uuid.to_string())
                .bind(exp)
                .execute(&self.pool)
                .await
                .map_err(|e| FreshBluError::Storage(e.to_string()))?;
        }
        Ok(())
    }

    async fn list_tokens(&self, uuid: &Uuid) -> Result<Vec<TokenRecord>> {
        let rows = sqlx::query(
            "SELECT hash, created_at, expires_on, tag FROM tokens WHERE device_uuid = ?",
        )
        .bind(uuid.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| FreshBluError::Storage(e.to_string()))?;

        let mut records = Vec::new();
        for row in rows {
            let hash: String = row.get("hash");
            let created_at_str: String = row.get("created_at");
            let expires_on: Option<i64> = row.get("expires_on");
            let tag: Option<String> = row.get("tag");
            if let Ok(created_at) =
                chrono::DateTime::parse_from_rfc3339(&created_at_str)
            {
                records.push(TokenRecord {
                    uuid: *uuid,
                    hash,
                    created_at: created_at.with_timezone(&chrono::Utc),
                    expires_on,
                    tag,
                });
            }
        }

        Ok(records)
    }

    async fn create_subscription(&self, params: &CreateSubscriptionParams) -> Result<Subscription> {
        sqlx::query(
            "INSERT OR IGNORE INTO subscriptions (emitter_uuid, subscriber_uuid, subscription_type) VALUES (?, ?, ?)",
        )
        .bind(params.emitter_uuid.to_string())
        .bind(params.subscriber_uuid.to_string())
        .bind(params.subscription_type.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| FreshBluError::Storage(e.to_string()))?;

        Ok(Subscription {
            emitter_uuid: params.emitter_uuid,
            subscriber_uuid: params.subscriber_uuid,
            subscription_type: params.subscription_type.clone(),
        })
    }

    async fn delete_subscription(
        &self,
        subscriber_uuid: &Uuid,
        emitter_uuid: Option<&Uuid>,
        sub_type: Option<&SubscriptionType>,
    ) -> Result<()> {
        match (emitter_uuid, sub_type) {
            (Some(e), Some(t)) => {
                sqlx::query(
                    "DELETE FROM subscriptions WHERE subscriber_uuid = ? AND emitter_uuid = ? AND subscription_type = ?",
                )
                .bind(subscriber_uuid.to_string())
                .bind(e.to_string())
                .bind(t.to_string())
                .execute(&self.pool)
                .await
                .map_err(|e| FreshBluError::Storage(e.to_string()))?;
            }
            (Some(e), None) => {
                sqlx::query(
                    "DELETE FROM subscriptions WHERE subscriber_uuid = ? AND emitter_uuid = ?",
                )
                .bind(subscriber_uuid.to_string())
                .bind(e.to_string())
                .execute(&self.pool)
                .await
                .map_err(|e| FreshBluError::Storage(e.to_string()))?;
            }
            _ => {
                sqlx::query("DELETE FROM subscriptions WHERE subscriber_uuid = ?")
                    .bind(subscriber_uuid.to_string())
                    .execute(&self.pool)
                    .await
                    .map_err(|e| FreshBluError::Storage(e.to_string()))?;
            }
        }
        Ok(())
    }

    async fn get_subscriptions(&self, subscriber_uuid: &Uuid) -> Result<Vec<Subscription>> {
        let rows = sqlx::query(
            "SELECT emitter_uuid, subscription_type FROM subscriptions WHERE subscriber_uuid = ?",
        )
        .bind(subscriber_uuid.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| FreshBluError::Storage(e.to_string()))?;

        let mut subs = Vec::new();
        for row in rows {
            let emitter_str: String = row.get("emitter_uuid");
            let type_str: String = row.get("subscription_type");
            if let (Ok(emitter), Ok(sub_type)) = (
                Uuid::parse_str(&emitter_str),
                SubscriptionType::from_str(&type_str),
            ) {
                subs.push(Subscription {
                    emitter_uuid: emitter,
                    subscriber_uuid: *subscriber_uuid,
                    subscription_type: sub_type,
                });
            }
        }

        Ok(subs)
    }

    async fn get_subscribers(
        &self,
        emitter_uuid: &Uuid,
        sub_type: &SubscriptionType,
    ) -> Result<Vec<Uuid>> {
        let rows = sqlx::query(
            "SELECT subscriber_uuid FROM subscriptions WHERE emitter_uuid = ? AND subscription_type = ?",
        )
        .bind(emitter_uuid.to_string())
        .bind(sub_type.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| FreshBluError::Storage(e.to_string()))?;

        let mut uuids = Vec::new();
        for row in rows {
            let s: String = row.get("subscriber_uuid");
            if let Ok(u) = Uuid::parse_str(&s) {
                uuids.push(u);
            }
        }

        Ok(uuids)
    }
}
