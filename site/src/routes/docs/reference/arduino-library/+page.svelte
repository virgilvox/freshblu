<script>
  import CodeBlock from '$lib/components/ui/CodeBlock.svelte';
</script>

<svelte:head><title>Arduino Library Reference - FreshBlu Docs</title></svelte:head>
<div class="doc-page">
  <h1 class="doc-title">Arduino Library</h1>
  <p class="doc-intro">The FreshBlu Arduino library provides HTTP and MQTT clients for ESP8266, ESP32, and other Arduino-compatible boards. It wraps the FreshBlu REST API and MQTT broker into a simple interface for IoT devices.</p>

  <h2>Installation</h2>
  <p>Copy the <code>libraries/FreshBlu</code> folder into your Arduino libraries directory, or add it as a library in PlatformIO. The library depends on <code>ArduinoJson</code> and <code>PubSubClient</code>.</p>

  <h2>FreshBlu (Main Class)</h2>
  <p>The top-level class that combines HTTP and MQTT functionality. Handles registration and connection in a single call.</p>

  <h3>Constructor</h3>
  <CodeBlock lang="cpp" code={`FreshBlu(Client& httpClient, Client& mqttClient,
         const char* host, uint16_t httpPort = 3000, uint16_t mqttPort = 1883);`} />
  <p>Takes two <code>Client</code> references (typically <code>WiFiClient</code> instances). One is used for HTTP requests, the other for the persistent MQTT connection. Both can be the same client if you do not need simultaneous HTTP and MQTT.</p>

  <h3>Members</h3>
  <table class="config-table">
    <thead>
      <tr>
        <th>Member</th>
        <th>Type</th>
        <th>Description</th>
      </tr>
    </thead>
    <tbody>
      <tr>
        <td><code>http</code></td>
        <td><code>FreshBluHttp</code></td>
        <td>HTTP client instance</td>
      </tr>
      <tr>
        <td><code>mqtt</code></td>
        <td><code>FreshBluMqtt</code></td>
        <td>MQTT client instance</td>
      </tr>
    </tbody>
  </table>

  <h3>begin()</h3>
  <CodeBlock lang="cpp" code={`bool begin(JsonDocument& properties);`} />
  <p>Registers the device via HTTP, then connects to the MQTT broker using the returned credentials. The <code>properties</code> document is sent as the registration body. Returns <code>true</code> on success.</p>

  <h3>loop()</h3>
  <CodeBlock lang="cpp" code={`void loop();`} />
  <p>Call this in your Arduino <code>loop()</code> function. Handles MQTT keepalive and incoming message dispatch.</p>

  <h2>FreshBluHttp</h2>
  <p>HTTP client for the FreshBlu REST API. Handles Basic Auth header construction and JSON serialization.</p>

  <h3>Constructor</h3>
  <CodeBlock lang="cpp" code={`FreshBluHttp(Client& client, const char* host, uint16_t port = 3000);`} />

  <h3>setAuth()</h3>
  <CodeBlock lang="cpp" code={`void setAuth(const char* uuid, const char* token);`} />
  <p>Set credentials for subsequent requests. Builds the Base64-encoded Basic Auth header internally.</p>

  <h3>uuid() / token()</h3>
  <CodeBlock lang="cpp" code={`const char* uuid() const;
const char* token() const;`} />
  <p>Return the currently configured credentials.</p>

  <h3>Device Management</h3>
  <table class="config-table">
    <thead>
      <tr>
        <th>Method</th>
        <th>Maps to</th>
        <th>Description</th>
      </tr>
    </thead>
    <tbody>
      <tr>
        <td><code>registerDevice(properties, result)</code></td>
        <td><code>POST /devices</code></td>
        <td>Register a new device. Populates <code>result</code> with the response including the token.</td>
      </tr>
      <tr>
        <td><code>authenticate(result)</code></td>
        <td><code>POST /authenticate</code></td>
        <td>Verify current credentials.</td>
      </tr>
      <tr>
        <td><code>whoami(result)</code></td>
        <td><code>GET /whoami</code></td>
        <td>Get the authenticated device's properties.</td>
      </tr>
      <tr>
        <td><code>getDevice(uuid, result)</code></td>
        <td><code>GET /devices/:uuid</code></td>
        <td>Get another device's properties.</td>
      </tr>
      <tr>
        <td><code>updateDevice(uuid, properties, result)</code></td>
        <td><code>PUT /devices/:uuid</code></td>
        <td>Update device properties.</td>
      </tr>
      <tr>
        <td><code>unregister(uuid)</code></td>
        <td><code>DELETE /devices/:uuid</code></td>
        <td>Delete a device.</td>
      </tr>
    </tbody>
  </table>

  <h3>Messaging</h3>
  <table class="config-table">
    <thead>
      <tr>
        <th>Method</th>
        <th>Maps to</th>
        <th>Description</th>
      </tr>
    </thead>
    <tbody>
      <tr>
        <td><code>sendMessage(targetUuid, payload)</code></td>
        <td><code>POST /messages</code></td>
        <td>Send a message to a specific device.</td>
      </tr>
      <tr>
        <td><code>broadcast(payload)</code></td>
        <td><code>POST /broadcasts</code></td>
        <td>Broadcast a message to all subscribers.</td>
      </tr>
    </tbody>
  </table>

  <h3>Status</h3>
  <CodeBlock lang="cpp" code={`bool status(JsonDocument& result);`} />
  <p>Calls <code>GET /status</code> and populates <code>result</code>.</p>

  <h3>Error Handling</h3>
  <CodeBlock lang="cpp" code={`int lastHttpStatus() const;
const char* lastError() const;`} />
  <p><code>lastHttpStatus()</code> returns the HTTP status code from the most recent request. <code>lastError()</code> returns a human-readable error string (up to 128 characters).</p>

  <h3>Buffer Size</h3>
  <p>The maximum HTTP response size defaults to 2048 bytes. Override by defining <code>FRESHBLU_MAX_RESPONSE</code> before including the header:</p>
  <CodeBlock lang="cpp" code={`#define FRESHBLU_MAX_RESPONSE 4096
#include <FreshBlu.h>`} />

  <h2>FreshBluMqtt</h2>
  <p>MQTT client for real-time messaging. Built on top of PubSubClient.</p>

  <h3>Constructor</h3>
  <CodeBlock lang="cpp" code={`FreshBluMqtt(Client& client, const char* host, uint16_t port = 1883);`} />

  <h3>connect()</h3>
  <CodeBlock lang="cpp" code={`bool connect(const char* uuid, const char* token);`} />
  <p>Connect to the MQTT broker using device credentials. The UUID is used as the MQTT username. Returns <code>true</code> on success.</p>

  <h3>disconnect()</h3>
  <CodeBlock lang="cpp" code={`void disconnect();`} />

  <h3>connected()</h3>
  <CodeBlock lang="cpp" code={`bool connected();`} />
  <p>Returns <code>true</code> if the MQTT client is currently connected.</p>

  <h3>loop()</h3>
  <CodeBlock lang="cpp" code={`void loop();`} />
  <p>Process incoming MQTT messages and maintain the connection. Call this frequently.</p>

  <h3>sendMessage()</h3>
  <CodeBlock lang="cpp" code={`bool sendMessage(const char* targetUuid, JsonDocument& payload);`} />
  <p>Publish a message to <code>{'{uuid}'}/message</code> topic targeting the specified device.</p>

  <h3>broadcast()</h3>
  <CodeBlock lang="cpp" code={`bool broadcast(JsonDocument& payload);`} />
  <p>Publish a broadcast to <code>{'{uuid}'}/broadcast</code> topic.</p>

  <h3>onMessage()</h3>
  <CodeBlock lang="cpp" code={`void onMessage(FreshBluMessageCallback callback);`} />
  <p>Register a callback for incoming messages. The callback signature is:</p>
  <CodeBlock lang="cpp" code={`void callback(const char* fromUuid, const char* topic, JsonDocument& payload);`} />

  <h3>Buffer Size</h3>
  <p>The MQTT buffer defaults to 512 bytes. Override with <code>FRESHBLU_MQTT_BUFFER_SIZE</code>:</p>
  <CodeBlock lang="cpp" code={`#define FRESHBLU_MQTT_BUFFER_SIZE 1024
#include <FreshBlu.h>`} />

  <h3>Singleton Limitation</h3>
  <p>Only one <code>FreshBluMqtt</code> instance can receive callbacks at a time. This is a PubSubClient limitation. It uses a C function pointer for its callback, so a static singleton dispatches to the most recently created instance. If you need multiple MQTT connections, use PubSubClient directly.</p>

  <h2>Example</h2>
  <CodeBlock lang="cpp" code={`#include <WiFi.h>
#include <FreshBlu.h>

WiFiClient httpClient, mqttClient;
FreshBlu fb(httpClient, mqttClient, "192.168.1.100");

void onMsg(const char* from, const char* topic, JsonDocument& payload) {
  Serial.print("Message from ");
  Serial.println(from);
}

void setup() {
  Serial.begin(115200);
  WiFi.begin("ssid", "password");
  while (WiFi.status() != WL_CONNECTED) delay(500);

  JsonDocument props;
  props["name"] = "esp32-sensor";
  props["type"] = "sensor";

  if (fb.begin(props)) {
    Serial.print("Registered as ");
    Serial.println(fb.http.uuid());
    fb.mqtt.onMessage(onMsg);
  }
}

void loop() {
  fb.loop();

  static unsigned long last = 0;
  if (millis() - last > 5000) {
    last = millis();
    JsonDocument payload;
    payload["temp"] = analogRead(A0) * 0.1;
    fb.mqtt.broadcast(payload);
  }
}`} />
</div>

