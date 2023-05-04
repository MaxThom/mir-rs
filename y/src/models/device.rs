use std::collections::HashMap;

use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct DevicePayload {
    pub device_id: i64,
    pub timestamp: i64,
    pub payload: HashMap<i64, f64>,
}