// BasicRegister — Register a device on a FreshBlu server and print credentials.
//
// Works with ESP32, ESP8266, Arduino + Ethernet, etc.
// Change WiFi/network setup to match your board.

#include <WiFi.h>       // ESP32; use <ESP8266WiFi.h> for ESP8266
#include <FreshBlu.h>

const char* WIFI_SSID  = "your-ssid";
const char* WIFI_PASS  = "your-password";
const char* FRESHBLU_HOST = "192.168.1.100";
const uint16_t FRESHBLU_PORT = 3000;

WiFiClient net;
FreshBluHttp fb(net, FRESHBLU_HOST, FRESHBLU_PORT);

void setup() {
    Serial.begin(115200);
    delay(1000);

    // Connect to WiFi
    WiFi.begin(WIFI_SSID, WIFI_PASS);
    Serial.print("Connecting to WiFi");
    while (WiFi.status() != WL_CONNECTED) {
        delay(500);
        Serial.print(".");
    }
    Serial.println(" connected!");

    // Register a new device
    JsonDocument props;
    props["type"] = "sensor";
    props["name"] = "temp-01";

    JsonDocument result;
    if (fb.registerDevice(props, result)) {
        Serial.println("Registered!");
        Serial.print("  UUID:  ");
        Serial.println(fb.uuid());
        Serial.print("  Token: ");
        Serial.println(fb.token());
    } else {
        Serial.print("Registration failed: ");
        Serial.println(fb.lastError());
    }
}

void loop() {
    // Nothing to do
}
