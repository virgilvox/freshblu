//! Arduino library wire-compatibility tests.
//!
//! These tests use raw TCP to send HTTP requests formatted exactly as the
//! FreshBlu Arduino library (`libraries/FreshBlu/`) produces them, then parse
//! responses exactly as the Arduino library does. This validates that the
//! server's wire format is compatible with the embedded client.

mod helpers;

use base64::Engine;
use helpers::*;
use serde_json::Value;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

// ---------------------------------------------------------------------------
// Raw-TCP HTTP helper — mimics FreshBluHttp::_request() byte-for-byte
// ---------------------------------------------------------------------------

struct RawResponse {
    status: u16,
    body: String,
}

/// Send an HTTP request using the exact format the Arduino library uses:
/// - Manual HTTP/1.1 request line
/// - Host header
/// - Connection: close
/// - Optional Authorization: Basic <base64>
/// - Optional Content-Type: application/json + Content-Length
fn arduino_request(
    port: u16,
    method: &str,
    path: &str,
    auth: Option<(&str, &str)>, // (uuid, token)
    body: Option<&str>,
) -> RawResponse {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();

    // Request line — exactly as Arduino print(method) + print(' ') + print(path) + println(" HTTP/1.1")
    let mut req = format!("{} {} HTTP/1.1\r\n", method, path);

    // Host header
    req.push_str("Host: 127.0.0.1\r\n");

    // Connection: close
    req.push_str("Connection: close\r\n");

    // Auth header — Arduino computes Base64 of "uuid:token" and sends "Basic <encoded>"
    if let Some((uuid, token)) = auth {
        let plain = format!("{}:{}", uuid, token);
        let encoded = arduino_base64(plain.as_bytes());
        req.push_str(&format!("Authorization: Basic {}\r\n", encoded));
    }

    // Body headers
    if let Some(b) = body {
        req.push_str("Content-Type: application/json\r\n");
        req.push_str(&format!("Content-Length: {}\r\n", b.len()));
    }

    // End of headers
    req.push_str("\r\n");

    // Body
    if let Some(b) = body {
        req.push_str(b);
    }

    stream.write_all(req.as_bytes()).unwrap();
    stream.flush().unwrap();

    // Read response line-by-line using BufRead for reliable parsing
    use std::io::BufRead;
    let mut reader = std::io::BufReader::new(&stream);

    // 1. Read status line
    let mut status_line = String::new();
    reader.read_line(&mut status_line).unwrap();
    let status: u16 = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    // 2. Read headers, extract Content-Length
    let mut content_length: usize = 0;
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        let trimmed = line.trim();
        if trimmed.is_empty() {
            break; // blank line = end of headers
        }
        if let Some(val) = trimmed.strip_prefix("content-length:") {
            content_length = val.trim().parse().unwrap_or(0);
        } else if let Some(val) = trimmed.strip_prefix("Content-Length:") {
            content_length = val.trim().parse().unwrap_or(0);
        }
    }

    // 3. Read exactly content_length bytes of body
    let mut body_buf = vec![0u8; content_length];
    if content_length > 0 {
        reader.read_exact(&mut body_buf).unwrap();
    }
    let body_str = String::from_utf8_lossy(&body_buf).to_string();

    drop(reader);
    let _ = stream.shutdown(std::net::Shutdown::Both);

    RawResponse {
        status,
        body: body_str,
    }
}

/// Re-implementation of the Arduino library's Base64 encoder to verify
/// our encoder produces the same output as the standard.
fn arduino_base64(input: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = Vec::new();
    let mut i = 0;

    while i + 2 < input.len() {
        let a = input[i];
        let b = input[i + 1];
        let c = input[i + 2];
        output.push(CHARS[((a >> 2) & 0x3f) as usize]);
        output.push(CHARS[(((a & 0x03) << 4) | ((b >> 4) & 0x0f)) as usize]);
        output.push(CHARS[(((b & 0x0f) << 2) | ((c >> 6) & 0x03)) as usize]);
        output.push(CHARS[(c & 0x3f) as usize]);
        i += 3;
    }

    let remaining = input.len() - i;
    if remaining == 1 {
        let a = input[i];
        output.push(CHARS[((a >> 2) & 0x3f) as usize]);
        output.push(CHARS[((a & 0x03) << 4) as usize]);
        output.push(b'=');
        output.push(b'=');
    } else if remaining == 2 {
        let a = input[i];
        let b = input[i + 1];
        output.push(CHARS[((a >> 2) & 0x3f) as usize]);
        output.push(CHARS[(((a & 0x03) << 4) | ((b >> 4) & 0x0f)) as usize]);
        output.push(CHARS[((b & 0x0f) << 2) as usize]);
        output.push(b'=');
    }

    String::from_utf8(output).unwrap()
}

