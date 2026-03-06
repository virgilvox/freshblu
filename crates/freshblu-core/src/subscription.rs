use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// All subscription types in Meshblu v2.0
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SubscriptionType {
    /// Receive broadcast messages sent FROM the emitter
    BroadcastSent,
    /// Receive broadcast messages received BY the emitter
    BroadcastReceived,
    /// Receive config-update events sent FROM the emitter
    ConfigureSent,
    /// Receive config-update events received BY the emitter
    ConfigureReceived,
    /// Receive direct messages sent FROM the emitter
    MessageSent,
    /// Receive direct messages received BY the emitter
    MessageReceived,
    /// Receive unregister events sent FROM the emitter
    UnregisterSent,
    /// Receive unregister events received BY the emitter (the device being unregistered)
    UnregisterReceived,
}

impl std::fmt::Display for SubscriptionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::BroadcastSent => "broadcast.sent",
            Self::BroadcastReceived => "broadcast.received",
            Self::ConfigureSent => "configure.sent",
            Self::ConfigureReceived => "configure.received",
            Self::MessageSent => "message.sent",
            Self::MessageReceived => "message.received",
            Self::UnregisterSent => "unregister.sent",
            Self::UnregisterReceived => "unregister.received",
        };
        write!(f, "{}", s)
    }
}

impl std::str::FromStr for SubscriptionType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "broadcast.sent" => Ok(Self::BroadcastSent),
            "broadcast.received" => Ok(Self::BroadcastReceived),
            "configure.sent" => Ok(Self::ConfigureSent),
            "configure.received" => Ok(Self::ConfigureReceived),
            "message.sent" => Ok(Self::MessageSent),
            "message.received" => Ok(Self::MessageReceived),
            "unregister.sent" => Ok(Self::UnregisterSent),
            "unregister.received" => Ok(Self::UnregisterReceived),
            _ => Err(format!("unknown subscription type: {}", s)),
        }
    }
}

/// A subscription record: subscriber listens to events emitted by/for emitter
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Subscription {
    /// The device whose events we want to receive
    pub emitter_uuid: Uuid,
    /// The device that wants to receive those events
    pub subscriber_uuid: Uuid,
    /// The type of event to subscribe to
    pub subscription_type: SubscriptionType,
}

/// Create subscription params (from API)
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSubscriptionParams {
    pub emitter_uuid: Uuid,
    pub subscriber_uuid: Uuid,
    #[serde(rename = "type")]
    pub subscription_type: SubscriptionType,
}

/// Delete subscription params
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteSubscriptionParams {
    pub emitter_uuid: Option<Uuid>,
    pub subscriber_uuid: Uuid,
    #[serde(rename = "type")]
    pub subscription_type: Option<SubscriptionType>,
}

/// Route hop - tracks the path of a message through subscription chains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteHop {
    pub from: Uuid,
    pub to: Uuid,
    #[serde(rename = "type")]
    pub hop_type: SubscriptionType,
}
