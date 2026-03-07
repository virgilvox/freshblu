#include "FreshBlu.h"

FreshBlu::FreshBlu(Client& httpClient, Client& mqttClient,
                   const char* host, uint16_t httpPort, uint16_t mqttPort)
    : http(httpClient, host, httpPort),
      mqtt(mqttClient, host, mqttPort) {
}

bool FreshBlu::begin(JsonDocument& properties) {
    JsonDocument result;
    if (!http.registerDevice(properties, result)) {
        return false;
    }

    return mqtt.connect(http.uuid(), http.token());
}

void FreshBlu::loop() {
    mqtt.loop();
}