/// Extract the port from the ws_url returned by setup()
fn port_from_ws_url(ws_url: &str) -> u16 {
    // ws://127.0.0.1:PORT/ws
    ws_url
        .split(':')
        .nth(2)
        .and_then(|s| s.split('/').next())
        .and_then(|s| s.parse().ok())
        .unwrap()
}

fn parse_json(body: &str) -> Value {
    serde_json::from_str(body).expect(&format!("failed to parse JSON: {}", body))
}

// ===========================================================================
// Base64 encoding compatibility
// ===========================================================================

#[test]
fn arduino_base64_matches_standard() {
    // Verify our re-implementation of the Arduino Base64 encoder matches
    // the standard base64 crate that the server uses to decode.
    let test_cases = [
        "hello:world",
        "550e8400-e29b-41d4-a716-446655440000:abcdef1234567890abcdef",
        "a:b",
        "uuid-with-dashes:token-with-64-chars-0123456789abcdef0123456789abcdef01234",
        "", // empty
        "x",
        "xy",
    ];

    for input in &test_cases {
        let arduino = arduino_base64(input.as_bytes());
        let standard = base64::engine::general_purpose::STANDARD.encode(input.as_bytes());
        assert_eq!(
            arduino, standard,
            "Base64 mismatch for input {:?}",
            input
        );
    }
}

// ===========================================================================
// GET /status — no auth required
// ===========================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn raw_tcp_status() {
    let (ws_url, _state) = setup().await;
    let port = port_from_ws_url(&ws_url);

    let resp = arduino_request(port, "GET", "/status", None, None);
    assert_eq!(resp.status, 200);

    let json = parse_json(&resp.body);
    assert_eq!(json["meshblu"], true);
    assert_eq!(json["online"], true);
    assert!(json["engine"].as_str().is_some());
}

// ===========================================================================
// POST /devices — register a device
// ===========================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn raw_tcp_register_device() {
    let (ws_url, _state) = setup().await;
    let port = port_from_ws_url(&ws_url);

    let body = r#"{"type":"sensor","name":"temp-01"}"#;
    let resp = arduino_request(port, "POST", "/devices", None, Some(body));
    assert_eq!(resp.status, 200);

    let json = parse_json(&resp.body);
    assert!(json["uuid"].as_str().is_some(), "response must contain uuid");
    assert!(
        json["token"].as_str().is_some(),
        "response must contain token"
    );
    assert_eq!(json["type"], "sensor");
    assert_eq!(json["name"], "temp-01");

    // UUID should be valid format (36 chars with dashes)
    let uuid = json["uuid"].as_str().unwrap();
    assert_eq!(uuid.len(), 36);
    assert!(uuid::Uuid::parse_str(uuid).is_ok());

    // Token should be non-empty
    let token = json["token"].as_str().unwrap();
    assert!(!token.is_empty());
}

// ===========================================================================
// POST /devices with empty body
// ===========================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn raw_tcp_register_empty_props() {
    let (ws_url, _state) = setup().await;
    let port = port_from_ws_url(&ws_url);

    let resp = arduino_request(port, "POST", "/devices", None, Some("{}"));
    assert_eq!(resp.status, 200);

    let json = parse_json(&resp.body);
    assert!(json["uuid"].as_str().is_some());
    assert!(json["token"].as_str().is_some());
}

// ===========================================================================
// GET /whoami — with Basic auth from register response
// ===========================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn raw_tcp_register_then_whoami() {
    let (ws_url, _state) = setup().await;
    let port = port_from_ws_url(&ws_url);

    // Register
    let resp = arduino_request(
        port,
        "POST",
        "/devices",
        None,
        Some(r#"{"type":"whoami-test"}"#),
    );
    assert_eq!(resp.status, 200);
    let reg = parse_json(&resp.body);
    let uuid = reg["uuid"].as_str().unwrap();
    let token = reg["token"].as_str().unwrap();

    // Whoami with the same credentials — exactly as Arduino setAuth + whoami
    let resp = arduino_request(port, "GET", "/whoami", Some((uuid, token)), None);
    assert_eq!(resp.status, 200);

    let json = parse_json(&resp.body);
    assert_eq!(json["uuid"], uuid);
}

