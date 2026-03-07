use std::collections::HashMap;
use std::str::FromStr;

use async_trait::async_trait;
use freshblu_core::{
    auth::{compute_device_hash, generate_token, hash_token, verify_token},
    device::{Device, DeviceView, RegisterParams},
    error::{FreshBluError, Result},
    permissions::Whitelists,
    subscription::{CreateSubscriptionParams, Subscription, SubscriptionType},
    token::{GenerateTokenOptions, TokenRecord},
};
use serde_json::Value;
use sqlx::{postgres::PgPool, Row};
use uuid::Uuid;

use crate::store::DeviceStore;

pub struct PostgresStore {
    pool: PgPool,
}

impl PostgresStore {
    pub async fn new(database_url: &str) -> anyhow::Result<Self> {
        let pool = PgPool::connect(database_url).await?;
        let store = Self { pool };
        store.migrate().await?;
        Ok(store)
    }

    async fn migrate(&self) -> anyhow::Result<()> {
        let sql = include_str!("../migrations/001_initial.sql");
        sqlx::raw_sql(sql).execute(&self.pool).await?;
        Ok(())
    }

    fn deserialize_device(data: &Value) -> Result<Device> {
        serde_json::from_value(data.clone())
            .map_err(|e| FreshBluError::Storage(format!("deserialize error: {}", e)))
    }

    fn serialize_device(device: &Device) -> Result<Value> {
        serde_json::to_value(device)
            .map_err(|e| FreshBluError::Storage(format!("serialize error: {}", e)))
    }
}

#[async_trait]
impl DeviceStore for PostgresStore {
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

        let device_json =
            serde_json::to_string(&device).map_err(|e| FreshBluError::Storage(e.to_string()))?;
        let hash = compute_device_hash(&device_json);
        device.meshblu.hash = hash;

        let data = Self::serialize_device(&device)?;

        sqlx::query("INSERT INTO devices (uuid, data, online) VALUES ($1, $2, false)")
            .bind(uuid)
            .bind(&data)
            .execute(&self.pool)
            .await
            .map_err(|e| FreshBluError::Storage(e.to_string()))?;

        sqlx::query("INSERT INTO tokens (device_uuid, hash) VALUES ($1, $2)")
            .bind(uuid)
            .bind(&token_hash)
            .execute(&self.pool)
            .await
            .map_err(|e| FreshBluError::Storage(e.to_string()))?;

