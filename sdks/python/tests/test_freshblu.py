"""Tests for FreshBlu Python SDK."""
import base64
import json
import sys
import unittest
from unittest.mock import MagicMock, patch, call

# Mock httpx before importing freshblu so HAS_HTTPX is True
mock_httpx = MagicMock()
sys.modules["httpx"] = mock_httpx

import freshblu as freshblu_mod
from freshblu import (
    FreshBluConfig,
    FreshBluHttp,
    FreshBlu,
    FreshBluError,
    SubscriptionType,
    _parse_url,
)


# ---------------------------------------------------------------------------
# URL parsing
# ---------------------------------------------------------------------------
class TestParseUrl(unittest.TestCase):
    def test_https_default_port(self):
        h, p, s = _parse_url("https://api.freshblu.org")
        self.assertEqual(h, "api.freshblu.org")
        self.assertEqual(p, 443)
        self.assertTrue(s)

    def test_http_explicit_port(self):
        h, p, s = _parse_url("http://localhost:3000")
        self.assertEqual(h, "localhost")
        self.assertEqual(p, 3000)
        self.assertFalse(s)

    def test_http_no_port_defaults_to_3000(self):
        h, p, s = _parse_url("http://myhost.local")
        self.assertEqual(h, "myhost.local")
        self.assertEqual(p, 3000)
        self.assertFalse(s)

    def test_wss_treated_as_secure(self):
        h, p, s = _parse_url("wss://example.com")
        self.assertTrue(s)
        self.assertEqual(p, 443)

    def test_ws_treated_as_insecure(self):
        h, p, s = _parse_url("ws://example.com:8080")
        self.assertFalse(s)
        self.assertEqual(p, 8080)

    def test_missing_hostname_falls_back(self):
        # urlparse("://") gives hostname=None
        h, p, s = _parse_url("https://")
        self.assertEqual(h, "api.freshblu.org")


# ---------------------------------------------------------------------------
# Config dataclass
# ---------------------------------------------------------------------------
class TestConfig(unittest.TestCase):
    def test_secure_defaults(self):
        c = FreshBluConfig()
        self.assertEqual(c.base_url, "https://api.freshblu.org")
        self.assertEqual(c.ws_url, "wss://api.freshblu.org/ws")

    def test_insecure_custom_port_includes_port(self):
        c = FreshBluConfig(hostname="localhost", port=3000, secure=False)
        self.assertEqual(c.base_url, "http://localhost:3000")
        self.assertEqual(c.ws_url, "ws://localhost:3000/ws")

    def test_port_80_http_omits_port(self):
        c = FreshBluConfig(hostname="example.com", port=80, secure=False)
        self.assertEqual(c.base_url, "http://example.com")
        self.assertEqual(c.ws_url, "ws://example.com/ws")

    def test_port_443_https_omits_port(self):
        c = FreshBluConfig(hostname="example.com", port=443, secure=True)
        self.assertEqual(c.base_url, "https://example.com")

    def test_nonstandard_https_port_includes_port(self):
        c = FreshBluConfig(hostname="example.com", port=8443, secure=True)
        self.assertEqual(c.base_url, "https://example.com:8443")
        self.assertEqual(c.ws_url, "wss://example.com:8443/ws")

    def test_auth_header_encodes_correctly(self):
        c = FreshBluConfig(uuid="dev-001", token="secret-tok")
        raw = base64.b64decode(c.auth_header.split(" ")[1]).decode()
        self.assertEqual(raw, "dev-001:secret-tok")
        self.assertTrue(c.auth_header.startswith("Basic "))

    def test_auth_header_none_without_credentials(self):
        self.assertIsNone(FreshBluConfig().auth_header)

    def test_auth_header_none_with_partial_credentials(self):
        self.assertIsNone(FreshBluConfig(uuid="x").auth_header)
        self.assertIsNone(FreshBluConfig(token="y").auth_header)


# ---------------------------------------------------------------------------
# Helper: builds a client with a mocked httpx transport
# ---------------------------------------------------------------------------
def _client(**kwargs) -> FreshBluHttp:
    c = FreshBluHttp(**kwargs)
    c._client = MagicMock()
    return c


