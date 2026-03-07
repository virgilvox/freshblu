// SendMessage — Register a device, then send an HTTP message to another device.

#include <WiFi.h>
#include <FreshBlu.h>

const char* WIFI_SSID  = "your-ssid";
const char* WIFI_PASS  = "your-password";
const char* FRESHBLU_HOST = "192.168.1.100";

// UUID of the device you want to message (register it first)
const char* TARGET_UUID = "target-device-uuid";

WiFiClient net;
FreshBluHttp fb(net, FRESHBLU_HOST);

void setup() {
    Serial.begin(115200);
    delay(1000);

    WiFi.begin(WIFI_SSID, WIFI_PASS);
    while (WiFi.status() != WL_CONNECTED) delay(500);
    Serial.println("WiFi connected");

    // Register
    JsonDocument props;
    props["type"] = "controller";

    JsonDocument result;
    if (!fb.registerDevice(props, result)) {
        Serial.print("Register failed: ");
        Serial.println(fb.lastError());
        return;
    }
    Serial.print("Registered as: ");
    Serial.println(fb.uuid());

    // Send a message
    JsonDocument payload;
    payload["command"] = "turn_on";
    payload["brightness"] = 80;

    if (fb.sendMessage(TARGET_UUID, payload)) {
        Serial.println("Message sent!");
    } else {
        Serial.print("Send failed: ");
        Serial.println(fb.lastError());
    }
}

void loop() {
    // Send a reading every 10 seconds
    static unsigned long lastSend = 0;
    if (millis() - lastSend > 10000) {
        lastSend = millis();

        JsonDocument payload;
        payload["temp"] = analogRead(A0) * 0.1;

        if (fb.sendMessage(TARGET_UUID, payload)) {
            Serial.println("Reading sent");
        }
    }
}
