# freshblu-core

Core types and permission model for the FreshBlu IoT messaging platform.

## Overview

This crate provides the foundational types used across all FreshBlu components:

- **Device** -- UUID-identified entities with arbitrary JSON properties
- **Message** -- Messages routed between devices
- **Permissions** -- Meshblu v2.0 whitelist-based permission system
- **Subscriptions** -- Event subscription types for pub/sub routing
- **Tokens** -- Bcrypt-hashed authentication tokens
- **Auth** -- Token generation, hashing, and Basic Auth parsing

## Permission Model

Every device has a `meshblu.whitelists` block controlling access:

```rust
use freshblu_core::permissions::{Whitelists, PermissionChecker};

// Open whitelists (anyone can do anything)
let open = Whitelists::open();

// Private whitelists (only owner has access)
let private = Whitelists::private(&owner_uuid);

// Check permissions
let checker = PermissionChecker::new(&device.meshblu.whitelists, &actor_uuid, &device_uuid);
assert!(checker.can_discover_view());
assert!(checker.can_message_from());
```

## Usage

```rust
use freshblu_core::device::{Device, RegisterParams};
use freshblu_core::message::{Message, SendMessageParams};
use freshblu_core::permissions::Whitelists;

let device = Device::new(Default::default(), Whitelists::open());
let view = device.to_view(); // strips sensitive fields
```

## License

MIT OR Apache-2.0
