use serde::{Deserialize, Serialize};
use serde_json::Value;

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

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct DeviceTwin {
    pub etag: String,
    pub meta_properties: MetaProperties,
    pub tag_properties: TagProperties,
    pub desired_properties: DesiredProperties,
    pub reported_properties: ReportedProperties,
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
pub struct TagProperties {
    pub properties: Value,
    pub version: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct DesiredProperties {
    pub properties: Value,
    pub version: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ReportedProperties {
    pub properties: Value,
    pub version: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct NewDevice {
    pub device_id: String,
    pub model_id: String,
    pub status: Status,
}
