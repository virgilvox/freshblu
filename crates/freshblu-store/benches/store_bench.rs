use criterion::{criterion_group, criterion_main, Criterion};
use freshblu_core::device::RegisterParams;
use freshblu_core::subscription::{CreateSubscriptionParams, SubscriptionType};
use freshblu_store::{sqlite::SqliteStore, DeviceStore};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Runtime;

fn make_runtime() -> Runtime {
    Runtime::new().unwrap()
}

async fn make_store() -> Arc<SqliteStore> {
    Arc::new(SqliteStore::new("sqlite::memory:").await.unwrap())
}

fn bench_register(c: &mut Criterion) {
    let rt = make_runtime();
    let store = rt.block_on(make_store());

    c.bench_function("store_register", |b| {
        let store = store.clone();
        b.to_async(&rt).iter(|| {
            let store = store.clone();
            async move {
                let params = RegisterParams {
                    device_type: Some("bench".into()),
                    ..Default::default()
                };
                store.register(params).await.unwrap()
            }
        })
    });
}

fn bench_authenticate(c: &mut Criterion) {
    let rt = make_runtime();
    let store = rt.block_on(make_store());
    let (device, token) = rt.block_on(async {
        let params = RegisterParams {
            device_type: Some("bench".into()),
            ..Default::default()
        };
        store.register(params).await.unwrap()
    });

    c.bench_function("store_authenticate", |b| {
        let store = store.clone();
        b.to_async(&rt).iter(|| {
            let store = store.clone();
            let token = token.clone();
            async move {
                store.authenticate(&device.uuid, &token).await.unwrap()
            }
        })
    });
}

fn bench_get_device(c: &mut Criterion) {
    let rt = make_runtime();
    let store = rt.block_on(make_store());
    let (device, _) = rt.block_on(async {
        let params = RegisterParams {
            device_type: Some("bench".into()),
            ..Default::default()
        };
        store.register(params).await.unwrap()
    });

    c.bench_function("store_get_device", |b| {
        let store = store.clone();
        b.to_async(&rt).iter(|| {
            let store = store.clone();
            async move { store.get_device(&device.uuid).await.unwrap() }
        })
    });
}

fn bench_search_devices(c: &mut Criterion) {
    let rt = make_runtime();
    let store = rt.block_on(make_store());

    rt.block_on(async {
        for i in 0..100 {
            let dtype = if i < 50 { "sensor" } else { "gateway" };
            let params = RegisterParams {
                device_type: Some(dtype.into()),
                ..Default::default()
            };
            store.register(params).await.unwrap();
        }
    });

    let mut filters = HashMap::new();
    filters.insert(
        "type".to_string(),
        serde_json::Value::String("sensor".into()),
    );

    c.bench_function("store_search_devices", |b| {
        let store = store.clone();
        b.to_async(&rt).iter(|| {
            let store = store.clone();
            let filters = filters.clone();
            async move { store.search_devices(&filters).await.unwrap() }
        })
    });
}

fn bench_get_subscribers(c: &mut Criterion) {
    let rt = make_runtime();
    let store = rt.block_on(make_store());

    let emitter_uuid = rt.block_on(async {
        let emitter_params = RegisterParams {
            device_type: Some("emitter".into()),
            ..Default::default()
        };
        let (emitter, _) = store.register(emitter_params).await.unwrap();

        for _ in 0..20 {
            let sub_params = RegisterParams {
                device_type: Some("sub".into()),
                ..Default::default()
            };
            let (sub, _) = store.register(sub_params).await.unwrap();
            store
                .create_subscription(&CreateSubscriptionParams {
                    emitter_uuid: emitter.uuid,
                    subscriber_uuid: sub.uuid,
                    subscription_type: SubscriptionType::BroadcastSent,
                })
                .await
                .unwrap();
        }
        emitter.uuid
    });

    c.bench_function("store_get_subscribers", |b| {
        let store = store.clone();
        b.to_async(&rt).iter(|| {
            let store = store.clone();
            async move {
                store
                    .get_subscribers(&emitter_uuid, &SubscriptionType::BroadcastSent)
                    .await
                    .unwrap()
            }
        })
    });
}

criterion_group!(
    benches,
    bench_register,
    bench_authenticate,
    bench_get_device,
    bench_search_devices,
    bench_get_subscribers
);
criterion_main!(benches);