def _ok(client, json_data, status=200):
    """Wire up the mock to return a successful response."""
    resp = MagicMock()
    resp.is_success = 200 <= status < 300
    resp.status_code = status
    resp.json.return_value = json_data
    resp.content = json.dumps(json_data).encode() if json_data is not None else b""
    resp.text = json.dumps(json_data) if json_data is not None else ""
    client._client.request.return_value = resp
    return resp


def _last_call(client):
    """Return (method, url, json_body, headers) of the last _client.request call."""
    c = client._client.request.call_args
    return c.args[0], c.args[1], c.kwargs.get("json"), c.kwargs.get("headers", {})


# ---------------------------------------------------------------------------
# Constructor variants
# ---------------------------------------------------------------------------
class TestConstructors(unittest.TestCase):
    def test_url_string(self):
        c = FreshBluHttp("https://api.freshblu.org")
        self.assertEqual(c.config.hostname, "api.freshblu.org")
        self.assertTrue(c.config.secure)
        self.assertEqual(c.config.port, 443)

    def test_url_string_http(self):
        c = FreshBluHttp("http://10.0.0.5:3000")
        self.assertEqual(c.config.hostname, "10.0.0.5")
        self.assertFalse(c.config.secure)
        self.assertEqual(c.config.port, 3000)

    def test_kwargs(self):
        c = _client(hostname="myhost", port=9000, secure=False)
        self.assertEqual(c.config.hostname, "myhost")
        self.assertEqual(c.config.port, 9000)

    def test_config_object(self):
        cfg = FreshBluConfig(hostname="a.b.c", port=1234, secure=True)
        c = FreshBluHttp(config=cfg)
        self.assertIs(c.config, cfg)

    def test_default_constructor(self):
        c = FreshBluHttp()
        self.assertEqual(c.config.hostname, "api.freshblu.org")


