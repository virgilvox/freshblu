# FreshBlu Arduino Library

Arduino client for the FreshBlu IoT messaging platform (Meshblu-compatible). Provides HTTP REST for device management and MQTT for real-time messaging.

## Requirements

- A running FreshBlu server (see [Server Setup](#server-setup))
- **ArduinoJson v7** — JSON serialization
- **PubSubClient v2.8+** — MQTT client (only needed if using `FreshBluMqtt`)
- Any Arduino-compatible board with a `Client`-based network stack (ESP32, ESP8266, Arduino + Ethernet, etc.)

## Installation

### Arduino IDE

1. Copy the entire `FreshBlu` folder into your Arduino `libraries/` directory (typically `~/Arduino/libraries/` on macOS/Linux or `Documents\Arduino\libraries\` on Windows)
2. Install dependencies via **Sketch > Include Library > Manage Libraries**:
   - Search **ArduinoJson** by Benoit Blanchon, install v7.x
   - Search **PubSubClient** by Nick O'Leary, install v2.8+
3. Restart the Arduino IDE

### PlatformIO

Add to your `platformio.ini`:

```ini
lib_deps =
    bblanchon/ArduinoJson@^7
    knolleary/PubSubClient@^2.8
```

Then copy the `FreshBlu` folder into your project's `lib/` directory.

## Server Setup

Start a FreshBlu server for your devices to connect to:

```bash
# From the FreshBlu repository root
cargo run --bin freshblu-server
```

The server listens on:
- **HTTP:** port 3000 (configurable via `FRESHBLU_HTTP_PORT`)
- **MQTT:** port 1883 (configurable via `FRESHBLU_MQTT_PORT`)

For production, see `docker/docker-compose.prod.yml`.

## Quick Start

### HTTP Only (simplest)

```cpp
#include <WiFi.h>
#include <FreshBluHttp.h>

WiFiClient net;
FreshBluHttp fb(net, "192.168.1.100", 3000);

void setup() {
    Serial.begin(115200);
    WiFi.begin("ssid", "password");
    while (WiFi.status() != WL_CONNECTED) delay(500);

    // Register a new device
    JsonDocument props;
    props["type"] = "sensor";

    JsonDocument result;
    if (fb.registerDevice(props, result)) {
        Serial.print("UUID:  "); Serial.println(fb.uuid());
        Serial.print("Token: "); Serial.println(fb.token());
        // IMPORTANT: Save uuid() and token() to NVS/EEPROM for reuse after reboot
    } else {
        Serial.print("Error: "); Serial.println(fb.lastError());
    }
}

void loop() {}
```

### HTTP + MQTT (full)

```cpp
#include <WiFi.h>
#include <FreshBlu.h>

WiFiClient httpNet;
WiFiClient mqttNet;
FreshBlu fb(httpNet, mqttNet, "192.168.1.100");

void onMessage(const char* fromUuid, const char* topic, JsonDocument& payload) {
    Serial.print("From "); Serial.print(fromUuid);
    Serial.print(": "); serializeJson(payload, Serial);
    Serial.println();
}

void setup() {
    Serial.begin(115200);
    WiFi.begin("ssid", "password");
    while (WiFi.status() != WL_CONNECTED) delay(500);

    JsonDocument props;
    props["type"] = "sensor";

    if (fb.begin(props)) {
        Serial.print("Connected as: "); Serial.println(fb.http.uuid());
        fb.mqtt.onMessage(onMessage);
    } else {
        Serial.print("Failed: "); Serial.println(fb.http.lastError());
    }
}

void loop() {
    fb.loop();  // Must be called regularly to process MQTT
}
```

Two separate `Client` objects are required because HTTP and MQTT maintain independent TCP connections.

## Saving Credentials

`registerDevice()` creates a new device every time. To persist across reboots, save the UUID and token, then restore with `setAuth()`:

```cpp
#include <Preferences.h>  // ESP32 NVS

Preferences prefs;

void setup() {
    // ...WiFi setup...
    prefs.begin("freshblu", false);

    String savedUuid = prefs.getString("uuid", "");
    String savedToken = prefs.getString("token", "");

    if (savedUuid.length() > 0) {
        // Restore saved credentials
        fb.setAuth(savedUuid.c_str(), savedToken.c_str());

        // Verify they still work
        JsonDocument whoami;
        if (!fb.whoami(whoami)) {
            // Token revoked or device deleted — re-register
            savedUuid = "";
        }
    }

    if (savedUuid.length() == 0) {
        // First boot — register
        JsonDocument props;
        props["type"] = "sensor";
        JsonDocument result;
        if (fb.registerDevice(props, result)) {
            prefs.putString("uuid", fb.uuid());
            prefs.putString("token", fb.token());
        }
    }
}
```

For ESP8266, use `EEPROM.h` or `LittleFS` instead of `Preferences`.

## API Reference

### FreshBluHttp

The HTTP client handles device registration, authentication, property updates, and messaging via the REST API.

```cpp
FreshBluHttp(Client& client, const char* host, uint16_t port = 3000);
```

The `host` string must remain valid for the lifetime of the object (use a global `const char*` or string literal).

#### Credentials

```cpp
void setAuth(const char* uuid, const char* token);
const char* uuid() const;   // Current device UUID (empty string if not set)
const char* token() const;  // Current device token (empty string if not set)
```

`registerDevice()` calls `setAuth()` automatically with the returned credentials. Call `setAuth()` manually when restoring saved credentials.

#### Device Management

```cpp
// Register a new device. Properties are sent as the request body.
// On success, auto-calls setAuth() with the returned uuid/token.
bool registerDevice(JsonDocument& properties, JsonDocument& result);

// Verify current credentials are valid. Returns {"uuid":"..."} on success.
bool authenticate(JsonDocument& result);

// Get the authenticated device's own data.
bool whoami(JsonDocument& result);

// Get another device by UUID. Requires discover.view permission.
bool getDevice(const char* uuid, JsonDocument& result);

// Update device properties. Requires configure.update permission.
// System fields (uuid, token, meshblu) cannot be overwritten.
bool updateDevice(const char* uuid, JsonDocument& properties, JsonDocument& result);

// Delete a device. Requires configure.update permission.
bool unregister(const char* uuid);
```

#### Messaging

```cpp
// Send a direct message to a specific device.
// Requires the target to have the sender in its message.from whitelist.
bool sendMessage(const char* targetUuid, JsonDocument& payload);

// Broadcast to all devices subscribed to this device's broadcast.sent events.
bool broadcast(JsonDocument& payload);
```

#### Status

```cpp
// Check server status. No authentication required.
bool status(JsonDocument& result);
// Returns: {"meshblu":true,"version":"2.0.0","online":true,"connections":N,"engine":"freshblu"}
```

#### Error Handling

All methods return `false` on failure. Inspect the error:

```cpp
int lastHttpStatus() const;   // HTTP status code (0 if connection failed)
const char* lastError() const; // Human-readable error string
```

```cpp
if (!fb.sendMessage(target, payload)) {
    if (fb.lastHttpStatus() == 0) {
        Serial.println("Network error — check WiFi");
    } else if (fb.lastHttpStatus() == 401) {
        Serial.println("Bad credentials — re-register");
    } else if (fb.lastHttpStatus() == 403) {
        Serial.println("Permission denied — check whitelists");
    } else {
        Serial.print("Error: "); Serial.println(fb.lastError());
    }
}
```

### FreshBluMqtt

MQTT client for real-time pub/sub messaging. Wraps PubSubClient with FreshBlu topic conventions.

```cpp
FreshBluMqtt(Client& client, const char* host, uint16_t port = 1883);
```

**Limitation:** Only one `FreshBluMqtt` instance can receive callbacks at a time (PubSubClient uses a global C callback). If you need multiple MQTT connections, use PubSubClient directly.

#### Connection

```cpp
// Connect with device credentials (uuid as client ID, uuid as username, token as password).
// Automatically subscribes to {uuid}/message and {uuid}/broadcast.
bool connect(const char* uuid, const char* token);

void disconnect();
bool connected();

// Must be called in loop() to process incoming messages and maintain the connection.
void loop();
```

#### Messaging

```cpp
// Send a direct message. Publishes to {uuid}/message with the FreshBlu envelope.
bool sendMessage(const char* targetUuid, JsonDocument& payload);

// Broadcast. Publishes to {uuid}/broadcast.
bool broadcast(JsonDocument& payload);
```

#### Receiving Messages

```cpp
typedef void (*FreshBluMessageCallback)(
    const char* fromUuid,   // Sender's UUID
    const char* topic,      // "message" or "broadcast"
    JsonDocument& payload   // Parsed JSON payload
);

void onMessage(FreshBluMessageCallback callback);
```

### FreshBlu (Convenience Wrapper)

Combines HTTP + MQTT into a single object.

```cpp
FreshBlu(Client& httpClient, Client& mqttClient,
         const char* host, uint16_t httpPort = 3000, uint16_t mqttPort = 1883);

FreshBluHttp http;   // Direct access to HTTP client
FreshBluMqtt mqtt;   // Direct access to MQTT client

// Register via HTTP, then connect MQTT with the returned credentials.
bool begin(JsonDocument& properties);

// Calls mqtt.loop(). Must be called in your loop() function.
void loop();
```

## MQTT Reconnection

MQTT connections can drop. Handle reconnection in your `loop()`:

```cpp
void loop() {
    fb.loop();

    if (!fb.mqtt.connected()) {
        Serial.println("MQTT disconnected, reconnecting...");
        if (fb.mqtt.connect(fb.http.uuid(), fb.http.token())) {
            Serial.println("Reconnected");
            fb.mqtt.onMessage(onMessage);  // Re-register callback
        } else {
            delay(5000);  // Back off before retrying
        }
    }
}
```

## HTTPS / TLS

For encrypted connections, pass a `WiFiClientSecure` instead of `WiFiClient`:

```cpp
#include <WiFiClientSecure.h>

WiFiClientSecure secureNet;
secureNet.setInsecure();  // Skip certificate validation (dev only)
// OR: secureNet.setCACert(root_ca);  // Production: pin the server's CA cert

FreshBluHttp fb(secureNet, "freshblu.example.com", 443);
```

The library is transport-agnostic — any class that extends Arduino's `Client` works.

## Configuration

| Define | Default | Description |
|--------|---------|-------------|
| `FRESHBLU_MAX_RESPONSE` | `2048` | Max HTTP response body size (bytes, stack-allocated) |
| `FRESHBLU_MQTT_BUFFER_SIZE` | `512` | PubSubClient buffer size for MQTT packets |

Override before including the library:

```cpp
#define FRESHBLU_MAX_RESPONSE 4096
#define FRESHBLU_MQTT_BUFFER_SIZE 1024
#include <FreshBlu.h>
```

**Stack usage note:** `FRESHBLU_MAX_RESPONSE` is allocated on the stack per HTTP request. ESP32 default task stack is 8KB. If you increase this value on memory-constrained boards, watch for stack overflows.

The HTTP request body buffer is fixed at 512 bytes. If your properties document exceeds this, the method returns `false` with a "Properties too large" error.

## Message Format

Messages sent via both HTTP and MQTT use the same JSON envelope:

```json
{
  "devices": ["target-uuid"],
  "payload": { "temperature": 22.5, "unit": "C" }
}
```

- **Direct message:** `"devices": ["target-uuid"]` — sent to one specific device
- **Broadcast:** `"devices": ["*"]` — sent to all broadcast subscribers

The `sendMessage()` and `broadcast()` methods build this envelope automatically.

## Permissions

FreshBlu uses Meshblu v2.0 whitelists for access control. By default, newly registered devices have open whitelists (any device can discover, message, etc.). To restrict access:

```cpp
JsonDocument props;
props["type"] = "private-sensor";
// Whitelists are set via the meshblu.whitelists property — see the FreshBlu server docs
```

Common permission errors:
- **403 on `sendMessage()`**: Target device's `message.from` whitelist doesn't include the sender
- **403/404 on `getDevice()`**: Target's `discover.view` whitelist doesn't include the requester
- **403 on `updateDevice()`**: Target's `configure.update` whitelist doesn't include the requester

## Examples

| Example | Description |
|---------|-------------|
| **BasicRegister** | Register a device, print UUID and token |
| **SendMessage** | Register, then send periodic HTTP messages |
| **MqttMessaging** | Register via HTTP, real-time pub/sub via MQTT |
| **FullExample** | HTTP + MQTT + property updates + command handling + reconnection |

## Tested Platforms

- ESP32 (WiFi)
- ESP8266 (WiFi)
- Arduino Uno/Mega + Ethernet Shield (W5100/W5500)

Works with any board that provides a `Client`-compatible network class.

## License

MIT
