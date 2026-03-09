"""
FreshBlu Python SDK
==================
Meshblu-compatible IoT device registry and messaging client.

Usage:
    from freshblu import FreshBluHttp

    client = FreshBluHttp("https://api.freshblu.org")

    # Register
    device = client.register({"type": "temperature-sensor"})
    client.set_credentials(device["uuid"], device["token"])

    # Send message
    client.message({"devices": ["*"], "payload": {"temp": 72.4}})

    # Subscribe
    client.create_subscription(
        subscriber_uuid=device["uuid"],
        emitter_uuid=other_uuid,
        subscription_type="broadcast.sent"
    )
"""

from __future__ import annotations

import asyncio
import base64
import json
import threading
from typing import Any, Callable, Dict, List, Optional, Union
from dataclasses import dataclass, field
from enum import Enum

try:
    import httpx
    HAS_HTTPX = True
except ImportError:
    import urllib.request
    import urllib.error
    HAS_HTTPX = False

try:
    import websockets
    HAS_WS = True
except ImportError:
    HAS_WS = False


class SubscriptionType(str, Enum):
    BROADCAST_SENT = "broadcast.sent"
    BROADCAST_RECEIVED = "broadcast.received"
    CONFIGURE_SENT = "configure.sent"
    CONFIGURE_RECEIVED = "configure.received"
    MESSAGE_SENT = "message.sent"
    MESSAGE_RECEIVED = "message.received"
    UNREGISTER_SENT = "unregister.sent"
    UNREGISTER_RECEIVED = "unregister.received"


class FreshBluError(Exception):
    def __init__(self, message: str, status_code: Optional[int] = None):
        super().__init__(message)
        self.status_code = status_code


def _parse_url(url: str) -> tuple:
    """Parse a URL string into (hostname, port, secure)."""
    from urllib.parse import urlparse
    parsed = urlparse(url)
    secure = parsed.scheme in ("https", "wss")
    hostname = parsed.hostname or "api.freshblu.org"
    default_port = 443 if secure else 3000
    port = parsed.port or default_port
    return hostname, port, secure


@dataclass
class FreshBluConfig:
    hostname: str = "api.freshblu.org"
    port: int = 443
    secure: bool = True
    uuid: Optional[str] = None
    token: Optional[str] = None

    @property
    def base_url(self) -> str:
        scheme = "https" if self.secure else "http"
        if (self.secure and self.port == 443) or (not self.secure and self.port == 80):
            return f"{scheme}://{self.hostname}"
        return f"{scheme}://{self.hostname}:{self.port}"

    @property
    def ws_url(self) -> str:
        scheme = "wss" if self.secure else "ws"
        if (self.secure and self.port == 443) or (not self.secure and self.port == 80):
            return f"{scheme}://{self.hostname}/ws"
        return f"{scheme}://{self.hostname}:{self.port}/ws"

    @property
    def auth_header(self) -> Optional[str]:
        if not self.uuid or not self.token:
            return None
        creds = base64.b64encode(f"{self.uuid}:{self.token}".encode()).decode()
        return f"Basic {creds}"


