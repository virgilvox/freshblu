use criterion::{black_box, criterion_group, criterion_main, Criterion};
use freshblu_core::auth::{compute_device_hash, generate_token, hash_token, verify_token};
use freshblu_core::device::{Device, WhitelistEntry};
use freshblu_core::message::{DeviceEvent, Message};
use freshblu_core::permissions::{check_whitelist, Whitelists};
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

fn bench_bcrypt_hash(c: &mut Criterion) {
    let token = generate_token();
    c.bench_function("bcrypt_hash", |b| {
        b.iter(|| hash_token(black_box(&token)).unwrap())
    });
}

fn bench_bcrypt_verify(c: &mut Criterion) {
    let token = generate_token();
    let hash = hash_token(&token).unwrap();
    c.bench_function("bcrypt_verify", |b| {
        b.iter(|| verify_token(black_box(&token), black_box(&hash)))
    });
}

fn bench_check_whitelist_5(c: &mut Criterion) {
    let entries: Vec<WhitelistEntry> = (0..5)
        .map(|_| WhitelistEntry::for_uuid(&Uuid::new_v4()))
        .collect();
    let target = Uuid::new_v4();

    c.bench_function("check_whitelist_5_entries", |b| {
        b.iter(|| check_whitelist(black_box(&entries), black_box(&target)))
    });
}

fn bench_check_whitelist_1000(c: &mut Criterion) {
    let mut entries: Vec<WhitelistEntry> = (0..1000)
        .map(|_| WhitelistEntry::for_uuid(&Uuid::new_v4()))
        .collect();
    let target = Uuid::new_v4();
    // Put the matching entry at position 500
    entries[500] = WhitelistEntry::for_uuid(&target);

    c.bench_function("check_whitelist_1000_entries", |b| {
        b.iter(|| check_whitelist(black_box(&entries), black_box(&target)))
    });
}

fn bench_device_serialize(c: &mut Criterion) {
    let mut props = HashMap::new();
    props.insert("name".into(), json!("test-device"));
    props.insert("color".into(), json!("blue"));
    let device = Device::new(props, Whitelists::open());

    c.bench_function("device_serialize", |b| {
        b.iter(|| serde_json::to_string(black_box(&device)).unwrap())
    });
}

fn bench_device_deserialize(c: &mut Criterion) {
    let mut props = HashMap::new();
    props.insert("name".into(), json!("test-device"));
    props.insert("color".into(), json!("blue"));
    let device = Device::new(props, Whitelists::open());
    let json_str = serde_json::to_string(&device).unwrap();

    c.bench_function("device_deserialize", |b| {
        b.iter(|| serde_json::from_str::<Device>(black_box(&json_str)).unwrap())
    });
}

fn bench_device_event_serialize(c: &mut Criterion) {
    let msg = Message {
        devices: vec!["*".into()],
        from_uuid: Some(Uuid::new_v4()),
        topic: Some("test".into()),
        payload: Some(json!({"key": "value", "numbers": [1,2,3]})),
        metadata: None,
        extra: HashMap::new(),
    };
    let event = DeviceEvent::Message(msg);

    c.bench_function("device_event_serialize", |b| {
        b.iter(|| serde_json::to_string(black_box(&event)).unwrap())
    });
}

criterion_group!(
    benches,
    bench_bcrypt_hash,
    bench_bcrypt_verify,
    bench_check_whitelist_5,
    bench_check_whitelist_1000,
    bench_device_serialize,
    bench_device_deserialize,
    bench_device_event_serialize
);
criterion_main!(benches);
