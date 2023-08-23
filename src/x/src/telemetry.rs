use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// TODO: Payload is a Generic so user can send whatever
//       And they should not care about metadata of the payload
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct DeviceTelemetryRequest {
    pub device_id: String,
    pub timestamp: i64,
    pub telemetry: Telemetry,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Telemetry {
    pub floats: HashMap<i64, f64>,
    pub ints: HashMap<i64, i64>,
    pub bools: HashMap<i64, bool>,
    pub strings: HashMap<i64, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct DeviceHeartbeatRequest {
    pub device_id: String,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct DeviceDesiredRequest {
    pub device_id: String,
    pub timestamp: i64,
}