        Ok((device, plaintext_token))
    }

    async fn get_device(&self, uuid: &Uuid) -> Result<Option<Device>> {
        let row = sqlx::query("SELECT data, online FROM devices WHERE uuid = $1")
            .bind(uuid)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| FreshBluError::Storage(e.to_string()))?;

        match row {
            Some(r) => {
                let data: Value = r.get("data");
                let online: bool = r.get("online");
                let mut device = Self::deserialize_device(&data)?;
                device.online = online;
                Ok(Some(device))
            }
            None => Ok(None),
        }
    }

    async fn update_device(
        &self,
        uuid: &Uuid,
        properties: HashMap<String, Value>,
    ) -> Result<Device> {
        let mut device = self
            .get_device(uuid)
            .await?
            .ok_or(FreshBluError::NotFound)?;

        for (k, v) in properties {
            match k.as_str() {
                "uuid" | "token" | "meshblu" => {}
                _ => {
                    device.properties.insert(k, v);
                }
            }
        }

        device.meshblu.updated_at = Some(chrono::Utc::now());
        let device_json =
            serde_json::to_string(&device).map_err(|e| FreshBluError::Storage(e.to_string()))?;
        let hash = compute_device_hash(&device_json);
        device.meshblu.hash = hash;

        let data = Self::serialize_device(&device)?;

        sqlx::query("UPDATE devices SET data = $1, updated_at = NOW() WHERE uuid = $2")
            .bind(&data)
            .bind(uuid)
            .execute(&self.pool)
            .await
            .map_err(|e| FreshBluError::Storage(e.to_string()))?;

        Ok(device)
    }

    async fn set_online(&self, uuid: &Uuid, online: bool) -> Result<()> {
        sqlx::query("UPDATE devices SET online = $1 WHERE uuid = $2")
            .bind(online)
            .bind(uuid)
            .execute(&self.pool)
            .await
            .map_err(|e| FreshBluError::Storage(e.to_string()))?;
        Ok(())
    }

    async fn unregister(&self, uuid: &Uuid) -> Result<()> {
        sqlx::query("DELETE FROM devices WHERE uuid = $1")
            .bind(uuid)
            .execute(&self.pool)
            .await
            .map_err(|e| FreshBluError::Storage(e.to_string()))?;
        Ok(())
    }

    async fn search_devices(&self, filters: &HashMap<String, Value>) -> Result<Vec<DeviceView>> {
        let mut conditions = vec!["1=1".to_string()];
        let mut bind_values: Vec<String> = Vec::new();

        if let Some(type_val) = filters.get("type").and_then(|v| v.as_str()) {
            bind_values.push(type_val.to_string());
            conditions.push(format!("data->>'type' = ${}", bind_values.len()));
        }

        if let Some(online_val) = filters.get("online") {
            let want = online_val == "true" || online_val == &Value::Bool(true);
            bind_values.push(if want {
                "true".to_string()
            } else {
                "false".to_string()
            });
            conditions.push(format!("online = ${}", bind_values.len()));
        }

        // For arbitrary JSON filters, use JSONB containment
        let mut remaining_filters: HashMap<&String, &Value> = HashMap::new();
        for (k, v) in filters {
            if k != "type" && k != "online" {
                remaining_filters.insert(k, v);
            }
        }

        let sql = format!(
            "SELECT data, online FROM devices WHERE {} LIMIT 100",
            conditions.join(" AND ")
        );

        let mut query = sqlx::query(&sql);
        for val in &bind_values {
            query = query.bind(val);
        }

        let rows = query
            .fetch_all(&self.pool)
            .await
            .map_err(|e| FreshBluError::Storage(e.to_string()))?;

        let mut results = Vec::new();
        for row in rows {
            let data: Value = row.get("data");
            let online: bool = row.get("online");
            if let Ok(mut device) = Self::deserialize_device(&data) {
                device.online = online;
                let view = device.to_view();

                let matches = remaining_filters.iter().all(|(k, v)| {
                    view.properties
                        .get(k.as_str())
                        .map(|pv| pv == *v)
                        .unwrap_or(false)
                });

                if matches {
                    results.push(view);
                }
            }
        }

        Ok(results)
    }

    async fn authenticate(&self, uuid: &Uuid, token: &str) -> Result<Option<Device>> {
        let now_ts = chrono::Utc::now().timestamp();
        let rows = sqlx::query(
            "SELECT hash FROM tokens WHERE device_uuid = $1 AND (expires_on IS NULL OR expires_on > $2)",
        )
        .bind(uuid)
        .bind(now_ts)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| FreshBluError::Storage(e.to_string()))?;

        let token = token.to_string();
        for row in &rows {
            let hash: String = row.get("hash");
            if verify_token(&token, &hash) {
                return self.get_device(uuid).await;
            }
        }
        Ok(None)
    }

    async fn generate_token(
        &self,
        uuid: &Uuid,
        opts: GenerateTokenOptions,
    ) -> Result<(TokenRecord, String)> {
        let plaintext = generate_token();
        let hash = hash_token(&plaintext)?;

        sqlx::query(
            "INSERT INTO tokens (device_uuid, hash, expires_on, tag) VALUES ($1, $2, $3, $4)",
        )
        .bind(uuid)
        .bind(&hash)
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
        let rows = sqlx::query("SELECT id, hash FROM tokens WHERE device_uuid = $1")
            .bind(uuid)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| FreshBluError::Storage(e.to_string()))?;

        for row in rows {
            let id: i64 = row.get("id");
            let hash: String = row.get("hash");
            if verify_token(token, &hash) {
                sqlx::query("DELETE FROM tokens WHERE id = $1")
                    .bind(id)
                    .execute(&self.pool)
                    .await
                    .map_err(|e| FreshBluError::Storage(e.to_string()))?;
                return Ok(());
            }
        }

        Err(FreshBluError::NotFound)
    }

    async fn revoke_tokens_by_query(
        &self,
        uuid: &Uuid,
        query: HashMap<String, Value>,
    ) -> Result<()> {
        if let Some(tag) = query.get("tag").and_then(|v| v.as_str()) {
            sqlx::query("DELETE FROM tokens WHERE device_uuid = $1 AND tag = $2")
                .bind(uuid)
                .bind(tag)
                .execute(&self.pool)
                .await
                .map_err(|e| FreshBluError::Storage(e.to_string()))?;
        } else if let Some(exp) = query.get("expiresOn").and_then(|v| v.as_i64()) {
            sqlx::query("DELETE FROM tokens WHERE device_uuid = $1 AND expires_on = $2")
                .bind(uuid)
                .bind(exp)
                .execute(&self.pool)
                .await
                .map_err(|e| FreshBluError::Storage(e.to_string()))?;
        }
        Ok(())
    }

    async fn list_tokens(&self, uuid: &Uuid) -> Result<Vec<TokenRecord>> {
        let rows = sqlx::query(
            "SELECT hash, created_at, expires_on, tag FROM tokens WHERE device_uuid = $1",
        )
        .bind(uuid)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| FreshBluError::Storage(e.to_string()))?;

        let mut records = Vec::new();
        for row in rows {
            let hash: String = row.get("hash");
            let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
            let expires_on: Option<i64> = row.get("expires_on");
            let tag: Option<String> = row.get("tag");
            records.push(TokenRecord {
                uuid: *uuid,
                hash,
                created_at,
                expires_on,
                tag,
            });
        }

        Ok(records)
    }

    async fn create_subscription(&self, params: &CreateSubscriptionParams) -> Result<Subscription> {
        sqlx::query(
            "INSERT INTO subscriptions (emitter_uuid, subscriber_uuid, subscription_type) \
             VALUES ($1, $2, $3) \
             ON CONFLICT (emitter_uuid, subscriber_uuid, subscription_type) DO NOTHING",
        )
        .bind(params.emitter_uuid)
        .bind(params.subscriber_uuid)
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
                    "DELETE FROM subscriptions WHERE subscriber_uuid = $1 AND emitter_uuid = $2 AND subscription_type = $3",
                )
                .bind(subscriber_uuid)
                .bind(e)
                .bind(t.to_string())
                .execute(&self.pool)
                .await
                .map_err(|e| FreshBluError::Storage(e.to_string()))?;
            }
            (Some(e), None) => {
                sqlx::query(
                    "DELETE FROM subscriptions WHERE subscriber_uuid = $1 AND emitter_uuid = $2",
                )
                .bind(subscriber_uuid)
                .bind(e)
                .execute(&self.pool)
                .await
                .map_err(|e| FreshBluError::Storage(e.to_string()))?;
            }
            _ => {
                sqlx::query("DELETE FROM subscriptions WHERE subscriber_uuid = $1")
                    .bind(subscriber_uuid)
                    .execute(&self.pool)
                    .await
                    .map_err(|e| FreshBluError::Storage(e.to_string()))?;
            }
        }
        Ok(())
    }

    async fn get_subscriptions(&self, subscriber_uuid: &Uuid) -> Result<Vec<Subscription>> {
        let rows = sqlx::query(
            "SELECT emitter_uuid, subscription_type FROM subscriptions WHERE subscriber_uuid = $1",
        )
        .bind(subscriber_uuid)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| FreshBluError::Storage(e.to_string()))?;

        let mut subs = Vec::new();
        for row in rows {
            let emitter: Uuid = row.get("emitter_uuid");
            let type_str: String = row.get("subscription_type");
            if let Ok(sub_type) = SubscriptionType::from_str(&type_str) {
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
            "SELECT subscriber_uuid FROM subscriptions WHERE emitter_uuid = $1 AND subscription_type = $2",
        )
        .bind(emitter_uuid)
        .bind(sub_type.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| FreshBluError::Storage(e.to_string()))?;

        let uuids: Vec<Uuid> = rows.iter().map(|r| r.get("subscriber_uuid")).collect();
        Ok(uuids)
    }
}