# ---------------------------------------------------------------------------
# Device CRUD — verify correct HTTP method, URL, body, headers
# ---------------------------------------------------------------------------
class TestDeviceCrud(unittest.TestCase):
    def setUp(self):
        self.c = _client(hostname="localhost", port=3000, secure=False)
        self.c.set_credentials("my-uuid", "my-token")

    def test_register_sends_post_to_devices(self):
        _ok(self.c, {"uuid": "new-uuid", "token": "new-tok"})
        result = self.c.register({"type": "sensor", "name": "temp-01"})

        method, url, body, headers = _last_call(self.c)
        self.assertEqual(method, "POST")
        self.assertEqual(url, "http://localhost:3000/devices")
        self.assertEqual(body, {"type": "sensor", "name": "temp-01"})
        self.assertEqual(result["uuid"], "new-uuid")
        self.assertEqual(result["token"], "new-tok")

    def test_register_empty_body(self):
        _ok(self.c, {"uuid": "x"})
        self.c.register()
        _, _, body, _ = _last_call(self.c)
        self.assertEqual(body, {})

    def test_get_device(self):
        _ok(self.c, {"uuid": "dev-1", "type": "gateway"})
        result = self.c.get_device("dev-1")

        method, url, body, headers = _last_call(self.c)
        self.assertEqual(method, "GET")
        self.assertEqual(url, "http://localhost:3000/devices/dev-1")
        self.assertIsNone(body)
        self.assertNotIn("x-meshblu-as", headers)
        self.assertEqual(result["type"], "gateway")

    def test_get_device_as_another(self):
        _ok(self.c, {"uuid": "dev-1"})
        self.c.get_device("dev-1", as_uuid="proxy-uuid")

        _, _, _, headers = _last_call(self.c)
        self.assertEqual(headers["x-meshblu-as"], "proxy-uuid")

    def test_update_device(self):
        _ok(self.c, {"uuid": "dev-1", "name": "updated"})
        self.c.update_device("dev-1", {"name": "updated", "firmware": "2.0"})

        method, url, body, _ = _last_call(self.c)
        self.assertEqual(method, "PUT")
        self.assertEqual(url, "http://localhost:3000/devices/dev-1")
        self.assertEqual(body["firmware"], "2.0")

    def test_unregister(self):
        _ok(self.c, {"uuid": "dev-1"})
        self.c.unregister("dev-1")

        method, url, _, _ = _last_call(self.c)
        self.assertEqual(method, "DELETE")
        self.assertEqual(url, "http://localhost:3000/devices/dev-1")

    def test_whoami(self):
        _ok(self.c, {"uuid": "my-uuid", "online": True})
        result = self.c.whoami()

        method, url, _, _ = _last_call(self.c)
        self.assertEqual(method, "GET")
        self.assertIn("/whoami", url)
        self.assertTrue(result["online"])

    def test_search(self):
        _ok(self.c, [{"uuid": "a"}, {"uuid": "b"}])
        result = self.c.search({"type": "sensor"})

        method, url, body, _ = _last_call(self.c)
        self.assertEqual(method, "POST")
        self.assertIn("/devices/search", url)
        self.assertEqual(body, {"type": "sensor"})
        self.assertEqual(len(result), 2)

    def test_search_no_query(self):
        _ok(self.c, [])
        self.c.search()
        _, _, body, _ = _last_call(self.c)
        self.assertEqual(body, {})

    def test_my_devices(self):
        _ok(self.c, [{"uuid": "d1"}, {"uuid": "d2"}])
        result = self.c.my_devices()

        method, url, _, _ = _last_call(self.c)
        self.assertEqual(method, "GET")
        self.assertIn("/mydevices", url)
        self.assertEqual(len(result), 2)

    def test_claim_device(self):
        _ok(self.c, {"uuid": "unclaimed-1"})
        self.c.claim_device("unclaimed-1")

        method, url, _, _ = _last_call(self.c)
        self.assertEqual(method, "PUT")
        self.assertIn("/claimdevice/unclaimed-1", url)

    def test_status(self):
        _ok(self.c, {"meshblu": "online"})
        result = self.c.status()

        method, url, _, _ = _last_call(self.c)
        self.assertEqual(method, "GET")
        self.assertIn("/status", url)
        self.assertEqual(result["meshblu"], "online")


# ---------------------------------------------------------------------------
# Messaging
# ---------------------------------------------------------------------------
class TestMessaging(unittest.TestCase):
    def setUp(self):
        self.c = _client(hostname="localhost", port=3000, secure=False)
        self.c.set_credentials("sender", "tok")

    def test_message_sends_post_with_devices_and_payload(self):
        _ok(self.c, {"sent": True})
        msg = {"devices": ["target-a", "target-b"], "payload": {"temp": 22.5}}
        result = self.c.message(msg)

        method, url, body, _ = _last_call(self.c)
        self.assertEqual(method, "POST")
        self.assertIn("/messages", url)
        self.assertEqual(body["devices"], ["target-a", "target-b"])
        self.assertEqual(body["payload"]["temp"], 22.5)
        self.assertTrue(result["sent"])

    def test_broadcast(self):
        _ok(self.c, {"sent": True})
        self.c.broadcast({"payload": {"status": "alive"}})

        method, url, body, _ = _last_call(self.c)
        self.assertEqual(method, "POST")
        self.assertIn("/broadcasts", url)
        self.assertEqual(body["payload"]["status"], "alive")


