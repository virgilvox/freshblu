use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

use crate::forwarder::Forwarders;
use crate::permissions::Whitelists;

/// A device in the FreshBlu registry.
/// Every entity - physical device, software service, user, API -
/// is represented as a Device with a UUID and token.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    /// The device's unique identifier
    pub uuid: Uuid,

    /// Bcrypt-hashed token (never returned in plaintext after registration)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,

    /// Whether the device is currently online (has an active connection)
    #[serde(default)]
    pub online: bool,

    /// Device type (arbitrary string, used for search/filter)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "type")]
    pub device_type: Option<String>,

    /// Arbitrary device properties (JSON object)
    #[serde(flatten)]
    pub properties: HashMap<String, Value>,

    /// FreshBlu system metadata
    pub meshblu: MeshbluMeta,
}

/// System-managed metadata block inside every device
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MeshbluMeta {
    pub version: String,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
    /// SHA256 hash of the full device document (for integrity verification)
    pub hash: String,
    /// Permission whitelists (v2.0)
    #[serde(default)]
    pub whitelists: Whitelists,
    /// Forwarder configuration (webhooks, meshblu-to-meshblu)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forwarders: Option<Forwarders>,
    /// Device public key (PEM)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub public_key: Option<String>,
    /// Owner UUID (set when device is claimed)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<Uuid>,
}

impl MeshbluMeta {
    pub fn new(whitelists: Whitelists) -> Self {
        Self {
            version: "2.0.0".to_string(),
            created_at: Utc::now(),
            updated_at: None,
            hash: String::new(),
            whitelists,
            forwarders: None,
            public_key: None,
            owner: None,
        }
    }
}

/// The response returned when registering a new device.
/// Token is only ever returned once - in plaintext - at registration time.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterResponse {
    pub uuid: Uuid,
    pub token: String,
    pub online: bool,
    pub meshblu: MeshbluMeta,
    #[serde(flatten)]
    pub properties: HashMap<String, Value>,
}

/// The response returned for whoami / get device (no token)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceView {
    pub uuid: Uuid,
    pub online: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "type")]
    pub device_type: Option<String>,
    pub meshblu: MeshbluMeta,
    #[serde(flatten)]
    pub properties: HashMap<String, Value>,
}

/// Parameters for registering a new device
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RegisterParams {
    #[serde(rename = "type")]
    pub device_type: Option<String>,
    /// Override the whitelists at registration time
    pub meshblu: Option<WhitelistOverride>,
    /// Arbitrary properties to set at registration
    #[serde(flatten)]
    pub properties: HashMap<String, Value>,
}

#[derive(Debug, Deserialize, Default)]
pub struct WhitelistOverride {
    pub whitelists: Option<crate::permissions::Whitelists>,
}

/// Parameters for updating a device
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateParams {
    pub uuid: Uuid,
    #[serde(flatten)]
    pub properties: HashMap<String, Value>,
}

/// Device search / query parameters
#[derive(Debug, Deserialize, Default)]
pub struct DeviceQuery {
    #[serde(flatten)]
    pub filters: HashMap<String, Value>,
}

impl Device {
    pub fn new(properties: HashMap<String, Value>, whitelists: Whitelists) -> Self {
        let uuid = Uuid::new_v4();
        Self {
            uuid,
            token: None,
            online: false,
            device_type: None,
            properties,
            meshblu: MeshbluMeta::new(whitelists),
        }
    }

    /// Returns true if the '*' wildcard is in the given whitelist, meaning anyone is allowed
    pub fn is_open_whitelist(list: &[WhitelistEntry]) -> bool {
        list.iter().any(|e| e.uuid == "*")
    }

    pub fn to_view(&self) -> DeviceView {
        let mut props = self.properties.clone();
        props.remove("token");
        DeviceView {
            uuid: self.uuid,
            online: self.online,
            device_type: self.device_type.clone(),
            meshblu: self.meshblu.clone(),
            properties: props,
        }
    }
}

/// Entry in a whitelist: a UUID or '*'
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WhitelistEntry {
    pub uuid: String,
}

impl WhitelistEntry {
    pub fn wildcard() -> Self {
        Self {
            uuid: "*".to_string(),
        }
    }
    pub fn for_uuid(uuid: &Uuid) -> Self {
        Self {
            uuid: uuid.to_string(),
        }
    }
    pub fn matches(&self, target: &Uuid) -> bool {
        self.uuid == "*" || self.uuid == target.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_device_new() {
        let props = HashMap::new();
        let whitelists = Whitelists::default();
        let device = Device::new(props, whitelists);

        // UUID should be generated (non-nil)
        assert!(!device.uuid.is_nil());
        // meshblu should be populated
        assert_eq!(device.meshblu.version, "2.0.0");
        assert!(device.token.is_none());
        assert!(!device.online);
    }

    #[test]
    fn test_device_to_view_strips_token() {
        let mut props = HashMap::new();
        props.insert("token".to_string(), Value::String("secret".to_string()));
        props.insert("name".to_string(), Value::String("mydevice".to_string()));

        let device = Device::new(props, Whitelists::default());
        let view = device.to_view();

        // The view should not contain a "token" key in properties
        assert!(!view.properties.contains_key("token"));
        // Other properties should still be present
        assert_eq!(
            view.properties.get("name"),
            Some(&Value::String("mydevice".to_string()))
        );
    }

    #[test]
    fn test_whitelist_entry_matches() {
        let target = Uuid::new_v4();
        let other = Uuid::new_v4();

        // Wildcard matches everything
        let wildcard = WhitelistEntry::wildcard();
        assert!(wildcard.matches(&target));
        assert!(wildcard.matches(&other));

        // Specific entry matches only that UUID
        let specific = WhitelistEntry::for_uuid(&target);
        assert!(specific.matches(&target));
        assert!(!specific.matches(&other));
    }

    #[test]
    fn test_register_params_deserialization() {
        let json = serde_json::json!({
            "type": "device:sensor",
            "name": "temp-sensor"
        });

        let params: RegisterParams = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(params.device_type, Some("device:sensor".to_string()));
        assert_eq!(
            params.properties.get("name"),
            Some(&Value::String("temp-sensor".to_string()))
        );
        assert!(params.meshblu.is_none());

        // Round-trip: serialize back and verify key fields
        let re_json = serde_json::to_value(&json).unwrap();
        assert_eq!(re_json["type"], "device:sensor");
    }
}