class FreshBluHttp:
    """Synchronous HTTP client for FreshBlu."""

    def __init__(self, url: Optional[str] = None, config: Optional[FreshBluConfig] = None, **kwargs):
        if url is not None:
            hostname, port, secure = _parse_url(url)
            self.config = FreshBluConfig(hostname=hostname, port=port, secure=secure)
        elif config is not None:
            self.config = config
        else:
            self.config = FreshBluConfig(**kwargs)
        self._client = httpx.Client() if HAS_HTTPX else None

    def set_credentials(self, uuid: str, token: str) -> None:
        self.config.uuid = uuid
        self.config.token = token

    def _headers(self, extra: Optional[Dict] = None) -> Dict[str, str]:
        h = {"Content-Type": "application/json"}
        auth = self.config.auth_header
        if auth:
            h["Authorization"] = auth
        if extra:
            h.update(extra)
        return h

    def _request(
        self,
        method: str,
        path: str,
        body: Optional[Any] = None,
        extra_headers: Optional[Dict] = None,
    ) -> Any:
        url = f"{self.config.base_url}{path}"
        headers = self._headers(extra_headers)

        if HAS_HTTPX:
            resp = self._client.request(
                method, url, json=body, headers=headers
            )
            if not resp.is_success:
                try:
                    err = resp.json().get("error", resp.text)
                except Exception:
                    err = resp.text
                raise FreshBluError(str(err), resp.status_code)
            return resp.json() if resp.content else {}
        else:
            # Fallback: urllib
            import urllib.request as urllib_req
            import urllib.error as urllib_err
            data = json.dumps(body).encode() if body is not None else None
            req = urllib_req.Request(url, data=data, headers=headers, method=method)
            try:
                with urllib_req.urlopen(req) as resp:
                    return json.loads(resp.read().decode())
            except urllib_err.HTTPError as e:
                body = e.read().decode()
                try:
                    err = json.loads(body).get("error", body)
                except Exception:
                    err = body
                raise FreshBluError(str(err), e.code)

    def register(self, properties: Optional[Dict] = None) -> Dict:
        """Register a new device. Returns device with plaintext token."""
        return self._request("POST", "/devices", properties or {})

    def whoami(self) -> Dict:
        """Get authenticated device info."""
        return self._request("GET", "/whoami")

    def get_device(self, uuid: str, as_uuid: Optional[str] = None) -> Dict:
        """Get a device by UUID."""
        headers = {"x-meshblu-as": as_uuid} if as_uuid else None
        return self._request("GET", f"/devices/{uuid}", extra_headers=headers)

    def update_device(self, uuid: str, properties: Dict) -> Dict:
        """Update a device's properties."""
        return self._request("PUT", f"/devices/{uuid}", properties)

    def unregister(self, uuid: str) -> Dict:
        """Unregister a device."""
        return self._request("DELETE", f"/devices/{uuid}")

    def search(self, query: Optional[Dict] = None) -> List[Dict]:
        """Search for devices matching a query."""
        return self._request("POST", "/devices/search", query or {})

    def message(self, msg: Dict) -> Dict:
        """Send a message to one or more devices."""
        return self._request("POST", "/messages", msg)

    def create_subscription(
        self,
        subscriber_uuid: str,
        emitter_uuid: str,
        subscription_type: Union[SubscriptionType, str],
    ) -> Dict:
        """Subscribe to events from an emitter device."""
        return self._request(
            "POST",
            f"/devices/{subscriber_uuid}/subscriptions",
            {
                "emitterUuid": emitter_uuid,
                "subscriberUuid": subscriber_uuid,
                "type": subscription_type.value if isinstance(subscription_type, SubscriptionType) else str(subscription_type),
            },
        )

    def delete_subscription(
        self,
        subscriber_uuid: str,
        emitter_uuid: str,
        subscription_type: Union[SubscriptionType, str],
    ) -> None:
        """Delete a subscription."""
        type_val = subscription_type.value if isinstance(subscription_type, SubscriptionType) else str(subscription_type)
        type_str = type_val.replace(".", "-")
        self._request(
            "DELETE",
            f"/devices/{subscriber_uuid}/subscriptions/{emitter_uuid}/{type_str}",
        )

    def subscriptions(self, subscriber_uuid: str) -> List[Dict]:
        """List all subscriptions for a device."""
        return self._request("GET", f"/devices/{subscriber_uuid}/subscriptions")

    def generate_token(
        self,
        uuid: str,
        expires_on: Optional[int] = None,
        tag: Optional[str] = None,
    ) -> Dict:
        """Generate a new token for a device."""
        opts: Dict[str, Any] = {}
        if expires_on is not None:
            opts["expiresOn"] = expires_on
        if tag is not None:
            opts["tag"] = tag
        return self._request("POST", f"/devices/{uuid}/tokens", opts)

    def revoke_token(self, uuid: str, token: str) -> None:
        """Revoke a token."""
        self._request("DELETE", f"/devices/{uuid}/tokens/{token}")

    def my_devices(self) -> List[Dict]:
        """Get devices owned by the authenticated device."""
        return self._request("GET", "/mydevices")

    def claim_device(self, uuid: str) -> Dict:
        """Claim an unclaimed device."""
        return self._request("PUT", f"/claimdevice/{uuid}")

    def broadcast(self, payload: Dict) -> Dict:
        """Broadcast a message to all subscribers."""
        return self._request("POST", "/broadcasts", payload)

    def reset_token(self, uuid: str) -> Dict:
        """Reset all tokens for a device, returning a new one."""
        return self._request("POST", f"/devices/{uuid}/token")

    def status(self) -> Dict:
        """Get server status."""
        return self._request("GET", "/status")

    def __enter__(self):
        return self

    def __exit__(self, *args):
        if self._client:
            self._client.close()


