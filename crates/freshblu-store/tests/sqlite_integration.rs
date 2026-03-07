use freshblu_core::{
    device::RegisterParams,
    subscription::{CreateSubscriptionParams, SubscriptionType},
    token::GenerateTokenOptions,
};
use freshblu_store::{sqlite::SqliteStore, DeviceStore};
use serde_json::Value;
use std::collections::HashMap;

async fn setup() -> SqliteStore {
    SqliteStore::new("sqlite::memory:").await.unwrap()
}

#[tokio::test]
async fn register_and_get() {
    let store = setup().await;
    let params = RegisterParams {
        device_type: Some("test".into()),
        ..Default::default()
    };
    let (device, _token) = store.register(params).await.unwrap();

    let fetched = store.get_device(&device.uuid).await.unwrap();
    assert!(fetched.is_some());
    assert_eq!(fetched.unwrap().uuid, device.uuid);
}

#[tokio::test]
async fn register_returns_valid_token() {
    let store = setup().await;
    let params = RegisterParams {
        device_type: Some("test".into()),
        ..Default::default()
    };
    let (device, token) = store.register(params).await.unwrap();

    let result = store.authenticate(&device.uuid, &token).await.unwrap();
    assert!(result.is_some());
}

#[tokio::test]
async fn authenticate_valid() {
    let store = setup().await;
    let params = RegisterParams {
        device_type: Some("test".into()),
        ..Default::default()
    };
    let (device, token) = store.register(params).await.unwrap();

    let authed = store.authenticate(&device.uuid, &token).await.unwrap();
    assert!(authed.is_some());
    assert_eq!(authed.unwrap().uuid, device.uuid);
}

#[tokio::test]
async fn authenticate_invalid_token() {
    let store = setup().await;
    let params = RegisterParams {
        device_type: Some("test".into()),
        ..Default::default()
    };
    let (device, _token) = store.register(params).await.unwrap();

    let result = store
        .authenticate(&device.uuid, "totally-wrong-token")
        .await
        .unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn authenticate_expired_token() {
    let store = setup().await;
    let params = RegisterParams {
        device_type: Some("test".into()),
        ..Default::default()
    };
    let (device, _token) = store.register(params).await.unwrap();

    // Generate a token that expired in the past
    let opts = GenerateTokenOptions {
        expires_on: Some(1_000_000), // Unix timestamp far in the past
        tag: None,
    };
    let (_record, expired_token) = store.generate_token(&device.uuid, opts).await.unwrap();

    let result = store
        .authenticate(&device.uuid, &expired_token)
        .await
        .unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn update_device() {
    let store = setup().await;
    let params = RegisterParams {
        device_type: Some("test".into()),
        ..Default::default()
    };
    let (device, _token) = store.register(params).await.unwrap();

    let mut props = HashMap::new();
    props.insert("color".to_string(), Value::String("blue".to_string()));
    let updated = store.update_device(&device.uuid, props).await.unwrap();

    assert_eq!(
        updated.properties.get("color"),
        Some(&Value::String("blue".to_string()))
    );

    // Verify it persists
    let fetched = store.get_device(&device.uuid).await.unwrap().unwrap();
    assert_eq!(
        fetched.properties.get("color"),
        Some(&Value::String("blue".to_string()))
    );
}

#[tokio::test]
async fn update_ignores_system_fields() {
    let store = setup().await;
    let params = RegisterParams {
        device_type: Some("test".into()),
        ..Default::default()
    };
    let (device, _token) = store.register(params).await.unwrap();
    let original_uuid = device.uuid;

    let mut props = HashMap::new();
    props.insert(
        "uuid".to_string(),
        Value::String("xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx".to_string()),
    );
    props.insert("meshblu".to_string(), Value::Object(Default::default()));
    store.update_device(&device.uuid, props).await.unwrap();

    let fetched = store.get_device(&original_uuid).await.unwrap().unwrap();
    assert_eq!(fetched.uuid, original_uuid);
}

#[tokio::test]
async fn set_online() {
    let store = setup().await;
    let params = RegisterParams {
        device_type: Some("test".into()),
        ..Default::default()
    };
    let (device, _token) = store.register(params).await.unwrap();

    // set_online should not error
    store.set_online(&device.uuid, true).await.unwrap();

    // Device should still be retrievable
    let fetched = store.get_device(&device.uuid).await.unwrap();
    assert!(fetched.is_some());
}

#[tokio::test]
async fn unregister() {
    let store = setup().await;
    let params = RegisterParams {
        device_type: Some("test".into()),
        ..Default::default()
    };
    let (device, _token) = store.register(params).await.unwrap();

    store.unregister(&device.uuid).await.unwrap();

    let fetched = store.get_device(&device.uuid).await.unwrap();
    assert!(fetched.is_none());
}

#[tokio::test]
async fn search_devices_by_type() {
    let store = setup().await;

    let params_a = RegisterParams {
        device_type: Some("sensor".into()),
        ..Default::default()
    };
    store.register(params_a).await.unwrap();

    let params_b = RegisterParams {
        device_type: Some("gateway".into()),
        ..Default::default()
    };
    store.register(params_b).await.unwrap();

    let mut filters = HashMap::new();
    filters.insert("type".to_string(), Value::String("sensor".to_string()));

    let results = store.search_devices(&filters).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].device_type.as_deref(), Some("sensor"));
}

