use freshblu_core::{
    device::RegisterParams,
    subscription::{CreateSubscriptionParams, SubscriptionType},
    token::GenerateTokenOptions,
};
use freshblu_store::{sqlite::SqliteStore, DeviceStore};
use serde_json::Value;
use std::collections::HashMap;

async fn new_store() -> SqliteStore {
    SqliteStore::new("sqlite::memory:").await.unwrap()
}

// ---------------------------------------------------------------------------
// Ported from sqlite_integration.rs (17 tests)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn register_and_get() {
    let store = new_store().await;
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
    let store = new_store().await;
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
    let store = new_store().await;
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
    let store = new_store().await;
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
    let store = new_store().await;
    let params = RegisterParams {
        device_type: Some("test".into()),
        ..Default::default()
    };
    let (device, _token) = store.register(params).await.unwrap();

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
    let store = new_store().await;
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

    let fetched = store.get_device(&device.uuid).await.unwrap().unwrap();
    assert_eq!(
        fetched.properties.get("color"),
        Some(&Value::String("blue".to_string()))
    );
}

#[tokio::test]
async fn update_ignores_system_fields() {
    let store = new_store().await;
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
    let store = new_store().await;
    let params = RegisterParams {
        device_type: Some("test".into()),
        ..Default::default()
    };
    let (device, _token) = store.register(params).await.unwrap();

    store.set_online(&device.uuid, true).await.unwrap();

    let fetched = store.get_device(&device.uuid).await.unwrap();
    assert!(fetched.is_some());
}

#[tokio::test]
async fn unregister() {
    let store = new_store().await;
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
    let store = new_store().await;

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
    let store = new_store().await;
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
    let store = new_store().await;
    let params = RegisterParams {
        device_type: Some("test".into()),
        ..Default::default()
    };
    let (device, token) = store.register(params).await.unwrap();

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
    let store = new_store().await;
    let params = RegisterParams {
        device_type: Some("test".into()),
        ..Default::default()
    };
    let (device, _token) = store.register(params).await.unwrap();

    let opts = GenerateTokenOptions::default();
    store.generate_token(&device.uuid, opts).await.unwrap();

    let tokens = store.list_tokens(&device.uuid).await.unwrap();
    assert_eq!(tokens.len(), 2);
}

#[tokio::test]
async fn create_subscription() {
    let store = new_store().await;
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
    let store = new_store().await;
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
    let store = new_store().await;
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
    let store = new_store().await;

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

// ---------------------------------------------------------------------------
// New conformance tests (9 tests)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn register_100_unique_uuids() {
    let store = new_store().await;
    let mut uuids = std::collections::HashSet::new();

    for _ in 0..100 {
        let params = RegisterParams {
            device_type: Some("bulk".into()),
            ..Default::default()
        };
        let (device, _token) = store.register(params).await.unwrap();
        assert!(
            uuids.insert(device.uuid),
            "duplicate UUID detected: {}",
            device.uuid
        );
    }

    assert_eq!(uuids.len(), 100);
}

#[tokio::test]
async fn update_merges_properties() {
    let store = new_store().await;
    let params = RegisterParams {
        device_type: Some("test".into()),
        ..Default::default()
    };
    let (device, _token) = store.register(params).await.unwrap();

    // First update: set color
    let mut props1 = HashMap::new();
    props1.insert("color".to_string(), Value::String("blue".to_string()));
    store.update_device(&device.uuid, props1).await.unwrap();

    // Second update: set name (should not erase color)
    let mut props2 = HashMap::new();
    props2.insert("name".to_string(), Value::String("test".to_string()));
    let updated = store.update_device(&device.uuid, props2).await.unwrap();

    assert_eq!(
        updated.properties.get("color"),
        Some(&Value::String("blue".to_string())),
        "color should still be present after setting name"
    );
    assert_eq!(
        updated.properties.get("name"),
        Some(&Value::String("test".to_string())),
        "name should be present after update"
    );

    // Verify persistence
    let fetched = store.get_device(&device.uuid).await.unwrap().unwrap();
    assert_eq!(
        fetched.properties.get("color"),
        Some(&Value::String("blue".to_string()))
    );
    assert_eq!(
        fetched.properties.get("name"),
        Some(&Value::String("test".to_string()))
    );
}

#[tokio::test]
async fn unregister_cascades_tokens() {
    let store = new_store().await;
    let params = RegisterParams {
        device_type: Some("test".into()),
        ..Default::default()
    };
    let (device, _token) = store.register(params).await.unwrap();

    // Generate an extra token
    let opts = GenerateTokenOptions::default();
    store.generate_token(&device.uuid, opts).await.unwrap();

    // Verify tokens exist before unregister
    let tokens_before = store.list_tokens(&device.uuid).await.unwrap();
    assert_eq!(tokens_before.len(), 2);

    // Unregister device
    store.unregister(&device.uuid).await.unwrap();

    // Tokens should be gone (cascade delete or device gone)
    let tokens_after = store.list_tokens(&device.uuid).await;
    match tokens_after {
        Ok(tokens) => assert!(
            tokens.is_empty(),
            "tokens should be empty after unregister, got {}",
            tokens.len()
        ),
        Err(_) => {
            // Also acceptable: error because device does not exist
        }
    }
}

