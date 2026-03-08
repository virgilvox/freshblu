<script lang="ts">
  import CodeBlock from '$lib/components/ui/CodeBlock.svelte';
</script>

<svelte:head><title>Arduino Sensor - FreshBlu Docs</title></svelte:head>

<div class="doc-page">
  <h1 class="doc-title">Arduino Sensor</h1>
  <p class="doc-intro">Wire a sensor to an Arduino, push readings to FreshBlu over MQTT, and handle commands from the server.</p>

  <h2>1. Install the Library</h2>
  <p>Open the Arduino IDE. Go to Sketch &gt; Include Library &gt; Manage Libraries. Search for <code>FreshBlu</code> and install it.</p>
  <p>Alternatively, download the ZIP from the releases page and install via Sketch &gt; Include Library &gt; Add .ZIP Library.</p>

  <h2>2. Include Headers</h2>
  <p>Your sketch needs two includes. <code>FreshBlu.h</code> provides core types. <code>FreshBluMqtt.h</code> handles the MQTT transport.</p>
  <CodeBlock lang="cpp" code={`#include <WiFi.h>
#include <FreshBlu.h>
#include <FreshBluMqtt.h>`} />

  <h2>3. Configure WiFi and Client</h2>
  <p>Set your WiFi credentials and create the MQTT client with the server address, port, device UUID, and token.</p>
  <CodeBlock lang="cpp" code={`const char* WIFI_SSID  = "YourNetwork";
const char* WIFI_PASS  = "YourPassword";

const char* SERVER     = "192.168.1.100";
const int   PORT       = 1883;
const char* UUID       = "your-device-uuid";
const char* TOKEN      = "your-device-token";

FreshBluMqtt client(SERVER, PORT, UUID, TOKEN);

void setup() {
  Serial.begin(115200);

  WiFi.begin(WIFI_SSID, WIFI_PASS);
  while (WiFi.status() != WL_CONNECTED) {
    delay(500);
    Serial.print(".");
  }
  Serial.println("\\nWiFi connected");

  client.begin();
}`} />

  <h2>4. Read Sensor and Send Messages</h2>
  <p>In your loop, read the sensor value, build a JSON payload, and call <code>sendMessage</code> with the target UUID.</p>
  <CodeBlock lang="cpp" code={`const char* TARGET_UUID = "dashboard-device-uuid";

void loop() {
  client.loop();  // keep MQTT connection alive

  int raw = analogRead(A0);
  float voltage = raw * (3.3 / 4095.0);
  float tempC = voltage * 100.0;

  char payload[64];
  snprintf(payload, sizeof(payload),
    "{\\"temp\\": %.1f, \\"raw\\": %d}", tempC, raw);

  client.sendMessage(TARGET_UUID, payload);

  delay(5000);  // send every 5 seconds
}`} />

  <h2>5. Handle Incoming Messages</h2>
  <p>Register a callback to process messages sent to this device. The callback receives the sender UUID, topic, and payload.</p>
  <CodeBlock lang="cpp" code={`void onMessage(const char* fromUuid, const char* topic,
               const char* payload) {
  Serial.print("From: ");
  Serial.println(fromUuid);
  Serial.print("Payload: ");
  Serial.println(payload);

  // example: parse a command
  if (strstr(payload, "\\"led\\": true")) {
    digitalWrite(LED_BUILTIN, HIGH);
  }
}

void setup() {
  Serial.begin(115200);
  pinMode(LED_BUILTIN, OUTPUT);

  WiFi.begin(WIFI_SSID, WIFI_PASS);
  while (WiFi.status() != WL_CONNECTED) {
    delay(500);
  }

  client.onMessage(onMessage);
  client.begin();
}`} />
  <p>The callback fires on the main loop thread. Keep it short. Offload heavy work to flags checked in <code>loop()</code>.</p>
</div>

<style>
  .doc-page { max-width: 740px; }
  .doc-title { font-family: var(--font-display); font-size: var(--text-2xl); font-weight: 700; letter-spacing: 0.04em; text-transform: uppercase; margin-bottom: 8px; }
  .doc-intro { font-size: var(--text-sm); color: var(--ink-soft); line-height: var(--leading-relaxed); margin-bottom: 40px; }
  h2 { font-family: var(--font-display); font-size: var(--text-lg); font-weight: 700; letter-spacing: 0.04em; margin: 40px 0 16px; padding-bottom: 8px; border-bottom: 1px solid var(--border); }
  p { font-size: var(--text-sm); color: var(--ink-soft); line-height: var(--leading-relaxed); margin-bottom: 16px; }
  code { font-family: var(--font-body); font-size: var(--text-sm); color: var(--pulse); }
</style>
