#ifndef FRESHBLU_HTTP_H
#define FRESHBLU_HTTP_H

#include <Arduino.h>
#include <Client.h>
#include <ArduinoJson.h>

#ifndef FRESHBLU_MAX_RESPONSE
#define FRESHBLU_MAX_RESPONSE 2048
#endif

class FreshBluHttp {
public:
    FreshBluHttp(Client& client, const char* host, uint16_t port = 3000);

    void setAuth(const char* uuid, const char* token);

    const char* uuid() const { return _uuid; }
    const char* token() const { return _token; }

    // Device management
    bool registerDevice(JsonDocument& properties, JsonDocument& result);
    bool authenticate(JsonDocument& result);
    bool whoami(JsonDocument& result);
    bool getDevice(const char* uuid, JsonDocument& result);
    bool updateDevice(const char* uuid, JsonDocument& properties, JsonDocument& result);
    bool unregister(const char* uuid);

    // Messaging
    bool sendMessage(const char* targetUuid, JsonDocument& payload);
    bool broadcast(JsonDocument& payload);

    // Status
    bool status(JsonDocument& result);

    // Error info
    int lastHttpStatus() const { return _lastStatus; }
    const char* lastError() const { return _lastError; }

private:
    Client& _client;
    const char* _host;
    uint16_t _port;
    char _uuid[37];
    char _token[65];
    char _authHeader[150];
    int _lastStatus;
    char _lastError[128];

    bool _request(const char* method, const char* path,
                  const char* body, size_t bodyLen,
                  JsonDocument* response);
    void _buildAuthHeader();
    bool _waitForData(unsigned long timeoutMs);

    static void _base64Encode(const char* input, size_t len, char* output);
    static const char _b64chars[];
};

#endif
