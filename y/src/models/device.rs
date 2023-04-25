use std::collections::HashMap;

use serde::Serialize;


#[derive(Debug, Serialize, Clone, Default)]
pub struct DevicePayload {
    pub device_id: String,
    pub timestamp: String,
    pub payload: HashMap<String, f32>,
}