<style>
  .doc-page { max-width: 740px; }
  .doc-title { font-family: var(--font-display); font-size: var(--text-2xl); font-weight: 700; letter-spacing: 0.04em; text-transform: uppercase; margin-bottom: 8px; }
  .doc-intro { font-size: var(--text-sm); color: var(--ink-soft); line-height: var(--leading-relaxed); margin-bottom: 40px; }
  h2 { font-family: var(--font-display); font-size: var(--text-lg); font-weight: 700; letter-spacing: 0.04em; margin: 40px 0 16px; padding-bottom: 8px; border-bottom: 1px solid var(--border); }
  h3 { font-family: var(--font-display); font-size: var(--text-md); font-weight: 600; margin: 24px 0 8px; }
  p { font-size: var(--text-sm); color: var(--ink-soft); line-height: var(--leading-relaxed); margin-bottom: 16px; }
  code { font-family: var(--font-body); font-size: var(--text-sm); color: var(--pulse); }
  .config-table { width: 100%; border-collapse: collapse; margin-bottom: 24px; }
  .config-table th { font-family: var(--font-ui); font-size: 9px; letter-spacing: 0.15em; text-transform: uppercase; color: var(--ink-muted); text-align: left; padding: 8px 12px; border-bottom: 1px solid var(--border); }
  .config-table td { font-family: var(--font-ui); font-size: var(--text-xs); padding: 10px 12px; border-bottom: 1px solid var(--border); color: var(--ink-soft); }
  .config-table td code { color: var(--pulse); }
</style>