# ---------------------------------------------------------------------------
# Subscriptions — verify URL path construction and body format
# ---------------------------------------------------------------------------
class TestSubscriptions(unittest.TestCase):
    def setUp(self):
        self.c = _client(hostname="localhost", port=3000, secure=False)
        self.c.set_credentials("sub-uuid", "tok")

    def test_create_subscription_string_type(self):
        _ok(self.c, {"subscriptionType": "broadcast-sent"})
        self.c.create_subscription(
            subscriber_uuid="sub-uuid",
            emitter_uuid="emitter-uuid",
            subscription_type="broadcast.sent",
        )

        method, url, body, _ = _last_call(self.c)
        self.assertEqual(method, "POST")
        self.assertIn("/devices/sub-uuid/subscriptions", url)
        self.assertEqual(body["emitterUuid"], "emitter-uuid")
        self.assertEqual(body["subscriberUuid"], "sub-uuid")
        self.assertEqual(body["type"], "broadcast.sent")

    def test_create_subscription_enum_type(self):
        _ok(self.c, {})
        self.c.create_subscription(
            subscriber_uuid="sub",
            emitter_uuid="emit",
            subscription_type=SubscriptionType.MESSAGE_RECEIVED,
        )
        _, _, body, _ = _last_call(self.c)
        self.assertEqual(body["type"], "message.received")

    def test_delete_subscription_converts_dots_to_hyphens(self):
        _ok(self.c, {})
        self.c.delete_subscription(
            subscriber_uuid="sub",
            emitter_uuid="emit",
            subscription_type="configure.sent",
        )
        method, url, _, _ = _last_call(self.c)
        self.assertEqual(method, "DELETE")
        self.assertIn("/devices/sub/subscriptions/emit/configure-sent", url)

    def test_delete_subscription_with_enum(self):
        _ok(self.c, {})
        self.c.delete_subscription(
            subscriber_uuid="s",
            emitter_uuid="e",
            subscription_type=SubscriptionType.UNREGISTER_SENT,
        )
        _, url, _, _ = _last_call(self.c)
        self.assertIn("/unregister-sent", url)

    def test_list_subscriptions(self):
        subs = [
            {"emitterUuid": "a", "type": "broadcast.sent"},
            {"emitterUuid": "b", "type": "message.received"},
        ]
        _ok(self.c, subs)
        result = self.c.subscriptions("sub-uuid")

        method, url, _, _ = _last_call(self.c)
        self.assertEqual(method, "GET")
        self.assertIn("/devices/sub-uuid/subscriptions", url)
        self.assertEqual(len(result), 2)


# ---------------------------------------------------------------------------
# Token management
# ---------------------------------------------------------------------------
class TestTokens(unittest.TestCase):
    def setUp(self):
        self.c = _client(hostname="localhost", port=3000, secure=False)
        self.c.set_credentials("owner", "tok")

    def test_generate_token_minimal(self):
        _ok(self.c, {"token": "new-tok-123"})
        result = self.c.generate_token("dev-1")

        method, url, body, _ = _last_call(self.c)
        self.assertEqual(method, "POST")
        self.assertIn("/devices/dev-1/tokens", url)
        self.assertEqual(body, {})
        self.assertEqual(result["token"], "new-tok-123")

    def test_generate_token_with_options(self):
        _ok(self.c, {"token": "t"})
        self.c.generate_token("dev-1", expires_on=1700000000, tag="ci-runner")

        _, _, body, _ = _last_call(self.c)
        self.assertEqual(body["expiresOn"], 1700000000)
        self.assertEqual(body["tag"], "ci-runner")

    def test_generate_token_tag_only(self):
        _ok(self.c, {"token": "t"})
        self.c.generate_token("dev-1", tag="session-a")
        _, _, body, _ = _last_call(self.c)
        self.assertNotIn("expiresOn", body)
        self.assertEqual(body["tag"], "session-a")

    def test_revoke_token(self):
        _ok(self.c, {})
        self.c.revoke_token("dev-1", "tok-to-revoke")

        method, url, _, _ = _last_call(self.c)
        self.assertEqual(method, "DELETE")
        self.assertIn("/devices/dev-1/tokens/tok-to-revoke", url)

    def test_reset_token(self):
        _ok(self.c, {"uuid": "dev-1", "token": "fresh-tok"})
        result = self.c.reset_token("dev-1")

        method, url, _, _ = _last_call(self.c)
        self.assertEqual(method, "POST")
        self.assertIn("/devices/dev-1/token", url)
        # /token not /tokens
        self.assertNotIn("/tokens", url)
        self.assertEqual(result["token"], "fresh-tok")