class FreshBlu(FreshBluHttp):
    """
    Full FreshBlu client with WebSocket support for real-time messaging.
    Requires: pip install freshblu[ws]  (installs websockets)
    """

    def __init__(self, url: Optional[str] = None, **kwargs):
        super().__init__(url=url, **kwargs)
        self._ws = None
        self._ws_thread: Optional[threading.Thread] = None
        self._listeners: Dict[str, List[Callable]] = {}
        self._loop: Optional[asyncio.AbstractEventLoop] = None

    def on(self, event: str, callback: Callable) -> "FreshBlu":
        """Register an event listener."""
        if event not in self._listeners:
            self._listeners[event] = []
        self._listeners[event].append(callback)
        return self

    def _emit(self, event: str, data: Any) -> None:
        for cb in self._listeners.get(event, []):
            try:
                cb(data)
            except Exception as e:
                print(f"FreshBlu listener error: {e}")

    def connect(self, callback: Optional[Callable] = None) -> None:
        """Connect to the WebSocket server."""
        if not HAS_WS:
            raise FreshBluError(
                "WebSocket support requires: pip install websockets"
            )

        def run():
            loop = asyncio.new_event_loop()
            self._loop = loop
            loop.run_until_complete(self._ws_connect(callback))

        self._ws_thread = threading.Thread(target=run, daemon=True)
        self._ws_thread.start()

    async def _ws_connect(self, callback: Optional[Callable]):
        async with websockets.connect(self.config.ws_url) as ws:
            self._ws = ws

            # Authenticate
            await ws.send(json.dumps({
                "event": "identity",
                "uuid": self.config.uuid,
                "token": self.config.token,
            }))

            async for raw in ws:
                try:
                    data = json.loads(raw)
                    event = data.get("event")
                    if event:
                        self._emit(event, data)
                        if event == "ready" and callback:
                            callback()
                except json.JSONDecodeError:
                    pass

    def send_message(self, msg: Dict) -> None:
        """Send a message via WebSocket."""
        self._ws_send({"event": "message", **msg})

    def subscribe_ws(self, emitter_uuid: str, sub_type: str) -> None:
        """Subscribe to events via WebSocket."""
        self._ws_send({
            "event": "subscribe",
            "emitterUuid": emitter_uuid,
            "type": sub_type,
        })

    def _ws_send(self, data: Dict) -> None:
        if not self._ws or not self._loop:
            raise FreshBluError("Not connected")
        asyncio.run_coroutine_threadsafe(
            self._ws.send(json.dumps(data)), self._loop
        )

    def disconnect(self) -> None:
        if self._ws:
            asyncio.run_coroutine_threadsafe(self._ws.close(), self._loop)


# ---- Async client ----

class AsyncFreshBlu:
    """Async version - use with asyncio."""

    def __init__(self, url: Optional[str] = None, **kwargs):
        if url is not None:
            hostname, port, secure = _parse_url(url)
            self.config = FreshBluConfig(hostname=hostname, port=port, secure=secure)
        else:
            self.config = FreshBluConfig(**kwargs)
        self._client = httpx.AsyncClient() if HAS_HTTPX else None

    def set_credentials(self, uuid: str, token: str) -> None:
        self.config.uuid = uuid
        self.config.token = token

    def _headers(self) -> Dict[str, str]:
        h = {"Content-Type": "application/json"}
        auth = self.config.auth_header
        if auth:
            h["Authorization"] = auth
        return h

    async def _request(self, method: str, path: str, body: Optional[Any] = None) -> Any:
        if not HAS_HTTPX:
            raise FreshBluError("Async client requires: pip install httpx")
        url = f"{self.config.base_url}{path}"
        resp = await self._client.request(method, url, json=body, headers=self._headers())
        if not resp.is_success:
            err = resp.json().get("error", resp.text)
            raise FreshBluError(str(err), resp.status_code)
        return resp.json() if resp.content else {}

    async def register(self, properties: Optional[Dict] = None) -> Dict:
        return await self._request("POST", "/devices", properties or {})

    async def whoami(self) -> Dict:
        return await self._request("GET", "/whoami")

    async def message(self, msg: Dict) -> Dict:
        return await self._request("POST", "/messages", msg)

    async def search(self, query: Optional[Dict] = None) -> List[Dict]:
        return await self._request("POST", "/devices/search", query or {})

    async def status(self) -> Dict:
        return await self._request("GET", "/status")

    async def close(self) -> None:
        if self._client:
            await self._client.aclose()

    async def __aenter__(self):
        return self

    async def __aexit__(self, *args):
        await self.close()
