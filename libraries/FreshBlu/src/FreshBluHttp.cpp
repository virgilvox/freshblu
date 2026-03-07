#include "FreshBluHttp.h"

const char FreshBluHttp::_b64chars[] =
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

FreshBluHttp::FreshBluHttp(Client& client, const char* host, uint16_t port)
    : _client(client), _host(host), _port(port), _lastStatus(0) {
    _uuid[0] = '\0';
    _token[0] = '\0';
    _authHeader[0] = '\0';
    _lastError[0] = '\0';
}

void FreshBluHttp::setAuth(const char* uuid, const char* token) {
    strncpy(_uuid, uuid, sizeof(_uuid) - 1);
    _uuid[sizeof(_uuid) - 1] = '\0';
    strncpy(_token, token, sizeof(_token) - 1);
    _token[sizeof(_token) - 1] = '\0';
    _buildAuthHeader();
}

void FreshBluHttp::_buildAuthHeader() {
    // Build "uuid:token" then Base64 encode it
    char plain[102]; // 36 + 1 + 64 + 1
    snprintf(plain, sizeof(plain), "%s:%s", _uuid, _token);
    size_t plainLen = strlen(plain);

    char encoded[140];
    _base64Encode(plain, plainLen, encoded);

    snprintf(_authHeader, sizeof(_authHeader), "Basic %s", encoded);
}

void FreshBluHttp::_base64Encode(const char* input, size_t len, char* output) {
    size_t i = 0, j = 0;
    uint8_t a3[3];
    uint8_t a4[4];

    while (len--) {
        a3[i++] = *(input++);
        if (i == 3) {
            a4[0] = (a3[0] & 0xfc) >> 2;
            a4[1] = ((a3[0] & 0x03) << 4) | ((a3[1] & 0xf0) >> 4);
            a4[2] = ((a3[1] & 0x0f) << 2) | ((a3[2] & 0xc0) >> 6);
            a4[3] = a3[2] & 0x3f;
            for (i = 0; i < 4; i++)
                output[j++] = _b64chars[a4[i]];
            i = 0;
        }
    }

    if (i) {
        for (size_t k = i; k < 3; k++)
            a3[k] = '\0';

        a4[0] = (a3[0] & 0xfc) >> 2;
        a4[1] = ((a3[0] & 0x03) << 4) | ((a3[1] & 0xf0) >> 4);
        a4[2] = ((a3[1] & 0x0f) << 2) | ((a3[2] & 0xc0) >> 6);

        for (size_t k = 0; k < i + 1; k++)
            output[j++] = _b64chars[a4[k]];

        while (i++ < 3)
            output[j++] = '=';
    }

    output[j] = '\0';
}

bool FreshBluHttp::_waitForData(unsigned long timeoutMs) {
    unsigned long deadline = millis() + timeoutMs;
    while (!_client.available()) {
        if (millis() > deadline) return false;
        if (!_client.connected()) return false;
        delay(1);
    }
    return true;
}

bool FreshBluHttp::_request(const char* method, const char* path,
                            const char* body, size_t bodyLen,
                            JsonDocument* response) {
    _lastStatus = 0;
    _lastError[0] = '\0';

    if (!_client.connect(_host, _port)) {
        strncpy(_lastError, "Connection failed", sizeof(_lastError));
        return false;
    }

    // Request line
    _client.print(method);
    _client.print(' ');
    _client.print(path);
    _client.println(" HTTP/1.1");

    // Headers
    _client.print("Host: ");
    _client.println(_host);
    _client.println("Connection: close");

    if (_authHeader[0] != '\0') {
        _client.print("Authorization: ");
        _client.println(_authHeader);
    }

    if (body && bodyLen > 0) {
        _client.println("Content-Type: application/json");
        _client.print("Content-Length: ");
        _client.println(bodyLen);
    }

    _client.println(); // End headers

    if (body && bodyLen > 0) {
        _client.write((const uint8_t*)body, bodyLen);
    }

    // Wait for response
    if (!_waitForData(10000)) {
        strncpy(_lastError, "Response timeout", sizeof(_lastError));
        _client.stop();
        return false;
    }

    // Parse status code from "HTTP/1.1 200 OK"
    char statusLine[64];
    size_t sl = 0;
    while (_client.available() && sl < sizeof(statusLine) - 1) {
        char c = _client.read();
        if (c == '\n') break;
        if (c != '\r') statusLine[sl++] = c;
    }
    statusLine[sl] = '\0';

    char* codeStart = strchr(statusLine, ' ');
    if (codeStart) {
        _lastStatus = atoi(codeStart + 1);
    }

    // Skip headers — find the blank line (\r\n\r\n)
    // Must wait for data between reads in case headers arrive in chunks
    int consecutiveNewlines = 0;
    unsigned long headerDeadline = millis() + 5000;
    while (consecutiveNewlines < 2) {
        if (_client.available()) {
            char c = _client.read();
            if (c == '\n') {
                consecutiveNewlines++;
            } else if (c != '\r') {
                consecutiveNewlines = 0;
            }
        } else if (!_client.connected() || millis() > headerDeadline) {
            break;
        } else {
            delay(1);
        }
    }

    // Read body
    char bodyBuf[FRESHBLU_MAX_RESPONSE];
    size_t bi = 0;
    unsigned long bodyDeadline = millis() + 5000;
    while (bi < sizeof(bodyBuf) - 1) {
        if (_client.available()) {
            bodyBuf[bi++] = _client.read();
            bodyDeadline = millis() + 1000; // reset on data
        } else if (!_client.connected() || millis() > bodyDeadline) {
            break;
        } else {
            delay(1);
        }
    }
    bodyBuf[bi] = '\0';
    _client.stop();

    if (_lastStatus < 200 || _lastStatus >= 300) {
        // Try to extract error message from JSON
        JsonDocument errDoc;
        if (deserializeJson(errDoc, bodyBuf) == DeserializationError::Ok) {
            const char* err = errDoc["error"];
            if (err) {
                strncpy(_lastError, err, sizeof(_lastError) - 1);
                _lastError[sizeof(_lastError) - 1] = '\0';
            }
        }
        if (_lastError[0] == '\0') {
            snprintf(_lastError, sizeof(_lastError), "HTTP %d", _lastStatus);
        }
        return false;
    }

    if (response && bi > 0) {
        DeserializationError err = deserializeJson(*response, bodyBuf);
        if (err) {
            snprintf(_lastError, sizeof(_lastError), "JSON parse: %s", err.c_str());
            return false;
        }
    }

    return true;
}

