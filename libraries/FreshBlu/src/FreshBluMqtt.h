#ifndef FRESHBLU_MQTT_H
#define FRESHBLU_MQTT_H

#include <Arduino.h>
#include <Client.h>
#include <ArduinoJson.h>
#include <PubSubClient.h>

#ifndef FRESHBLU_MQTT_BUFFER_SIZE
#define FRESHBLU_MQTT_BUFFER_SIZE 512
#endif

typedef void (*FreshBluMessageCallback)(const char* fromUuid, const char* topic, JsonDocument& payload);

// NOTE: Only one FreshBluMqtt instance can receive callbacks at a time.
// This is a PubSubClient limitation — it uses a C function pointer for its
// callback, so a static singleton dispatches to the most recently created
// instance. If you need multiple MQTT connections, use PubSubClient directly.
class FreshBluMqtt {
public:
    FreshBluMqtt(Client& client, const char* host, uint16_t port = 1883);

    bool connect(const char* uuid, const char* token);
    void disconnect();
    bool connected();

    void loop();

    bool sendMessage(const char* targetUuid, JsonDocument& payload);
    bool broadcast(JsonDocument& payload);

    void onMessage(FreshBluMessageCallback callback);

private:
    PubSubClient _mqtt;
    const char* _host;
    uint16_t _port;
    char _uuid[37];
    FreshBluMessageCallback _messageCallback;

    void _handleMessage(char* topic, byte* payload, unsigned int length);

    static FreshBluMqtt* _instance;
    static void _staticCallback(char* topic, byte* payload, unsigned int length);
};

#endif
