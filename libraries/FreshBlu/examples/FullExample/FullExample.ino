// FullExample — HTTP registration + property updates + MQTT real-time messaging.

#include <WiFi.h>
#include <FreshBlu.h>

const char* WIFI_SSID  = "your-ssid";
const char* WIFI_PASS  = "your-password";
const char* FRESHBLU_HOST = "192.168.1.100";

WiFiClient httpNet;
WiFiClient mqttNet;

FreshBlu fb(httpNet, mqttNet, FRESHBLU_HOST);

void onMessage(const char* fromUuid, const char* topic, JsonDocument& payload) {
    Serial.print("[");
    Serial.print(topic);
    Serial.print("] from ");
    Serial.print(fromUuid);
    Serial.print(": ");
    serializeJson(payload, Serial);
    Serial.println();

    // React to commands
    if (payload["command"] == "set_led") {
        int brightness = payload["value"] | 0;
        analogWrite(LED_BUILTIN, brightness);
        Serial.print("LED set to ");
        Serial.println(brightness);
    }
}

void setup() {
    Serial.begin(115200);
    pinMode(LED_BUILTIN, OUTPUT);
    delay(1000);

    WiFi.begin(WIFI_SSID, WIFI_PASS);
    Serial.print("Connecting");
    while (WiFi.status() != WL_CONNECTED) {
        delay(500);
        Serial.print(".");
    }
    Serial.println(" OK");

    // Register and connect
    JsonDocument props;
    props["type"] = "sensor-actuator";
    props["name"] = "living-room";
    props["capabilities"][0] = "temperature";
    props["capabilities"][1] = "led";

    if (!fb.begin(props)) {
        Serial.print("Setup failed: ");
        Serial.println(fb.http.lastError());
        while (true) delay(1000);
    }

    Serial.print("Device UUID: ");
    Serial.println(fb.http.uuid());

    // Verify with whoami
    JsonDocument whoami;
    if (fb.http.whoami(whoami)) {
        Serial.print("Server sees us as: ");
        serializeJson(whoami["uuid"], Serial);
        Serial.println();
    }

    // Set message callback
    fb.mqtt.onMessage(onMessage);

    Serial.println("Ready. Sending readings every 10s.");
}

void loop() {
    fb.loop();

    // Reconnect MQTT if disconnected
    if (!fb.mqtt.connected()) {
        Serial.println("MQTT disconnected, reconnecting...");
        if (fb.mqtt.connect(fb.http.uuid(), fb.http.token())) {
            Serial.println("Reconnected");
        } else {
            delay(5000);
            return;
        }
    }

    // Periodic sensor reading + property update
    static unsigned long lastUpdate = 0;
    if (millis() - lastUpdate > 10000) {
        lastUpdate = millis();

        float temp = 20.0 + random(0, 100) * 0.1;
        float humidity = 40.0 + random(0, 30);

        // Broadcast via MQTT
        JsonDocument payload;
        payload["temperature"] = temp;
        payload["humidity"] = humidity;
        fb.mqtt.broadcast(payload);

        // Update device properties via HTTP
        JsonDocument update;
        update["lastTemp"] = temp;
        update["lastHumidity"] = humidity;
        update["lastUpdate"] = millis() / 1000;

        JsonDocument result;
        if (fb.http.updateDevice(fb.http.uuid(), update, result)) {
            Serial.print("Updated: temp=");
            Serial.print(temp);
            Serial.print(" humidity=");
            Serial.println(humidity);
        }
    }
}
