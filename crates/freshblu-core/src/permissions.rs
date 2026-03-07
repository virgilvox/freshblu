/// Meshblu v2.0 Permission System
///
/// Every device has a `meshblu.whitelists` block that controls who can do what.
/// Permissions are broken into categories (discover, configure, message, broadcast)
/// each with sub-types (sent, received, as, update, view, from).
///
/// The special UUID "*" in any whitelist means "anyone is allowed".
/// An empty whitelist means "nobody except self is allowed".
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::device::WhitelistEntry;

/// The full permission whitelist structure for a device (v2.0)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Whitelists {
    #[serde(default)]
    pub discover: DiscoverWhitelist,
    #[serde(default)]
    pub configure: ConfigureWhitelist,
    #[serde(default)]
    pub message: MessageWhitelist,
    #[serde(default)]
    pub broadcast: BroadcastWhitelist,
}

/// Controls who can discover / view this device
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct DiscoverWhitelist {
    /// Who can GET this device's properties
    #[serde(default)]
    pub view: Vec<WhitelistEntry>,
    /// Who can act as this device for discovery purposes
    #[serde(default)]
    pub r#as: Vec<WhitelistEntry>,
}

/// Controls who can modify this device's config
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ConfigureWhitelist {
    /// Who can update this device
    #[serde(default)]
    pub update: Vec<WhitelistEntry>,
    /// Who can receive config-change events this device emits
    #[serde(default)]
    pub sent: Vec<WhitelistEntry>,
    /// Who can receive config-change events sent TO this device
    #[serde(default)]
    pub received: Vec<WhitelistEntry>,
    /// Who can act as this device for configure operations
    #[serde(default)]
    pub r#as: Vec<WhitelistEntry>,
}

/// Controls who can send/receive direct messages
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct MessageWhitelist {
    /// Who can send messages TO this device
    #[serde(default)]
    pub from: Vec<WhitelistEntry>,
    /// Who can receive messages SENT by this device
    #[serde(default)]
    pub sent: Vec<WhitelistEntry>,
    /// Who can receive messages RECEIVED by this device
    #[serde(default)]
    pub received: Vec<WhitelistEntry>,
    /// Who can act as this device for messaging
    #[serde(default)]
    pub r#as: Vec<WhitelistEntry>,
}

/// Controls who can subscribe to broadcast events
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct BroadcastWhitelist {
    /// Who can subscribe to broadcasts SENT by this device
    #[serde(default)]
    pub sent: Vec<WhitelistEntry>,
    /// Who can subscribe to broadcasts RECEIVED by this device
    #[serde(default)]
    pub received: Vec<WhitelistEntry>,
    /// Who can act as this device for broadcasting
    #[serde(default)]
    pub r#as: Vec<WhitelistEntry>,
}

/// Open (public) whitelists - everyone can do everything
/// Used for newly registered devices in dev/open mode
impl Whitelists {
    pub fn open() -> Self {
        let all = vec![WhitelistEntry::wildcard()];
        Self {
            discover: DiscoverWhitelist {
                view: all.clone(),
                r#as: vec![],
            },
            configure: ConfigureWhitelist {
                update: all.clone(),
                sent: all.clone(),
                received: all.clone(),
                r#as: vec![],
            },
            message: MessageWhitelist {
                from: all.clone(),
                sent: all.clone(),
                received: all.clone(),
                r#as: vec![],
            },
            broadcast: BroadcastWhitelist {
                sent: all.clone(),
                received: all.clone(),
                r#as: vec![],
            },
        }
    }

    /// Locked-down defaults: only self has access
    pub fn private(owner: &Uuid) -> Self {
        let me = vec![WhitelistEntry::for_uuid(owner)];
        Self {
            discover: DiscoverWhitelist {
                view: me.clone(),
                r#as: vec![],
            },
            configure: ConfigureWhitelist {
                update: me.clone(),
                sent: me.clone(),
                received: me.clone(),
                r#as: vec![],
            },
            message: MessageWhitelist {
                from: me.clone(),
                sent: me.clone(),
                received: me.clone(),
                r#as: vec![],
            },
            broadcast: BroadcastWhitelist {
                sent: me.clone(),
                received: me.clone(),
                r#as: vec![],
            },
        }
    }
}

/// Check if an actor UUID is in a whitelist
pub fn check_whitelist(list: &[WhitelistEntry], actor: &Uuid) -> bool {
    list.iter().any(|e| e.matches(actor))
}

/// The complete set of permission check operations
pub struct PermissionChecker<'a> {
    pub device: &'a Whitelists,
    pub actor: &'a Uuid,
    pub device_uuid: &'a Uuid,
}

impl<'a> PermissionChecker<'a> {
    pub fn new(device: &'a Whitelists, actor: &'a Uuid, device_uuid: &'a Uuid) -> Self {
        Self { device, actor, device_uuid }
    }

    /// Is actor the device itself?
    fn is_self(&self) -> bool {
        self.actor == self.device_uuid
    }

    /// Can actor view this device's properties?
    pub fn can_discover_view(&self) -> bool {
        self.is_self() || check_whitelist(&self.device.discover.view, self.actor)
    }