#[tokio::test]
async fn generate_and_use_additional_token() {
    let store = setup().await;
    let params = RegisterParams {
        device_type: Some("test".into()),
        ..Default::default()
    };
    let (device, _token) = store.register(params).await.unwrap();

    let opts = GenerateTokenOptions::default();
    let (_record, new_token) = store.generate_token(&device.uuid, opts).await.unwrap();

    let authed = store.authenticate(&device.uuid, &new_token).await.unwrap();
    assert!(authed.is_some());
    assert_eq!(authed.unwrap().uuid, device.uuid);
}

#[tokio::test]
async fn revoke_token() {
    let store = setup().await;
    let params = RegisterParams {
        device_type: Some("test".into()),
        ..Default::default()
    };
    let (device, token) = store.register(params).await.unwrap();

    // Confirm it works before revoking
    assert!(store
        .authenticate(&device.uuid, &token)
        .await
        .unwrap()
        .is_some());

    store.revoke_token(&device.uuid, &token).await.unwrap();

    let result = store.authenticate(&device.uuid, &token).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn list_tokens() {
    let store = setup().await;
    let params = RegisterParams {
        device_type: Some("test".into()),
        ..Default::default()
    };
    let (device, _token) = store.register(params).await.unwrap();

    // Generate an additional token
    let opts = GenerateTokenOptions::default();
    store.generate_token(&device.uuid, opts).await.unwrap();

    let tokens = store.list_tokens(&device.uuid).await.unwrap();
    assert_eq!(tokens.len(), 2);
}

#[tokio::test]
async fn create_subscription() {
    let store = setup().await;
    let params = RegisterParams {
        device_type: Some("test".into()),
        ..Default::default()
    };
    let (device, _token) = store.register(params).await.unwrap();

    let sub_params = CreateSubscriptionParams {
        emitter_uuid: device.uuid,
        subscriber_uuid: device.uuid,
        subscription_type: SubscriptionType::BroadcastSent,
    };
    let sub = store.create_subscription(&sub_params).await.unwrap();

    assert_eq!(sub.emitter_uuid, device.uuid);
    assert_eq!(sub.subscriber_uuid, device.uuid);
    assert_eq!(sub.subscription_type, SubscriptionType::BroadcastSent);

    let subs = store.get_subscriptions(&device.uuid).await.unwrap();
    assert_eq!(subs.len(), 1);
}

#[tokio::test]
async fn create_duplicate_subscription_idempotent() {
    let store = setup().await;
    let params = RegisterParams {
        device_type: Some("test".into()),
        ..Default::default()
    };
    let (device, _token) = store.register(params).await.unwrap();

    let sub_params = CreateSubscriptionParams {
        emitter_uuid: device.uuid,
        subscriber_uuid: device.uuid,
        subscription_type: SubscriptionType::BroadcastSent,
    };
    store.create_subscription(&sub_params).await.unwrap();
    store.create_subscription(&sub_params).await.unwrap();

    let subs = store.get_subscriptions(&device.uuid).await.unwrap();
    assert_eq!(subs.len(), 1);
}

#[tokio::test]
async fn delete_subscription() {
    let store = setup().await;
    let params = RegisterParams {
        device_type: Some("test".into()),
        ..Default::default()
    };
    let (device, _token) = store.register(params).await.unwrap();

    let sub_params = CreateSubscriptionParams {
        emitter_uuid: device.uuid,
        subscriber_uuid: device.uuid,
        subscription_type: SubscriptionType::BroadcastSent,
    };
    store.create_subscription(&sub_params).await.unwrap();

    store
        .delete_subscription(
            &device.uuid,
            Some(&device.uuid),
            Some(&SubscriptionType::BroadcastSent),
        )
        .await
        .unwrap();

    let subs = store.get_subscriptions(&device.uuid).await.unwrap();
    assert!(subs.is_empty());
}

#[tokio::test]
async fn get_subscribers() {
    let store = setup().await;

    let params_a = RegisterParams {
        device_type: Some("test".into()),
        ..Default::default()
    };
    let (device_a, _) = store.register(params_a).await.unwrap();

    let params_b = RegisterParams {
        device_type: Some("test".into()),
        ..Default::default()
    };
    let (device_b, _) = store.register(params_b).await.unwrap();

    let sub_params = CreateSubscriptionParams {
        emitter_uuid: device_b.uuid,
        subscriber_uuid: device_a.uuid,
        subscription_type: SubscriptionType::BroadcastSent,
    };
    store.create_subscription(&sub_params).await.unwrap();

    let subscribers = store
        .get_subscribers(&device_b.uuid, &SubscriptionType::BroadcastSent)
        .await
        .unwrap();
    assert_eq!(subscribers.len(), 1);
    assert_eq!(subscribers[0], device_a.uuid);
}
