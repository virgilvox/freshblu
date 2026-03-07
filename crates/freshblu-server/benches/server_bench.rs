use criterion::{criterion_group, criterion_main, Criterion};
use freshblu_core::message::{DeviceEvent, Message};
use freshblu_server::bus::MessageBus;
use freshblu_server::local_bus::LocalBus;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Runtime;
use uuid::Uuid;

fn make_runtime() -> Runtime {
    Runtime::new().unwrap()
}

fn make_msg(from: Uuid) -> Message {
    Message {
        devices: vec!["*".into()],
        from_uuid: Some(from),
        topic: None,
        payload: Some(json!({"key": "value"})),
        metadata: None,
        extra: HashMap::new(),
    }
}

fn bench_message_delivery_single(c: &mut Criterion) {
    let rt = make_runtime();
    let bus = Arc::new(LocalBus::new());
    let uuid = Uuid::new_v4();
    let _rx = bus.connect(uuid);

    let from = Uuid::new_v4();
    let event = DeviceEvent::Message(make_msg(from));

    c.bench_function("message_delivery_single", |b| {
        b.to_async(&rt).iter(|| {
            let event = event.clone();
            let bus = bus.clone();
            async move { bus.publish(&uuid, event).await.unwrap() }
        })
    });
}

fn bench_message_fanout_100(c: &mut Criterion) {
    let rt = make_runtime();
    let bus = Arc::new(LocalBus::new());

    let uuids: Vec<Uuid> = (0..100)
        .map(|_| {
            let u = Uuid::new_v4();
            let _rx = bus.connect(u);
            u
        })
        .collect();

    let from = Uuid::new_v4();
    let event = DeviceEvent::Message(make_msg(from));

    c.bench_function("message_fanout_100", |b| {
        b.to_async(&rt).iter(|| {
            let event = event.clone();
            let bus = bus.clone();
            let uuids = uuids.clone();
            async move { bus.publish_many(&uuids, event).await.unwrap() }
        })
    });
}

fn bench_message_fanout_10000(c: &mut Criterion) {
    let rt = make_runtime();
    let bus = Arc::new(LocalBus::new());

    let uuids: Vec<Uuid> = (0..10_000)
        .map(|_| {
            let u = Uuid::new_v4();
            let _rx = bus.connect(u);
            u
        })
        .collect();

    let from = Uuid::new_v4();
    let event = DeviceEvent::Message(make_msg(from));

    c.bench_function("message_fanout_10000", |b| {
        b.to_async(&rt).iter(|| {
            let event = event.clone();
            let bus = bus.clone();
            let uuids = uuids.clone();
            async move { bus.publish_many(&uuids, event).await.unwrap() }
        })
    });
}

fn bench_ws_connection_rate(c: &mut Criterion) {
    let bus = LocalBus::new();

    c.bench_function("bus_connect_disconnect", |b| {
        b.iter(|| {
            let uuid = Uuid::new_v4();
            let _rx = bus.connect(uuid);
            bus.disconnect(&uuid);
        })
    });
}

criterion_group!(
    benches,
    bench_message_delivery_single,
    bench_message_fanout_100,
    bench_message_fanout_10000,
    bench_ws_connection_rate
);
criterion_main!(benches);
