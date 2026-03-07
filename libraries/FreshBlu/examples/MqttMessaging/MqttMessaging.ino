// MqttMessaging — Register via HTTP, then send and receive messages over MQTT.

#include <WiFi.h>
#include <FreshBlu.h>

const char* WIFI_SSID  = "your-ssid";
const char* WIFI_PASS  = "your-password";
const char* FRESHBLU_HOST = "192.168.1.100";

WiFiClient httpNet;
WiFiClient mqttNet;

FreshBlu fb(httpNet, mqttNet, FRESHBLU_HOST);

void onMessage(const char* fromUuid, const char* topic, JsonDocument& payload) {
    Serial.print("Message from ");
    Serial.print(fromUuid);
    Serial.print(" [");
    Serial.print(topic);
    Serial.print("]: ");
    serializeJson(payload, Serial);
    Serial.println();
}

void setup() {
    Serial.begin(115200);
    delay(1000);

    WiFi.begin(WIFI_SSID, WIFI_PASS);
    while (WiFi.status() != WL_CONNECTED) delay(500);
    Serial.println("WiFi connected");

    // Register device and connect MQTT in one call
    JsonDocument props;
    props["type"] = "mqtt-device";

    if (!fb.begin(props)) {
        Serial.print("Setup failed: ");
        Serial.println(fb.http.lastError());
        return;
    }

    Serial.print("Connected as: ");
    Serial.println(fb.http.uuid());

    // Set up message handler
    fb.mqtt.onMessage(onMessage);
}

void loop() {
    fb.loop();

    // Broadcast a reading every 5 seconds
    static unsigned long lastBroadcast = 0;
    if (millis() - lastBroadcast > 5000) {
        lastBroadcast = millis();

        JsonDocument payload;
        payload["temp"] = 22.5 + random(0, 50) * 0.1;
        payload["humidity"] = 45 + random(0, 20);

        if (fb.mqtt.broadcast(payload)) {
            Serial.println("Broadcast sent");
        }
    }
}
