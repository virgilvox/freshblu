#include "FreshBluMqtt.h"

FreshBluMqtt* FreshBluMqtt::_instance = nullptr;

FreshBluMqtt::FreshBluMqtt(Client& client, const char* host, uint16_t port)
    : _mqtt(client), _host(host), _port(port), _messageCallback(nullptr) {
    _uuid[0] = '\0';
    _instance = this;
}

void FreshBluMqtt::_staticCallback(char* topic, byte* payload, unsigned int length) {
    if (_instance) {
        _instance->_handleMessage(topic, payload, length);
    }
}

bool FreshBluMqtt::connect(const char* uuid, const char* token) {
    strncpy(_uuid, uuid, sizeof(_uuid) - 1);
    _uuid[sizeof(_uuid) - 1] = '\0';

    _mqtt.setServer(_host, _port);
    _mqtt.setBufferSize(FRESHBLU_MQTT_BUFFER_SIZE);
    _mqtt.setCallback(_staticCallback);

    if (!_mqtt.connect(_uuid, _uuid, token)) {
        return false;
    }

    // Subscribe to incoming messages and broadcasts
    char topic[74]; // uuid(36) + /broadcast(10) + null
    snprintf(topic, sizeof(topic), "%s/message", _uuid);
    _mqtt.subscribe(topic);

    snprintf(topic, sizeof(topic), "%s/broadcast", _uuid);
    _mqtt.subscribe(topic);

    return true;
}

void FreshBluMqtt::disconnect() {
    _mqtt.disconnect();
}

bool FreshBluMqtt::connected() {
    return _mqtt.connected();
}

void FreshBluMqtt::loop() {
    _mqtt.loop();
}

bool FreshBluMqtt::sendMessage(const char* targetUuid, JsonDocument& payload) {
    if (!_mqtt.connected()) return false;

    JsonDocument msg;
    msg["devices"][0] = targetUuid;
    msg["payload"] = payload;

    char buf[512];
    size_t len = serializeJson(msg, buf, sizeof(buf));
    if (len >= sizeof(buf)) return false;

    char topic[74];
    snprintf(topic, sizeof(topic), "%s/message", _uuid);

    return _mqtt.publish(topic, (const uint8_t*)buf, len);
}

bool FreshBluMqtt::broadcast(JsonDocument& payload) {
    if (!_mqtt.connected()) return false;

    char buf[512];
    size_t len = serializeJson(payload, buf, sizeof(buf));
    if (len >= sizeof(buf)) return false;

    char topic[74];
    snprintf(topic, sizeof(topic), "%s/broadcast", _uuid);

    return _mqtt.publish(topic, (const uint8_t*)buf, len);
}

void FreshBluMqtt::onMessage(FreshBluMessageCallback callback) {
    _messageCallback = callback;
}

void FreshBluMqtt::_handleMessage(char* topic, byte* payload, unsigned int length) {
    if (!_messageCallback) return;

    JsonDocument doc;
    DeserializationError err = deserializeJson(doc, payload, length);
    if (err) return;

    // Extract sender UUID from topic: "{uuid}/message" or "{uuid}/broadcast"
    char fromUuid[37] = {0};
    const char* slash = strchr(topic, '/');
    if (slash) {
        size_t uuidLen = slash - topic;
        if (uuidLen >= sizeof(fromUuid)) uuidLen = sizeof(fromUuid) - 1;
        memcpy(fromUuid, topic, uuidLen);
        fromUuid[uuidLen] = '\0';
    }

    // Determine topic type (after the slash)
    const char* topicType = slash ? slash + 1 : topic;

    // For messages, payload is nested; for broadcasts, it's the whole doc
    if (strcmp(topicType, "message") == 0) {
        // Message format: {"devices":[...],"payload":{...},"fromUuid":"..."}
        const char* from = doc["fromUuid"] | fromUuid;
        JsonDocument payloadDoc;
        payloadDoc.set(doc["payload"]);
        _messageCallback(from, topicType, payloadDoc);
    } else {
        _messageCallback(fromUuid, topicType, doc);
    }
}
