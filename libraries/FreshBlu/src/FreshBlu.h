#ifndef FRESHBLU_H
#define FRESHBLU_H

#include "FreshBluHttp.h"
#include "FreshBluMqtt.h"

class FreshBlu {
public:
    FreshBlu(Client& httpClient, Client& mqttClient,
             const char* host, uint16_t httpPort = 3000, uint16_t mqttPort = 1883);

    FreshBluHttp http;
    FreshBluMqtt mqtt;

    // Register via HTTP, then connect MQTT with returned credentials
    bool begin(JsonDocument& properties);

    // Call in loop()
    void loop();
};

#endif
