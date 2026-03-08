use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use freshblu_core::{
    device::{Device, DeviceView, RegisterParams},
    error::Result,
    subscription::{CreateSubscriptionParams, Subscription, SubscriptionType},
    token::{GenerateTokenOptions, TokenRecord},
};
use serde_json::Value;
use uuid::Uuid;

/// Trait for all storage backends.
/// Implement this to add PostgreSQL, Redis, etc.
#[async_trait]
pub trait DeviceStore: Send + Sync + 'static {
    // -- Device CRUD --

    /// Register a new device and return it with the plaintext token
    async fn register(&self, params: RegisterParams) -> Result<(Device, String)>;

    /// Get a device by UUID (returns None if not found or not permitted)
    async fn get_device(&self, uuid: &Uuid) -> Result<Option<Device>>;

    /// Update a device's properties
    async fn update_device(
        &self,
        uuid: &Uuid,
        properties: HashMap<String, Value>,
    ) -> Result<Device>;

    /// Set device online/offline status
    async fn set_online(&self, uuid: &Uuid, online: bool) -> Result<()>;

    /// Delete a device
    async fn unregister(&self, uuid: &Uuid) -> Result<()>;

    /// Search devices by property filters (caller must check permissions)
    async fn search_devices(&self, filters: &HashMap<String, Value>) -> Result<Vec<DeviceView>>;

    /// Find all devices owned by the given UUID
    async fn find_by_owner(&self, owner: &Uuid) -> Result<Vec<DeviceView>>;

    // -- Auth --

    /// Verify uuid + token, returns device if valid
    async fn authenticate(&self, uuid: &Uuid, token: &str) -> Result<Option<Device>>;

    // -- Token management --

    /// Generate a new token for a device and store its hash
    async fn generate_token(
        &self,
        uuid: &Uuid,
        opts: GenerateTokenOptions,
    ) -> Result<(TokenRecord, String)>;

    /// Revoke a specific token
    async fn revoke_token(&self, uuid: &Uuid, token: &str) -> Result<()>;

    /// Revoke tokens matching a query (e.g. by tag or expiry)
    async fn revoke_tokens_by_query(
        &self,
        uuid: &Uuid,
        query: HashMap<String, Value>,
    ) -> Result<()>;

    /// Get all tokens for a device (hashes only, never plaintext)
    async fn list_tokens(&self, uuid: &Uuid) -> Result<Vec<TokenRecord>>;

    // -- Device claim --

    /// Claim a device: set owner and switch to private whitelists.
    /// Returns error if already claimed.
    async fn claim_device(&self, uuid: &Uuid, owner: &Uuid) -> Result<Device>;

    // -- Token reset --

    /// Revoke all existing tokens, generate a new root token
    async fn reset_token(&self, uuid: &Uuid) -> Result<String>;

    // -- Token search --

    /// Search tokens by tag, device UUID, or expiry
    async fn search_tokens(&self, query: &HashMap<String, Value>) -> Result<Vec<TokenRecord>>;

    // -- Subscriptions --

    /// Create a subscription
    async fn create_subscription(&self, params: &CreateSubscriptionParams) -> Result<Subscription>;

    /// Delete subscriptions matching criteria
    async fn delete_subscription(
        &self,
        subscriber_uuid: &Uuid,
        emitter_uuid: Option<&Uuid>,
        sub_type: Option<&SubscriptionType>,
    ) -> Result<()>;

    /// Get all subscriptions for a subscriber
    async fn get_subscriptions(&self, subscriber_uuid: &Uuid) -> Result<Vec<Subscription>>;

    /// Get all subscribers for an emitter + type (for routing)
    async fn get_subscribers(
        &self,
        emitter_uuid: &Uuid,
        sub_type: &SubscriptionType,
    ) -> Result<Vec<Uuid>>;
}

pub type DynStore = Arc<dyn DeviceStore>;
