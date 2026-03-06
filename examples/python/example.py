"""
FreshBlu Python Example
======================
Requires: pip install freshblu[all]
Run:      python examples/python/example.py
"""

import time
import threading
from freshblu import FreshBlu, FreshBluHttp, SubscriptionType, AsyncFreshBlu

SERVER = "localhost"
PORT = 3000


def sync_example():
    print("=== FreshBlu Python Sync Example ===\n")

    http = FreshBluHttp(hostname=SERVER, port=PORT)

    # 1. Status check
    print("1. Status:")
    status = http.status()
    print(f"   {status}\n")

    # 2. Register a device
    print("2. Registering device...")
    device = http.register({
        "type": "python-client",
        "framework": "freshblu-python",
    })
    print(f"   UUID:  {device['uuid']}")
    print(f"   Token: {device['token'][:8]}...\n")

    # Set credentials
    http.set_credentials(device["uuid"], device["token"])

    # 3. Whoami
    print("3. Whoami:")
    me = http.whoami()
    print(f"   {me['uuid']} ({me.get('type', 'unknown')})\n")

    # 4. Update device
    print("4. Updating device...")
    updated = http.update_device(device["uuid"], {
        "firmware": "1.0.0",
        "platform": "python",
    })
    print(f"   Updated hash: {updated['meshblu']['hash'][:16]}...\n")

    # 5. Register a second device
    device2 = http.register({"type": "python-listener"})
    http2 = FreshBluHttp(
        hostname=SERVER, port=PORT,
        uuid=device2["uuid"], token=device2["token"]
    )

    # 6. Subscribe device2 to device1 broadcasts
    print("5. Setting up subscriptions...")
    http2.create_subscription(
        subscriber_uuid=device2["uuid"],
        emitter_uuid=device["uuid"],
        subscription_type=SubscriptionType.BROADCAST_SENT,
    )
    print("   Subscribed device2 to device1 broadcasts\n")

    # 7. Send a message
    print("6. Sending broadcast message...")
    result = http.message({
        "devices": ["*"],
        "topic": "sensor-data",
        "payload": {"temperature": 23.4, "humidity": 65},
    })
    print(f"   Sent: {result}\n")

    # 8. Search
    print("7. Searching for Python clients...")
    devices = http.search({"type": "python-client"})
    print(f"   Found {len(devices)} device(s)\n")

    # 9. Token management
    print("8. Token management...")
    token_record = http2.generate_token(device2["uuid"], tag="session")
    print(f"   Generated token: {token_record['token'][:8]}...\n")

    # 10. Cleanup
    print("9. Cleaning up...")
    http.unregister(device["uuid"])
    http2.unregister(device2["uuid"])
    print("   Devices unregistered\n")

    print("=== Example complete ===")


async def async_example():
    """Async version using AsyncFreshBlu"""
    import asyncio
    print("\n=== Async Example ===\n")

    async with AsyncFreshBlu(hostname=SERVER, port=PORT) as client:
        status = await client.status()
        print(f"Status: {status}")

        device = await client.register({"type": "async-client"})
        client.set_credentials(device["uuid"], device["token"])
        print(f"Registered async device: {device['uuid']}")

        await client.message({
            "devices": ["*"],
            "payload": {"from": "async-example"},
        })
        print("Async message sent!")


def websocket_example():
    """WebSocket real-time example"""
    print("\n=== WebSocket Example ===\n")

    received = []

    device = FreshBluHttp(hostname=SERVER, port=PORT).register({"type": "ws-demo"})
    
    client = FreshBlu(
        hostname=SERVER,
        port=PORT,
        uuid=device["uuid"],
        token=device["token"],
    )

    def on_ready(data):
        print(f"Connected! UUID: {data.get('uuid')}")

    def on_message(msg):
        print(f"Message received: {msg}")
        received.append(msg)

    client.on("ready", on_ready)
    client.on("message", on_message)
    client.on("broadcast", on_message)

    client.connect()
    time.sleep(1)  # Wait for connection

    # Send a message to self
    http = FreshBluHttp(
        hostname=SERVER, port=PORT,
        uuid=device["uuid"], token=device["token"]
    )
    http.message({
        "devices": [device["uuid"]],
        "payload": {"hello": "from ws example"},
    })

    time.sleep(0.5)
    client.disconnect()
    
    http.unregister(device["uuid"])
    print(f"WebSocket example done, received {len(received)} messages")


if __name__ == "__main__":
    import asyncio

    try:
        sync_example()
        asyncio.run(async_example())
        # websocket_example()  # Uncomment if websockets is installed
    except Exception as e:
        print(f"Error: {e}")
        print("Make sure FreshBlu server is running: freshblu-server")
        raise