// ===========================================================================
// POST /authenticate — body-based auth
// ===========================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn raw_tcp_authenticate() {
    let (ws_url, _state) = setup().await;
    let port = port_from_ws_url(&ws_url);

    // Register first
    let resp = arduino_request(port, "POST", "/devices", None, Some("{}"));
    assert_eq!(resp.status, 200);
    let reg = parse_json(&resp.body);
    let uuid = reg["uuid"].as_str().unwrap();
    let token = reg["token"].as_str().unwrap();

    // Authenticate — Arduino sends uuid+token in body, no auth header
    let auth_body = format!(r#"{{"uuid":"{}","token":"{}"}}"#, uuid, token);
    let resp = arduino_request(port, "POST", "/authenticate", None, Some(&auth_body));
    assert_eq!(resp.status, 200);

    let json = parse_json(&resp.body);
    assert_eq!(json["uuid"], uuid);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn raw_tcp_authenticate_bad_token() {
    let (ws_url, _state) = setup().await;
    let port = port_from_ws_url(&ws_url);

    // Register
    let resp = arduino_request(port, "POST", "/devices", None, Some("{}"));
    let reg = parse_json(&resp.body);
    let uuid = reg["uuid"].as_str().unwrap();

    // Authenticate with wrong token
    let auth_body = format!(r#"{{"uuid":"{}","token":"wrong-token"}}"#, uuid);
    let resp = arduino_request(port, "POST", "/authenticate", None, Some(&auth_body));
    assert_eq!(resp.status, 401);

    let json = parse_json(&resp.body);
    assert!(json["error"].as_str().is_some());
}

// ===========================================================================
// GET /devices/:uuid — retrieve another device
// ===========================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn raw_tcp_get_device() {
    let (ws_url, _state) = setup().await;
    let port = port_from_ws_url(&ws_url);

    // Register device A
    let resp = arduino_request(
        port,
        "POST",
        "/devices",
        None,
        Some(r#"{"type":"target"}"#),
    );
    let reg_a = parse_json(&resp.body);
    let uuid_a = reg_a["uuid"].as_str().unwrap().to_string();

    // Register device B (the one making the request)
    let resp = arduino_request(
        port,
        "POST",
        "/devices",
        None,
        Some(r#"{"type":"requester"}"#),
    );
    let reg_b = parse_json(&resp.body);
    let uuid_b = reg_b["uuid"].as_str().unwrap().to_string();
    let token_b = reg_b["token"].as_str().unwrap().to_string();

    // B gets A
    let path = format!("/devices/{}", uuid_a);
    let resp = arduino_request(port, "GET", &path, Some((&uuid_b, &token_b)), None);
    assert_eq!(resp.status, 200);

    let json = parse_json(&resp.body);
    assert_eq!(json["uuid"], uuid_a.as_str());
    assert_eq!(json["type"], "target");
    // Token should NOT be in the response (only returned on register)
    assert!(json.get("token").is_none() || json["token"].is_null());
}

// ===========================================================================
// PUT /devices/:uuid — update device properties
// ===========================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn raw_tcp_update_device() {
    let (ws_url, _state) = setup().await;
    let port = port_from_ws_url(&ws_url);

    // Register
    let resp = arduino_request(port, "POST", "/devices", None, Some(r#"{"type":"updatable"}"#));
    let reg = parse_json(&resp.body);
    let uuid = reg["uuid"].as_str().unwrap().to_string();
    let token = reg["token"].as_str().unwrap().to_string();

    // Update properties
    let path = format!("/devices/{}", uuid);
    let update_body = r#"{"color":"blue","temperature":22.5}"#;
    let resp = arduino_request(port, "PUT", &path, Some((&uuid, &token)), Some(update_body));
    assert_eq!(resp.status, 200);

    let json = parse_json(&resp.body);
    assert_eq!(json["uuid"], uuid.as_str());
    assert_eq!(json["color"], "blue");
    assert_eq!(json["temperature"], 22.5);

    // Verify via GET
    let resp = arduino_request(port, "GET", &path, Some((&uuid, &token)), None);
    assert_eq!(resp.status, 200);
    let json = parse_json(&resp.body);
    assert_eq!(json["color"], "blue");
    assert_eq!(json["temperature"], 22.5);
}

// ===========================================================================
// DELETE /devices/:uuid — unregister
// ===========================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn raw_tcp_unregister_device() {
    let (ws_url, _state) = setup().await;
    let port = port_from_ws_url(&ws_url);

    // Register
    let resp = arduino_request(port, "POST", "/devices", None, Some("{}"));
    let reg = parse_json(&resp.body);
    let uuid = reg["uuid"].as_str().unwrap().to_string();
    let token = reg["token"].as_str().unwrap().to_string();

    // Delete
    let path = format!("/devices/{}", uuid);
    let resp = arduino_request(port, "DELETE", &path, Some((&uuid, &token)), None);
    assert_eq!(resp.status, 200);

    // Verify device is gone — GET should fail
    let resp = arduino_request(port, "GET", &path, Some((&uuid, &token)), None);
    assert!(
        resp.status == 401 || resp.status == 404,
        "deleted device should not be accessible, got {}",
        resp.status
    );
}

// ===========================================================================
// POST /messages — send a direct message
// ===========================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn raw_tcp_send_message() {
    let (ws_url, state) = setup().await;
    let port = port_from_ws_url(&ws_url);

    // Register sender via raw TCP (as Arduino would)
    let resp = arduino_request(
        port,
        "POST",
        "/devices",
        None,
        Some(r#"{"type":"sender"}"#),
    );
    assert_eq!(resp.status, 200);
    let reg_s = parse_json(&resp.body);
    let sender_uuid = reg_s["uuid"].as_str().unwrap().to_string();
    let sender_token = reg_s["token"].as_str().unwrap().to_string();

    // Register receiver via store (so we can connect WS to verify delivery)
    let (recv_uuid, recv_token) = register_device(&state).await;

    // Connect receiver via WebSocket to observe delivery
    let mut ws_recv = connect_and_auth(&ws_url, &recv_uuid, &recv_token).await;

    // Send message via raw TCP — exactly as Arduino sendMessage() formats it
    let msg_body = format!(
        r#"{{"devices":["{}"],"payload":{{"alert":"temperature high","value":42}}}}"#,
        recv_uuid
    );
    let resp = arduino_request(
        port,
        "POST",
        "/messages",
        Some((&sender_uuid, &sender_token)),
        Some(&msg_body),
    );
    assert_eq!(resp.status, 200);

    // Verify the receiver got the message via WS
    let msg = recv_json(&mut ws_recv)
        .await
        .expect("receiver should get the message");
    assert_eq!(msg["event"], "message");
    assert_eq!(msg["fromUuid"], sender_uuid.as_str());
    assert_eq!(msg["payload"]["alert"], "temperature high");
    assert_eq!(msg["payload"]["value"], 42);
}

// ===========================================================================
// POST /messages with devices:["*"] — broadcast
// ===========================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn raw_tcp_broadcast() {
    let (ws_url, state) = setup().await;
    let port = port_from_ws_url(&ws_url);

    // Register broadcaster via raw TCP
    let resp = arduino_request(
        port,
        "POST",
        "/devices",
        None,
        Some(r#"{"type":"broadcaster"}"#),
    );
    let reg = parse_json(&resp.body);
    let bc_uuid = reg["uuid"].as_str().unwrap().to_string();
    let bc_token = reg["token"].as_str().unwrap().to_string();

    // Create a subscriber that listens to broadcaster's broadcast.sent
    let (sub_uuid, sub_token) = register_device(&state).await;

    let bc_parsed: uuid::Uuid = bc_uuid.parse().unwrap();
    let sub_parsed: uuid::Uuid = sub_uuid.parse().unwrap();

    use freshblu_core::subscription::{CreateSubscriptionParams, SubscriptionType};
    let params = CreateSubscriptionParams {
        emitter_uuid: bc_parsed,
        subscriber_uuid: sub_parsed,
        subscription_type: SubscriptionType::BroadcastSent,
    };
    state.store.create_subscription(&params).await.unwrap();

    // Connect subscriber via WS
    let mut ws_sub = connect_and_auth(&ws_url, &sub_uuid, &sub_token).await;

    // Broadcast via raw TCP — exactly as Arduino broadcast() formats it
    let msg_body = format!(
        r#"{{"devices":["*"],"payload":{{"temp":72.4,"unit":"F"}}}}"#
    );
    let resp = arduino_request(
        port,
        "POST",
        "/messages",
        Some((&bc_uuid, &bc_token)),
        Some(&msg_body),
    );
    assert_eq!(resp.status, 200);

    // Subscriber should receive the broadcast
    let msg = recv_json(&mut ws_sub)
        .await
        .expect("subscriber should receive broadcast");
    assert_eq!(msg["event"], "broadcast");
    assert_eq!(msg["fromUuid"], bc_uuid.as_str());
    assert_eq!(msg["payload"]["temp"], 72.4);
    assert_eq!(msg["payload"]["unit"], "F");
}

// ===========================================================================
// Auth failures — wrong credentials
// ===========================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn raw_tcp_whoami_no_auth() {
    let (ws_url, _state) = setup().await;
    let port = port_from_ws_url(&ws_url);

    let resp = arduino_request(port, "GET", "/whoami", None, None);
    assert_eq!(resp.status, 401);

    let json = parse_json(&resp.body);
    assert!(json["error"].as_str().is_some());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn raw_tcp_whoami_wrong_token() {
    let (ws_url, _state) = setup().await;
    let port = port_from_ws_url(&ws_url);

    // Register a device
    let resp = arduino_request(port, "POST", "/devices", None, Some("{}"));
    let reg = parse_json(&resp.body);
    let uuid = reg["uuid"].as_str().unwrap().to_string();

    // Use wrong token
    let resp = arduino_request(port, "GET", "/whoami", Some((&uuid, "totally-wrong")), None);
    assert_eq!(resp.status, 401);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn raw_tcp_whoami_nonexistent_uuid() {
    let (ws_url, _state) = setup().await;
    let port = port_from_ws_url(&ws_url);

    let resp = arduino_request(
        port,
        "GET",
        "/whoami",
        Some(("00000000-0000-0000-0000-000000000000", "fake")),
        None,
    );
    assert!(
        resp.status == 401 || resp.status == 404,
        "nonexistent uuid should be rejected, got {}",
        resp.status
    );
}

// ===========================================================================
// GET /devices/:uuid — permission denied (private device)
// ===========================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn raw_tcp_get_private_device_forbidden() {
    let (ws_url, state) = setup().await;
    let port = port_from_ws_url(&ws_url);

    // Register requester via raw TCP
    let resp = arduino_request(port, "POST", "/devices", None, Some("{}"));
    let reg = parse_json(&resp.body);
    let req_uuid = reg["uuid"].as_str().unwrap().to_string();
    let req_token = reg["token"].as_str().unwrap().to_string();

    // Register private device via store
    let (priv_uuid, _priv_token) = register_private_device(&state).await;

    // Try to get private device
    let path = format!("/devices/{}", priv_uuid);
    let resp = arduino_request(port, "GET", &path, Some((&req_uuid, &req_token)), None);
    assert!(
        resp.status == 403 || resp.status == 404,
        "private device should be forbidden, got {}",
        resp.status
    );
}

// ===========================================================================
// Full lifecycle: register → update → whoami → message → unregister
// ===========================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn raw_tcp_full_lifecycle() {
    let (ws_url, state) = setup().await;
    let port = port_from_ws_url(&ws_url);

    // 1. Register device A
    let resp = arduino_request(
        port,
        "POST",
        "/devices",
        None,
        Some(r#"{"type":"lifecycle-test","name":"device-a"}"#),
    );
    assert_eq!(resp.status, 200, "register should succeed");
    let reg = parse_json(&resp.body);
    let uuid_a = reg["uuid"].as_str().unwrap().to_string();
    let token_a = reg["token"].as_str().unwrap().to_string();

    // 2. Verify with whoami
    let resp = arduino_request(port, "GET", "/whoami", Some((&uuid_a, &token_a)), None);
    assert_eq!(resp.status, 200, "whoami should succeed");
    let json = parse_json(&resp.body);
    assert_eq!(json["uuid"], uuid_a.as_str());

    // 3. Update properties
    let path_a = format!("/devices/{}", uuid_a);
    let resp = arduino_request(
        port,
        "PUT",
        &path_a,
        Some((&uuid_a, &token_a)),
        Some(r#"{"firmware":"2.1","lastTemp":22.5}"#),
    );
    assert_eq!(resp.status, 200, "update should succeed");
    let json = parse_json(&resp.body);
    assert_eq!(json["firmware"], "2.1");
    assert_eq!(json["lastTemp"], 22.5);

    // 4. Register device B, connect via WS
    let (uuid_b, token_b) = register_device(&state).await;
    let mut ws_b = connect_and_auth(&ws_url, &uuid_b, &token_b).await;

    // 5. A sends message to B via raw TCP
    let msg_body = format!(
        r#"{{"devices":["{}"],"payload":{{"hello":"from-a"}}}}"#,
        uuid_b
    );
    let resp = arduino_request(
        port,
        "POST",
        "/messages",
        Some((&uuid_a, &token_a)),
        Some(&msg_body),
    );
    assert_eq!(resp.status, 200, "message should succeed");

    // 6. B receives the message
    let msg = recv_json(&mut ws_b)
        .await
        .expect("B should receive message from A");
    assert_eq!(msg["event"], "message");
    assert_eq!(msg["fromUuid"], uuid_a.as_str());
    assert_eq!(msg["payload"]["hello"], "from-a");

    // 7. Verify update persisted via GET
    let resp = arduino_request(
        port,
        "GET",
        &path_a,
        Some((&uuid_a, &token_a)),
        None,
    );
    assert_eq!(resp.status, 200);
    let json = parse_json(&resp.body);
    assert_eq!(json["firmware"], "2.1");

    // 8. Unregister A
    let resp = arduino_request(
        port,
        "DELETE",
        &path_a,
        Some((&uuid_a, &token_a)),
        None,
    );
    assert_eq!(resp.status, 200, "unregister should succeed");

    // 9. Verify A is gone
    let resp = arduino_request(
        port,
        "GET",
        &path_a,
        Some((&uuid_a, &token_a)),
        None,
    );
    assert!(resp.status == 401 || resp.status == 404);
}

// ===========================================================================
// Multiple sequential requests on separate TCP connections
// ===========================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn raw_tcp_sequential_requests() {
    let (ws_url, _state) = setup().await;
    let port = port_from_ws_url(&ws_url);

    // Register
    let resp = arduino_request(port, "POST", "/devices", None, Some(r#"{"type":"seq"}"#));
    assert_eq!(resp.status, 200);
    let reg = parse_json(&resp.body);
    let uuid = reg["uuid"].as_str().unwrap().to_string();
    let token = reg["token"].as_str().unwrap().to_string();

    // Make 5 sequential whoami requests — each opens a new TCP connection
    // (Arduino uses Connection: close, so each request is a new connection)
    for i in 0..5 {
        let resp = arduino_request(port, "GET", "/whoami", Some((&uuid, &token)), None);
        assert_eq!(
            resp.status, 200,
            "sequential request {} should succeed",
            i
        );
        let json = parse_json(&resp.body);
        assert_eq!(json["uuid"], uuid.as_str());
    }

    // Make 3 sequential updates
    for i in 0..3 {
        let path = format!("/devices/{}", uuid);
        let body = format!(r#"{{"counter":{}}}"#, i);
        let resp = arduino_request(
            port,
            "PUT",
            &path,
            Some((&uuid, &token)),
            Some(&body),
        );
        assert_eq!(resp.status, 200, "sequential update {} should succeed", i);
        let json = parse_json(&resp.body);
        assert_eq!(json["counter"], i);
    }
}

// ===========================================================================
// Base64 auth header is accepted by server
// ===========================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn raw_tcp_base64_auth_accepted() {
    let (ws_url, _state) = setup().await;
    let port = port_from_ws_url(&ws_url);

    // Register
    let resp = arduino_request(port, "POST", "/devices", None, Some("{}"));
    let reg = parse_json(&resp.body);
    let uuid = reg["uuid"].as_str().unwrap().to_string();
    let token = reg["token"].as_str().unwrap().to_string();

    // Verify that our Arduino-style Base64 produces the same result as the
    // standard base64 crate (which the server uses to decode)
    let plain = format!("{}:{}", uuid, token);
    let arduino_encoded = arduino_base64(plain.as_bytes());
    let standard_encoded = base64::engine::general_purpose::STANDARD.encode(plain.as_bytes());
    assert_eq!(
        arduino_encoded, standard_encoded,
        "Arduino Base64 must match standard for real credentials"
    );

    // And that the server actually accepts it
    let resp = arduino_request(port, "GET", "/whoami", Some((&uuid, &token)), None);
    assert_eq!(resp.status, 200);
}

// ===========================================================================
// Error response JSON format — Arduino parses {"error":"..."}
// ===========================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn raw_tcp_error_response_format() {
    let (ws_url, _state) = setup().await;
    let port = port_from_ws_url(&ws_url);

    // 401 on bad auth
    let resp = arduino_request(
        port,
        "GET",
        "/whoami",
        Some(("fake-uuid", "fake-token")),
        None,
    );
    assert!(resp.status >= 400);
    let json = parse_json(&resp.body);
    assert!(
        json["error"].is_string(),
        "error response must have 'error' string field, got: {}",
        resp.body
    );

    // 404 on nonexistent device
    let resp = arduino_request(port, "POST", "/devices", None, Some("{}"));
    let reg = parse_json(&resp.body);
    let uuid = reg["uuid"].as_str().unwrap().to_string();
    let token = reg["token"].as_str().unwrap().to_string();

    let resp = arduino_request(
        port,
        "GET",
        "/devices/00000000-0000-0000-0000-000000000000",
        Some((&uuid, &token)),
        None,
    );
    assert!(resp.status >= 400);
    let json = parse_json(&resp.body);
    assert!(
        json["error"].is_string(),
        "error response must have 'error' field"
    );
}

// ===========================================================================
// Status endpoint works even when sending auth (Arduino status() used to
// strip auth, but it shouldn't matter)
// ===========================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn raw_tcp_status_with_auth() {
    let (ws_url, _state) = setup().await;
    let port = port_from_ws_url(&ws_url);

    // Register
    let resp = arduino_request(port, "POST", "/devices", None, Some("{}"));
    let reg = parse_json(&resp.body);
    let uuid = reg["uuid"].as_str().unwrap().to_string();
    let token = reg["token"].as_str().unwrap().to_string();

    // Status with auth — should still work
    let resp = arduino_request(port, "GET", "/status", Some((&uuid, &token)), None);
    assert_eq!(resp.status, 200);
    let json = parse_json(&resp.body);
    assert_eq!(json["meshblu"], true);
}

// ===========================================================================
// Message to private device is silently dropped (Arduino gets 200 but
// message is not delivered)
// ===========================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn raw_tcp_message_to_private_device() {
    let (ws_url, state) = setup().await;
    let port = port_from_ws_url(&ws_url);

    // Register sender via raw TCP
    let resp = arduino_request(port, "POST", "/devices", None, Some("{}"));
    let reg = parse_json(&resp.body);
    let sender_uuid = reg["uuid"].as_str().unwrap().to_string();
    let sender_token = reg["token"].as_str().unwrap().to_string();

    // Private receiver
    let (priv_uuid, priv_token) = register_private_device(&state).await;
    let mut ws_priv = connect_and_auth(&ws_url, &priv_uuid, &priv_token).await;

    // Send message to private device
    let msg_body = format!(
        r#"{{"devices":["{}"],"payload":{{"test":true}}}}"#,
        priv_uuid
    );
    let resp = arduino_request(
        port,
        "POST",
        "/messages",
        Some((&sender_uuid, &sender_token)),
        Some(&msg_body),
    );
    // Server returns 200 even if message is dropped
    assert_eq!(resp.status, 200);

    // Private device should NOT receive the message
    let msg = recv_json(&mut ws_priv).await;
    assert!(msg.is_none(), "private device should not receive message");
}

// ===========================================================================
// Update device — verify system fields (uuid, meshblu) cannot be overwritten
// ===========================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn raw_tcp_update_cannot_overwrite_uuid() {
    let (ws_url, _state) = setup().await;
    let port = port_from_ws_url(&ws_url);

    let resp = arduino_request(
        port,
        "POST",
        "/devices",
        None,
        Some(r#"{"type":"immutable-test"}"#),
    );
    let reg = parse_json(&resp.body);
    let uuid = reg["uuid"].as_str().unwrap().to_string();
    let token = reg["token"].as_str().unwrap().to_string();

    // Try to change the UUID
    let path = format!("/devices/{}", uuid);
    let resp = arduino_request(
        port,
        "PUT",
        &path,
        Some((&uuid, &token)),
        Some(r#"{"uuid":"hacked","color":"red"}"#),
    );
    assert_eq!(resp.status, 200);
    let json = parse_json(&resp.body);
    // UUID should NOT have changed
    assert_eq!(json["uuid"], uuid.as_str());
    // But color should have been set
    assert_eq!(json["color"], "red");
}

// ===========================================================================
// Register response contains all expected fields
// ===========================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn raw_tcp_register_response_structure() {
    let (ws_url, _state) = setup().await;
    let port = port_from_ws_url(&ws_url);

    let resp = arduino_request(
        port,
        "POST",
        "/devices",
        None,
        Some(r#"{"type":"struct-test","name":"my-device"}"#),
    );
    assert_eq!(resp.status, 200);

    let json = parse_json(&resp.body);

    // Required fields for Arduino library
    assert!(json["uuid"].is_string(), "must have uuid");
    assert!(json["token"].is_string(), "must have token");
    assert!(json["meshblu"].is_object(), "must have meshblu object");
    assert!(
        json["meshblu"]["version"].is_string(),
        "must have meshblu.version"
    );

    // User properties preserved
    assert_eq!(json["type"], "struct-test");
    assert_eq!(json["name"], "my-device");
}

// ===========================================================================
// Content-Type and Content-Length headers are correctly processed
// ===========================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn raw_tcp_large_properties() {
    let (ws_url, _state) = setup().await;
    let port = port_from_ws_url(&ws_url);

    // Build a larger property payload (but still within 512 byte Arduino limit)
    let body = r#"{"type":"large","a":"aaaaaaaaaa","b":"bbbbbbbbbb","c":"cccccccccc","d":"dddddddddd","e":"eeeeeeeeee","f":"ffffffffff","g":"gggggggggg","nested":{"x":1,"y":2,"z":3}}"#;
    assert!(
        body.len() < 512,
        "body should fit in Arduino's 512-byte buffer"
    );

    let resp = arduino_request(port, "POST", "/devices", None, Some(body));
    assert_eq!(resp.status, 200);

    let json = parse_json(&resp.body);
    assert_eq!(json["type"], "large");
    assert_eq!(json["nested"]["x"], 1);
    assert_eq!(json["nested"]["y"], 2);
}

// ===========================================================================
// Multiple devices interacting via raw TCP only
// ===========================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn raw_tcp_two_devices_get_each_other() {
    let (ws_url, _state) = setup().await;
    let port = port_from_ws_url(&ws_url);

    // Register A
    let resp = arduino_request(
        port,
        "POST",
        "/devices",
        None,
        Some(r#"{"type":"device-a"}"#),
    );
    let reg_a = parse_json(&resp.body);
    let uuid_a = reg_a["uuid"].as_str().unwrap().to_string();
    let token_a = reg_a["token"].as_str().unwrap().to_string();

    // Register B
    let resp = arduino_request(
        port,
        "POST",
        "/devices",
        None,
        Some(r#"{"type":"device-b"}"#),
    );
    let reg_b = parse_json(&resp.body);
    let uuid_b = reg_b["uuid"].as_str().unwrap().to_string();
    let token_b = reg_b["token"].as_str().unwrap().to_string();

    // A gets B
    let resp = arduino_request(
        port,
        "GET",
        &format!("/devices/{}", uuid_b),
        Some((&uuid_a, &token_a)),
        None,
    );
    assert_eq!(resp.status, 200);
    let json = parse_json(&resp.body);
    assert_eq!(json["uuid"], uuid_b.as_str());
    assert_eq!(json["type"], "device-b");

    // B gets A
    let resp = arduino_request(
        port,
        "GET",
        &format!("/devices/{}", uuid_a),
        Some((&uuid_b, &token_b)),
        None,
    );
    assert_eq!(resp.status, 200);
    let json = parse_json(&resp.body);
    assert_eq!(json["uuid"], uuid_a.as_str());
    assert_eq!(json["type"], "device-a");
}

// ===========================================================================
// Message with topic field
// ===========================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn raw_tcp_message_with_topic() {
    let (ws_url, state) = setup().await;
    let port = port_from_ws_url(&ws_url);

    // Register sender via raw TCP
    let resp = arduino_request(port, "POST", "/devices", None, Some("{}"));
    let reg = parse_json(&resp.body);
    let sender_uuid = reg["uuid"].as_str().unwrap().to_string();
    let sender_token = reg["token"].as_str().unwrap().to_string();

    // Receiver via store + WS
    let (recv_uuid, recv_token) = register_device(&state).await;
    let mut ws_recv = connect_and_auth(&ws_url, &recv_uuid, &recv_token).await;

    // Send message with topic field
    let msg_body = format!(
        r#"{{"devices":["{}"],"topic":"sensor-reading","payload":{{"temp":72.4}}}}"#,
        recv_uuid
    );
    let resp = arduino_request(
        port,
        "POST",
        "/messages",
        Some((&sender_uuid, &sender_token)),
        Some(&msg_body),
    );
    assert_eq!(resp.status, 200);

    let msg = recv_json(&mut ws_recv)
        .await
        .expect("should receive message with topic");
    assert_eq!(msg["event"], "message");
    assert_eq!(msg["payload"]["temp"], 72.4);
}

// ===========================================================================
// Concurrent registrations from different "Arduino devices"
// ===========================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn raw_tcp_concurrent_registrations() {
    let (ws_url, _state) = setup().await;
    let port = port_from_ws_url(&ws_url);

    // Spawn 10 concurrent registrations
    let mut handles = Vec::new();
    for i in 0..10 {
        let body = format!(r#"{{"type":"concurrent","index":{}}}"#, i);
        handles.push(tokio::task::spawn_blocking(move || {
            arduino_request(port, "POST", "/devices", None, Some(&body))
        }));
    }

    let mut uuids = std::collections::HashSet::new();
    for handle in handles {
        let resp = handle.await.unwrap();
        assert_eq!(resp.status, 200, "concurrent register should succeed");
        let json = parse_json(&resp.body);
        let uuid = json["uuid"].as_str().unwrap().to_string();
        assert!(uuids.insert(uuid), "each device should get a unique UUID");
    }

    assert_eq!(uuids.len(), 10);
}

// ===========================================================================
// Update then read-back matches — full round-trip property test
// ===========================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn raw_tcp_property_roundtrip() {
    let (ws_url, _state) = setup().await;
    let port = port_from_ws_url(&ws_url);

    // Register
    let resp = arduino_request(port, "POST", "/devices", None, Some("{}"));
    let reg = parse_json(&resp.body);
    let uuid = reg["uuid"].as_str().unwrap().to_string();
    let token = reg["token"].as_str().unwrap().to_string();
    let path = format!("/devices/{}", uuid);

    // Test various JSON value types that ArduinoJson would produce
    let updates = [
        r#"{"intVal":42}"#,
        r#"{"floatVal":3.14}"#,
        r#"{"boolVal":true}"#,
        r#"{"strVal":"hello world"}"#,
        r#"{"nullVal":null}"#,
        r#"{"arrVal":[1,2,3]}"#,
        r#"{"objVal":{"nested":"value"}}"#,
    ];

    for update in &updates {
        let resp = arduino_request(
            port,
            "PUT",
            &path,
            Some((&uuid, &token)),
            Some(update),
        );
        assert_eq!(resp.status, 200, "update with {} should succeed", update);
    }

    // Final GET should have all properties
    let resp = arduino_request(port, "GET", &path, Some((&uuid, &token)), None);
    assert_eq!(resp.status, 200);
    let json = parse_json(&resp.body);
    assert_eq!(json["intVal"], 42);
    assert!((json["floatVal"].as_f64().unwrap() - 3.14).abs() < f64::EPSILON);
    assert_eq!(json["boolVal"], true);
    assert_eq!(json["strVal"], "hello world");
    assert!(json["nullVal"].is_null());
    assert_eq!(json["arrVal"], serde_json::json!([1, 2, 3]));
    assert_eq!(json["objVal"]["nested"], "value");
}