    /// Can actor act-as this device for discover operations?
    pub fn can_discover_as(&self) -> bool {
        self.is_self() || check_whitelist(&self.device.discover.r#as, self.actor)
    }

    /// Can actor update this device's config?
    pub fn can_configure_update(&self) -> bool {
        self.is_self() || check_whitelist(&self.device.configure.update, self.actor)
    }

    /// Can actor receive configure.sent events from this device?
    pub fn can_configure_sent(&self) -> bool {
        self.is_self() || check_whitelist(&self.device.configure.sent, self.actor)
    }

    /// Can actor receive configure.received events for this device?
    pub fn can_configure_received(&self) -> bool {
        self.is_self() || check_whitelist(&self.device.configure.received, self.actor)
    }

    /// Can actor act-as this device for configure operations?
    pub fn can_configure_as(&self) -> bool {
        self.is_self() || check_whitelist(&self.device.configure.r#as, self.actor)
    }

    /// Can actor send messages TO this device?
    pub fn can_message_from(&self) -> bool {
        self.is_self() || check_whitelist(&self.device.message.from, self.actor)
    }

    /// Can actor receive messages.sent from this device?
    pub fn can_message_sent(&self) -> bool {
        self.is_self() || check_whitelist(&self.device.message.sent, self.actor)
    }

    /// Can actor receive messages.received by this device?
    pub fn can_message_received(&self) -> bool {
        self.is_self() || check_whitelist(&self.device.message.received, self.actor)
    }

    /// Can actor act-as this device for messaging?
    pub fn can_message_as(&self) -> bool {
        self.is_self() || check_whitelist(&self.device.message.r#as, self.actor)
    }

    /// Can actor subscribe to broadcast.sent events from this device?
    pub fn can_broadcast_sent(&self) -> bool {
        self.is_self() || check_whitelist(&self.device.broadcast.sent, self.actor)
    }

    /// Can actor subscribe to broadcast.received events for this device?
    pub fn can_broadcast_received(&self) -> bool {
        self.is_self() || check_whitelist(&self.device.broadcast.received, self.actor)
    }

    /// Can actor act-as this device for broadcasts?
    pub fn can_broadcast_as(&self) -> bool {
        self.is_self() || check_whitelist(&self.device.broadcast.r#as, self.actor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device::WhitelistEntry;

    #[test]
    fn test_check_whitelist_wildcard() {
        let list = vec![WhitelistEntry::wildcard()];
        let any_uuid = Uuid::new_v4();
        assert!(check_whitelist(&list, &any_uuid));
    }

    #[test]
    fn test_check_whitelist_specific_uuid() {
        let target = Uuid::new_v4();
        let other = Uuid::new_v4();
        let list = vec![WhitelistEntry::for_uuid(&target)];
        assert!(check_whitelist(&list, &target));
        assert!(!check_whitelist(&list, &other));
    }

    #[test]
    fn test_check_whitelist_empty() {
        let list: Vec<WhitelistEntry> = vec![];
        let any_uuid = Uuid::new_v4();
        assert!(!check_whitelist(&list, &any_uuid));
    }

    #[test]
    fn test_permission_checker_self_always_allowed() {
        let device_uuid = Uuid::new_v4();
        let whitelists = Whitelists::default(); // empty whitelists
        let checker = PermissionChecker::new(&whitelists, &device_uuid, &device_uuid);

        assert!(checker.can_discover_view());
        assert!(checker.can_configure_update());
        assert!(checker.can_message_from());
        assert!(checker.can_broadcast_sent());
    }

    #[test]
    fn test_permission_checker_non_self_needs_whitelist() {
        let device_uuid = Uuid::new_v4();
        let actor = Uuid::new_v4();
        let whitelists = Whitelists::default(); // empty whitelists
        let checker = PermissionChecker::new(&whitelists, &actor, &device_uuid);

        assert!(!checker.can_discover_view());
        assert!(!checker.can_configure_update());
        assert!(!checker.can_message_from());
        assert!(!checker.can_broadcast_sent());
    }

    #[test]
    fn test_whitelists_open_allows_all() {
        let device_uuid = Uuid::new_v4();
        let random_actor = Uuid::new_v4();
        let whitelists = Whitelists::open();
        let checker = PermissionChecker::new(&whitelists, &random_actor, &device_uuid);

        assert!(checker.can_discover_view());
        assert!(checker.can_configure_update());
        assert!(checker.can_configure_sent());
        assert!(checker.can_configure_received());
        assert!(checker.can_message_from());
        assert!(checker.can_message_sent());
        assert!(checker.can_message_received());
        assert!(checker.can_broadcast_sent());
        assert!(checker.can_broadcast_received());
    }

    #[test]
    fn test_whitelists_private_restricts() {
        let owner = Uuid::new_v4();
        let stranger = Uuid::new_v4();
        let whitelists = Whitelists::private(&owner);

        // Owner (not self, but in whitelist) can access
        let device_uuid = Uuid::new_v4();
        let checker_owner = PermissionChecker::new(&whitelists, &owner, &device_uuid);
        assert!(checker_owner.can_discover_view());
        assert!(checker_owner.can_configure_update());
        assert!(checker_owner.can_message_from());

        // Stranger cannot access
        let device_uuid2 = Uuid::new_v4();
        let checker_stranger = PermissionChecker::new(&whitelists, &stranger, &device_uuid2);
        assert!(!checker_stranger.can_discover_view());
        assert!(!checker_stranger.can_configure_update());
        assert!(!checker_stranger.can_message_from());
    }
}