# ---------------------------------------------------------------------------
# Auth header propagation
# ---------------------------------------------------------------------------
class TestAuthHeaders(unittest.TestCase):
    def test_unauthenticated_request_has_no_auth(self):
        c = _client()
        _ok(c, {"uuid": "x"})
        c.register()

        _, _, _, headers = _last_call(c)
        self.assertNotIn("Authorization", headers)

    def test_authenticated_request_includes_basic_auth(self):
        c = _client()
        c.set_credentials("u", "t")
        _ok(c, {})
        c.whoami()

        _, _, _, headers = _last_call(c)
        self.assertIn("Authorization", headers)
        decoded = base64.b64decode(headers["Authorization"].split(" ")[1]).decode()
        self.assertEqual(decoded, "u:t")

    def test_credentials_update_between_calls(self):
        c = _client()
        c.set_credentials("a", "1")
        _ok(c, {})
        c.whoami()
        _, _, _, h1 = _last_call(c)

        c.set_credentials("b", "2")
        _ok(c, {})
        c.whoami()
        _, _, _, h2 = _last_call(c)

        cred1 = base64.b64decode(h1["Authorization"].split(" ")[1]).decode()
        cred2 = base64.b64decode(h2["Authorization"].split(" ")[1]).decode()
        self.assertEqual(cred1, "a:1")
        self.assertEqual(cred2, "b:2")

    def test_extra_headers_merged(self):
        c = _client()
        c.set_credentials("u", "t")
        headers = c._headers(extra={"x-meshblu-as": "proxy-dev"})
        self.assertEqual(headers["x-meshblu-as"], "proxy-dev")
        self.assertIn("Authorization", headers)
        self.assertEqual(headers["Content-Type"], "application/json")


# ---------------------------------------------------------------------------
# Error handling
# ---------------------------------------------------------------------------
class TestErrorHandling(unittest.TestCase):
    def test_404_raises_with_status_code(self):
        c = _client()
        _ok(c, {"error": "device not found"}, status=404)
        with self.assertRaises(FreshBluError) as ctx:
            c.get_device("nonexistent")
        self.assertEqual(ctx.exception.status_code, 404)
        self.assertIn("device not found", str(ctx.exception))

    def test_403_forbidden(self):
        c = _client()
        c.set_credentials("u", "t")
        _ok(c, {"error": "forbidden"}, status=403)
        with self.assertRaises(FreshBluError) as ctx:
            c.update_device("x", {"name": "nope"})
        self.assertEqual(ctx.exception.status_code, 403)

    def test_500_server_error(self):
        c = _client()
        _ok(c, {"error": "internal server error"}, status=500)
        with self.assertRaises(FreshBluError) as ctx:
            c.status()
        self.assertEqual(ctx.exception.status_code, 500)

    def test_error_with_non_json_body_uses_text(self):
        c = _client()
        resp = MagicMock()
        resp.is_success = False
        resp.status_code = 502
        resp.json.side_effect = ValueError("not json")
        resp.text = "Bad Gateway"
        c._client.request.return_value = resp

        with self.assertRaises(FreshBluError) as ctx:
            c.status()
        self.assertEqual(ctx.exception.status_code, 502)
        self.assertIn("Bad Gateway", str(ctx.exception))

    def test_empty_response_body_returns_empty_dict(self):
        c = _client()
        resp = MagicMock()
        resp.is_success = True
        resp.status_code = 204
        resp.content = b""
        c._client.request.return_value = resp

        result = c.unregister("some-uuid")
        self.assertEqual(result, {})


# ---------------------------------------------------------------------------
# Context manager
# ---------------------------------------------------------------------------
class TestContextManager(unittest.TestCase):
    def test_enter_returns_self(self):
        c = _client()
        self.assertIs(c.__enter__(), c)

    def test_exit_closes_client(self):
        c = _client()
        mock_http = c._client
        c.__exit__(None, None, None)
        mock_http.close.assert_called_once()

    def test_with_statement(self):
        with _client() as c:
            _ok(c, {"uuid": "x"})
            c.register()
        c._client.close.assert_called_once()