bool FreshBluHttp::registerDevice(JsonDocument& properties, JsonDocument& result) {
    char body[512];
    size_t len = serializeJson(properties, body, sizeof(body));
    if (len >= sizeof(body)) {
        strncpy(_lastError, "Properties too large", sizeof(_lastError));
        return false;
    }

    if (!_request("POST", "/devices", body, len, &result)) {
        return false;
    }

    // Auto-set credentials from response
    const char* uuid = result["uuid"];
    const char* token = result["token"];
    if (uuid && token) {
        setAuth(uuid, token);
    }

    return true;
}

bool FreshBluHttp::authenticate(JsonDocument& result) {
    char body[128];
    int len = snprintf(body, sizeof(body),
             "{\"uuid\":\"%s\",\"token\":\"%s\"}", _uuid, _token);
    if (len < 0 || (size_t)len >= sizeof(body)) {
        strncpy(_lastError, "Credentials too large", sizeof(_lastError));
        return false;
    }

    // authenticate endpoint reads creds from body, not auth header
    char savedAuth[150];
    memcpy(savedAuth, _authHeader, sizeof(_authHeader));
    _authHeader[0] = '\0';
    bool ok = _request("POST", "/authenticate", body, strlen(body), &result);
    memcpy(_authHeader, savedAuth, sizeof(_authHeader));
    return ok;
}

bool FreshBluHttp::whoami(JsonDocument& result) {
    return _request("GET", "/whoami", nullptr, 0, &result);
}

bool FreshBluHttp::getDevice(const char* uuid, JsonDocument& result) {
    char path[64];
    snprintf(path, sizeof(path), "/devices/%s", uuid);

    return _request("GET", path, nullptr, 0, &result);
}

bool FreshBluHttp::updateDevice(const char* uuid, JsonDocument& properties,
                                 JsonDocument& result) {
    char path[64];
    snprintf(path, sizeof(path), "/devices/%s", uuid);

    char body[512];
    size_t len = serializeJson(properties, body, sizeof(body));
    if (len >= sizeof(body)) {
        strncpy(_lastError, "Properties too large", sizeof(_lastError));
        return false;
    }

    return _request("PUT", path, body, len, &result);
}

bool FreshBluHttp::unregister(const char* uuid) {
    char path[64];
    snprintf(path, sizeof(path), "/devices/%s", uuid);

    return _request("DELETE", path, nullptr, 0, nullptr);
}

bool FreshBluHttp::sendMessage(const char* targetUuid, JsonDocument& payload) {
    JsonDocument msg;
    msg["devices"][0] = targetUuid;
    msg["payload"] = payload;

    char body[512];
    size_t len = serializeJson(msg, body, sizeof(body));
    if (len >= sizeof(body)) {
        strncpy(_lastError, "Message too large", sizeof(_lastError));
        return false;
    }

    return _request("POST", "/messages", body, len, nullptr);
}

bool FreshBluHttp::broadcast(JsonDocument& payload) {
    JsonDocument msg;
    msg["devices"][0] = "*";
    msg["payload"] = payload;

    char body[512];
    size_t len = serializeJson(msg, body, sizeof(body));
    if (len >= sizeof(body)) {
        strncpy(_lastError, "Message too large", sizeof(_lastError));
        return false;
    }

    return _request("POST", "/messages", body, len, nullptr);
}

bool FreshBluHttp::status(JsonDocument& result) {
    return _request("GET", "/status", nullptr, 0, &result);
}
