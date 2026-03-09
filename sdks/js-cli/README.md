# freshblu-cli

Command-line interface for the [FreshBlu](https://github.com/virgilvox/freshblu) IoT messaging platform. Meshblu-compatible.

## Install

```bash
npm install -g freshblu-cli
```

## Quick Start

```bash
# Register a device (saves credentials to freshblu.json)
freshblu register --type sensor

# Check who you are
freshblu whoami

# Send a message
freshblu message -d '{"devices":["target-uuid"],"payload":{"temp":22}}'

# Check server status
freshblu status
```

## Configuration

On `register`, credentials are saved to `freshblu.json` in the current directory. All subsequent commands use these credentials automatically.

```bash
# Use a different config file
freshblu -c mydevice.json whoami

# Override credentials per-command
freshblu -U <uuid> -T <token> whoami

# Use a different server
freshblu -S https://api.freshblu.org register --type gateway
```

View current config:

```bash
freshblu config
```

## Commands

### Device Management

```bash
# Register with properties
freshblu register --type sensor
freshblu register -d '{"type":"gateway","name":"hub-1"}'

# Get device info
freshblu whoami
freshblu get <uuid>
freshblu get <uuid> --as <other-uuid>

# Update device
freshblu update -d '{"name":"updated-name"}'
freshblu update <uuid> -d '{"type":"new-type"}'

# Delete device
freshblu unregister
freshblu unregister <uuid>

# Search
freshblu search
freshblu search -d '{"type":"sensor"}'
```

### Messaging

```bash
# Direct message
freshblu message -d '{"devices":["target-uuid"],"payload":{"temp":22}}'

# Broadcast
freshblu message -d '{"devices":["*"],"payload":{"alert":"update available"}}'
```

### Subscriptions

```bash
# Subscribe to another device's broadcasts
freshblu subscribe <emitter-uuid> broadcast.sent

# Subscribe to message events
freshblu subscribe <emitter-uuid> message.sent
```

### Token Management

```bash
# Generate a new token
freshblu token generate
freshblu token generate <uuid>
freshblu token generate --tag "ci-token" --expires-on 1735689600

# Revoke a token
freshblu token revoke <token-string>
freshblu token revoke <uuid> <token-string>
```

## Global Options

| Flag | Description | Default |
|------|-------------|---------|
| `-S, --server <url>` | Server URL | `http://localhost:3000` |
| `-U, --uuid <uuid>` | Device UUID | from config file |
| `-T, --token <token>` | Device token | from config file |
| `-c, --config <path>` | Config file path | `freshblu.json` |
| `-f, --format <format>` | Output format (`json` or `pretty`) | `pretty` |

## Multi-Device Workflow

```bash
# Register primary device
freshblu -c primary.json register --type gateway

# Register a sensor
freshblu -c sensor.json register --type temperature

# Send from sensor to gateway using sensor credentials
freshblu -c sensor.json message -d '{"devices":["<gateway-uuid>"],"payload":{"temp":22}}'
```

## License

MIT