# ---------------------------------------------------------------------------
# FreshBlu WS client — event system (no real WS connection)
# ---------------------------------------------------------------------------
class TestFreshBluEventSystem(unittest.TestCase):
    def _ws_client(self):
        c = FreshBlu(hostname="localhost", port=3000, secure=False)
        c._client = MagicMock()
        return c

    def test_on_returns_self_for_chaining(self):
        c = self._ws_client()
        result = c.on("message", lambda d: None)
        self.assertIs(result, c)

    def test_on_registers_callback(self):
        c = self._ws_client()
        cb = MagicMock()
        c.on("message", cb)
        self.assertIn(cb, c._listeners["message"])

    def test_multiple_listeners_same_event(self):
        c = self._ws_client()
        cb1 = MagicMock()
        cb2 = MagicMock()
        c.on("message", cb1).on("message", cb2)

        c._emit("message", {"payload": "test"})
        cb1.assert_called_once_with({"payload": "test"})
        cb2.assert_called_once_with({"payload": "test"})

    def test_emit_different_events_isolated(self):
        c = self._ws_client()
        msg_cb = MagicMock()
        ready_cb = MagicMock()
        c.on("message", msg_cb)
        c.on("ready", ready_cb)

        c._emit("message", {"payload": "x"})
        msg_cb.assert_called_once()
        ready_cb.assert_not_called()

    def test_emit_unknown_event_does_nothing(self):
        c = self._ws_client()
        c._emit("nonexistent", {})  # should not raise

    def test_listener_exception_does_not_break_other_listeners(self):
        c = self._ws_client()
        bad_cb = MagicMock(side_effect=RuntimeError("boom"))
        good_cb = MagicMock()
        c.on("message", bad_cb)
        c.on("message", good_cb)

        c._emit("message", {"data": 1})
        bad_cb.assert_called_once()
        good_cb.assert_called_once()

    def test_send_message_without_connection_raises(self):
        c = self._ws_client()
        with self.assertRaises(FreshBluError) as ctx:
            c.send_message({"devices": ["x"], "payload": {}})
        self.assertIn("Not connected", str(ctx.exception))

    def test_ws_send_without_connection_raises(self):
        c = self._ws_client()
        with self.assertRaises(FreshBluError):
            c._ws_send({"event": "test"})

    def test_connect_without_websockets_raises(self):
        with patch.object(freshblu_mod, "HAS_WS", False):
            c = self._ws_client()
            with self.assertRaises(FreshBluError) as ctx:
                c.connect()
            self.assertIn("websockets", str(ctx.exception))

    def test_ws_client_url_constructor(self):
        c = FreshBlu("https://api.freshblu.org")
        self.assertEqual(c.config.hostname, "api.freshblu.org")
        self.assertTrue(c.config.secure)

    def test_ws_client_inherits_http_methods(self):
        c = self._ws_client()
        c.set_credentials("u", "t")
        _ok(c, {"uuid": "u"})
        result = c.whoami()
        self.assertEqual(result["uuid"], "u")

    def test_disconnect_without_connection_is_safe(self):
        c = self._ws_client()
        c.disconnect()  # should not raise


# ---------------------------------------------------------------------------
# SubscriptionType enum
# ---------------------------------------------------------------------------
class TestSubscriptionType(unittest.TestCase):
    def test_all_eight_types_exist(self):
        self.assertEqual(len(SubscriptionType), 8)

    def test_enum_values(self):
        self.assertEqual(SubscriptionType.BROADCAST_SENT.value, "broadcast.sent")
        self.assertEqual(SubscriptionType.MESSAGE_RECEIVED.value, "message.received")
        self.assertEqual(SubscriptionType.CONFIGURE_SENT.value, "configure.sent")
        self.assertEqual(SubscriptionType.UNREGISTER_RECEIVED.value, "unregister.received")

    def test_enum_is_str_subclass(self):
        # Important: SubscriptionType values can be used directly as strings
        self.assertIsInstance(SubscriptionType.BROADCAST_SENT, str)