#[tokio::test]
async fn unregister_cascades_subscriptions() {
    let store = new_store().await;

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

    // A subscribes to B's broadcasts
    let sub_params = CreateSubscriptionParams {
        emitter_uuid: device_b.uuid,
        subscriber_uuid: device_a.uuid,
        subscription_type: SubscriptionType::BroadcastSent,
    };
    store.create_subscription(&sub_params).await.unwrap();

    // Unregister A (the subscriber)
    store.unregister(&device_a.uuid).await.unwrap();

    // Check B's subscribers -- document the actual behavior
    let subscribers = store
        .get_subscribers(&device_b.uuid, &SubscriptionType::BroadcastSent)
        .await
        .unwrap();

    assert!(
        subscribers.is_empty(),
        "subscriptions should cascade-delete when subscriber device is removed"
    );
}

#[tokio::test]
async fn search_by_online() {
    let store = new_store().await;

    let params1 = RegisterParams {
        device_type: Some("test".into()),
        ..Default::default()
    };
    let (device1, _) = store.register(params1).await.unwrap();

    let params2 = RegisterParams {
        device_type: Some("test".into()),
        ..Default::default()
    };
    let (_device2, _) = store.register(params2).await.unwrap();

    // Set device1 online
    store.set_online(&device1.uuid, true).await.unwrap();

    let mut filters = HashMap::new();
    filters.insert("online".to_string(), Value::Bool(true));

    let results = store.search_devices(&filters).await.unwrap();
    assert_eq!(
        results.len(),
        1,
        "only one device should be online, got {}",
        results.len()
    );
}

#[tokio::test]
async fn search_limit_100() {
    let store = new_store().await;

    // Register 110 devices
    for i in 0..110 {
        let params = RegisterParams {
            device_type: Some(format!("bulk-{}", i)),
            ..Default::default()
        };
        store.register(params).await.unwrap();
    }

    // Search with no filters
    let filters = HashMap::new();
    let results = store.search_devices(&filters).await.unwrap();

    assert!(
        results.len() <= 100,
        "search should return at most 100 results, got {}",
        results.len()
    );
}

#[tokio::test]
async fn authenticate_future_token() {
    let store = new_store().await;
    let params = RegisterParams {
        device_type: Some("test".into()),
        ..Default::default()
    };
    let (device, _token) = store.register(params).await.unwrap();

    // Generate a token with expiry far in the future (year ~2100)
    let opts = GenerateTokenOptions {
        expires_on: Some(4_102_444_800), // 2100-01-01
        tag: None,
    };
    let (_record, future_token) = store.generate_token(&device.uuid, opts).await.unwrap();

    let result = store
        .authenticate(&device.uuid, &future_token)
        .await
        .unwrap();
    assert!(
        result.is_some(),
        "token with future expiry should authenticate successfully"
    );
}

#[tokio::test]
async fn revoke_tokens_by_tag() {
    let store = new_store().await;
    let params = RegisterParams {
        device_type: Some("test".into()),
        ..Default::default()
    };
    let (device, _initial_token) = store.register(params).await.unwrap();

    // Generate tokens with tag "temp"
    let opts1 = GenerateTokenOptions {
        expires_on: None,
        tag: Some("temp".to_string()),
    };
    let (_, tagged_token1) = store.generate_token(&device.uuid, opts1).await.unwrap();

    let opts2 = GenerateTokenOptions {
        expires_on: None,
        tag: Some("temp".to_string()),
    };
    let (_, tagged_token2) = store.generate_token(&device.uuid, opts2).await.unwrap();

    // Generate a token with a different tag
    let opts3 = GenerateTokenOptions {
        expires_on: None,
        tag: Some("permanent".to_string()),
    };
    let (_, perm_token) = store.generate_token(&device.uuid, opts3).await.unwrap();

    // Verify all tokens work
    assert!(store
        .authenticate(&device.uuid, &tagged_token1)
        .await
        .unwrap()
        .is_some());
    assert!(store
        .authenticate(&device.uuid, &tagged_token2)
        .await
        .unwrap()
        .is_some());
    assert!(store
        .authenticate(&device.uuid, &perm_token)
        .await
        .unwrap()
        .is_some());

    // Revoke by tag "temp"
    let mut query = HashMap::new();
    query.insert("tag".to_string(), Value::String("temp".to_string()));
    store
        .revoke_tokens_by_query(&device.uuid, query)
        .await
        .unwrap();

    // Tagged tokens should be gone
    assert!(
        store
            .authenticate(&device.uuid, &tagged_token1)
            .await
            .unwrap()
            .is_none(),
        "tagged token 1 should be revoked"
    );
    assert!(
        store
            .authenticate(&device.uuid, &tagged_token2)
            .await
            .unwrap()
            .is_none(),
        "tagged token 2 should be revoked"
    );

    // Permanent token should still work
    assert!(
        store
            .authenticate(&device.uuid, &perm_token)
            .await
            .unwrap()
            .is_some(),
        "permanent token should still be valid"
    );
}

#[tokio::test]
async fn concurrent_register_50() {
    let store = std::sync::Arc::new(new_store().await);
    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(5));
    let mut handles = Vec::new();

    for _ in 0..50 {
        let store = store.clone();
        let sem = semaphore.clone();
        handles.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            let params = RegisterParams {
                device_type: Some("concurrent".into()),
                ..Default::default()
            };
            store.register(params).await.unwrap()
        }));
    }

    let mut uuids = std::collections::HashSet::new();
    for handle in handles {
        let (device, _token) = handle.await.unwrap();
        assert!(
            uuids.insert(device.uuid),
            "duplicate UUID in concurrent registration: {}",
            device.uuid
        );
    }

    assert_eq!(
        uuids.len(),
        50,
        "all 50 concurrent registrations should produce unique UUIDs"
    );
}
