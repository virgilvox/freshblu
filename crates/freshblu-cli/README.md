# freshblu-cli

Command-line client for interacting with a FreshBlu server.

## Installation

```bash
cargo install freshblu-cli
```

## Usage

```bash
# Server defaults to http://localhost:3000
freshblu --server http://my-server:3000 <command>

# Register a device (saves credentials to ~/.freshblu/credentials.toml)
freshblu register
freshblu register -d '{"type":"sensor","location":"lab"}'

# Device operations
freshblu whoami
freshblu get <uuid>
freshblu update <uuid> -d '{"firmware":"2.0"}'
freshblu unregister <uuid>

# Messaging
freshblu message -d '{"devices":["*"],"payload":{"temp":72}}'
freshblu message -d '{"devices":["<uuid>"],"topic":"alert","payload":"fire"}'

# Subscriptions
freshblu subscribe <emitter-uuid> broadcast.sent
freshblu subscribe <emitter-uuid> message.received

# Token management
freshblu token generate
freshblu token generate --expires-on 1735689600 --tag session
freshblu token revoke <token>

# Server status
freshblu status
```

## License

MIT OR Apache-2.0
