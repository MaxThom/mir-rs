use serde::Deserialize;
use std::{collections::HashMap, fmt};
use thiserror::Error as ThisError;
use y::utils::telemetry::{get_telemetry_generator_factory, Error, TelemetryGeneratorType};

#[derive(Debug, Deserialize, Clone)]
pub struct Sensor {
    pub id: i64,
    pub name: String,
    pub hysteresis: f64,
    pub pattern_name: String,
    pub pattern_args: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Device {
    pub name: String,
    pub count: u32,
    pub send_interval_second: u32,
    pub sensors: Vec<Sensor>,
}

//////////
// Device Struct
////

pub struct LiveDevice {
    pub id: i64,
    pub name: String,
    pub sensors: Vec<LiveSensor>,
}

impl LiveDevice {
    pub fn new(id: i64, name: String) -> Result<Self, Error> {
        Ok(Self {
            id,
            name,
            sensors: Vec::new(),
        })
    }

    pub fn from_template(template: &Device, index: u32, id: i64) -> Result<Self, Error> {
        let mut device = LiveDevice::new(id, format!("{}-{}", template.name, index)).unwrap();
        for sensor in &template.sensors {
            device.add_sensor(LiveSensor {
                id: sensor.id,
                name: sensor.name.clone(),
                hysteresis: sensor.hysteresis,
                telemetry: get_telemetry_generator_factory(
                    sensor.pattern_name.as_str(),
                    sensor.pattern_args.clone(),
                )
                .unwrap(),
            });
        }
        Ok(device)
    }

    pub fn add_sensor(&mut self, sensor: LiveSensor) -> &mut Self {
        self.sensors.push(sensor);
        self
    }
}

pub struct LiveSensor {
    pub id: i64,
    pub name: String,
    pub hysteresis: f64,
    pub telemetry: TelemetryGeneratorType,
}

impl fmt::Debug for LiveSensor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Sensor")
            .field("name", &self.name)
            .field("hysteresis", &self.hysteresis)
            .finish()
    }
}
