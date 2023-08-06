use serde::{Deserialize, Serialize};
use serde_json::Value;
use surrealdb::sql::Thing;

#[derive(Debug, Serialize, Deserialize, Clone, Default, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TargetProperties {
    #[default]
    Desired,
    Reported,
    Tag,
    Meta,
    All,
}

impl TargetProperties {
    pub fn as_device_twin_route(&self) -> &str {
        match self {
            TargetProperties::Desired => "desired_properties",
            TargetProperties::Reported => "reported_properties",
            TargetProperties::Tag => "tag_properties",
            TargetProperties::Meta => "meta_properties",
            TargetProperties::All => "all_properties",
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            TargetProperties::Desired => "desired",
            TargetProperties::Reported => "reported",
            TargetProperties::Tag => "tag",
            TargetProperties::Meta => "meta",
            TargetProperties::All => "all",
        }
    }

    pub fn from_str(s: &str) -> TargetProperties {
        match s {
            "desired" => TargetProperties::Desired,
            "reported" => TargetProperties::Reported,
            "tag" => TargetProperties::Tag,
            "meta" => TargetProperties::Meta,
            "all" => TargetProperties::All,
            _ => TargetProperties::All,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Record {
    id: Thing,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub enum Status {
    #[default]
    Disabled,
    Enabled,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub enum StatusReason {
    #[default]
    Provisioned,
    Registered,
    Blocked,
    Unblocked,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub enum ConnectionState {
    Connected,
    #[default]
    Disconnected,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeviceTwin {
    pub id: Option<Thing>,
    pub meta_properties: Option<MetaProperties>,
    pub tag_properties: Option<Properties>,
    pub desired_properties: Option<Properties>,
    pub reported_properties: Option<Properties>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct MetaProperties {
    pub device_id: String,
    pub model_id: String,
    pub status: Status,
    pub status_reason: StatusReason,
    pub status_update_time: i64,
    pub connection_state: ConnectionState,
    pub last_activity_time: i64,
    pub version: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Properties {
    pub properties: Value,
    pub version: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct NewDevice {
    pub device_id: String,
    pub model_id: String,
    pub status: Status,
}
