use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

impl MeshbluMeta {
    pub fn new(whitelists: Whitelists) -> Self {
        Self {
            version: "2.0.0".to_string(),
            created_at: Utc::now(),
            updated_at: None,
            hash: String::new(),
            whitelists,
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
        Self { uuid: "*".to_string() }
    }
    pub fn for_uuid(uuid: &Uuid) -> Self {
        Self { uuid: uuid.to_string() }
    }
    pub fn matches(&self, target: &Uuid) -> bool {
        self.uuid == "*" || self.uuid == target.to_string()
    }
}