# ---------------------------------------------------------------------------
# FreshBluError
# ---------------------------------------------------------------------------
class TestFreshBluError(unittest.TestCase):
    def test_message_and_status(self):
        e = FreshBluError("something broke", 422)
        self.assertEqual(str(e), "something broke")
        self.assertEqual(e.status_code, 422)

    def test_no_status_code(self):
        e = FreshBluError("generic error")
        self.assertIsNone(e.status_code)

    def test_is_exception(self):
        self.assertTrue(issubclass(FreshBluError, Exception))


# ---------------------------------------------------------------------------
# Integration-style: register → set creds → message flow
# ---------------------------------------------------------------------------
class TestRegisterAndMessageFlow(unittest.TestCase):
    """Simulates the typical SDK usage pattern end-to-end through mocks."""

    def test_full_flow(self):
        c = _client(hostname="localhost", port=3000, secure=False)

        # Step 1: Register
        _ok(c, {"uuid": "dev-abc", "token": "tok-xyz", "online": False})
        device = c.register({"type": "sensor", "name": "temp-01"})
        self.assertEqual(device["uuid"], "dev-abc")

        # Step 2: Set credentials from registration response
        c.set_credentials(device["uuid"], device["token"])

        # Verify auth is now present
        self.assertIsNotNone(c.config.auth_header)

        # Step 3: Send a message
        _ok(c, {"sent": True})
        result = c.message({
            "devices": ["gateway-1"],
            "payload": {"temp": 22.5, "unit": "C"},
        })
        self.assertTrue(result["sent"])

        method, url, body, headers = _last_call(c)
        self.assertEqual(method, "POST")
        self.assertIn("/messages", url)
        self.assertEqual(body["devices"], ["gateway-1"])
        self.assertEqual(body["payload"]["temp"], 22.5)
        # Auth header should contain our registered credentials
        decoded = base64.b64decode(headers["Authorization"].split(" ")[1]).decode()
        self.assertEqual(decoded, "dev-abc:tok-xyz")

    def test_register_subscribe_flow(self):
        c = _client(hostname="localhost", port=3000, secure=False)

        # Register two devices
        _ok(c, {"uuid": "sensor-1", "token": "tok-1"})
        sensor = c.register({"type": "sensor"})

        _ok(c, {"uuid": "monitor-1", "token": "tok-2"})
        monitor = c.register({"type": "monitor"})

        # Authenticate as monitor
        c.set_credentials(monitor["uuid"], monitor["token"])

        # Subscribe to sensor's broadcasts
        _ok(c, {"subscriptionType": "broadcast-sent"})
        c.create_subscription(
            subscriber_uuid=monitor["uuid"],
            emitter_uuid=sensor["uuid"],
            subscription_type=SubscriptionType.BROADCAST_SENT,
        )

        method, url, body, _ = _last_call(c)
        self.assertEqual(method, "POST")
        self.assertIn(f"/devices/{monitor['uuid']}/subscriptions", url)
        self.assertEqual(body["emitterUuid"], sensor["uuid"])
        self.assertEqual(body["type"], "broadcast.sent")

        # List subscriptions
        _ok(c, [{"emitterUuid": "sensor-1", "type": "broadcast.sent"}])
        subs = c.subscriptions(monitor["uuid"])
        self.assertEqual(len(subs), 1)
        self.assertEqual(subs[0]["emitterUuid"], "sensor-1")

    def test_token_lifecycle(self):
        c = _client(hostname="localhost", port=3000, secure=False)
        c.set_credentials("owner-uuid", "owner-tok")

        # Generate a tagged token
        _ok(c, {"token": "new-session-tok"})
        result = c.generate_token("dev-1", tag="ci")
        self.assertEqual(result["token"], "new-session-tok")
        _, _, body, _ = _last_call(c)
        self.assertEqual(body["tag"], "ci")

        # Revoke it
        _ok(c, {})
        c.revoke_token("dev-1", "new-session-tok")
        method, url, _, _ = _last_call(c)
        self.assertEqual(method, "DELETE")
        self.assertIn("new-session-tok", url)

        # Reset all tokens
        _ok(c, {"uuid": "dev-1", "token": "completely-new-tok"})
        result = c.reset_token("dev-1")
        self.assertEqual(result["token"], "completely-new-tok")


if __name__ == "__main__":
    unittest.main()
