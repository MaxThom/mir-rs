use core::fmt::Error;
use std::fmt;

//#[derive(Debug, Clone)]
pub struct Swarm {
    pub devices: Vec<LiveDevice>,
}

impl Swarm {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            devices: Vec::new(),
        })
    }

    pub fn add_device(&mut self, device: LiveDevice) {
        self.devices.push(device);
    }
}

//#[derive(Debug, Clone)]
pub struct LiveDevice {
    pub name: String,
    pub sensors: Vec<LiveSensor>,
}

impl LiveDevice {
    pub fn new(name: String) -> Result<Self, Error> {
        Ok(Self {
            name,
            sensors: Vec::new(),
        })
    }

    pub fn add_sensor(&mut self, sensor: LiveSensor) -> &mut Self {
        self.sensors.push(sensor);
        self
    }
}

pub struct LiveSensor {
    pub name: String,
    pub hysteresis: f32,
    pub telemetry: Box<dyn TelemetryGenerator>,
}

impl fmt::Debug for LiveSensor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Sensor")
         .field("name", &self.name)
         .field("hysteresis", &self.hysteresis)
         .finish()
    }
}

pub trait TelemetryGenerator: {
    fn next_datapoint(&self) -> f32;
    fn previous_datapoint(&self) -> f32;
}

#[derive(Debug, Clone)]
pub struct LinearTelemetryGenerator {  // --------
    pub previous_value: f32,
    pub constant: f32,
}

impl TelemetryGenerator for LinearTelemetryGenerator {
    fn next_datapoint(&self) -> f32 {
        self.constant
    }

    fn previous_datapoint(&self) -> f32 {
        self.previous_value
    }
}

impl LinearTelemetryGenerator {
    pub fn new(constant: f32) -> Result<Self, Error> {
        Ok(Self {
            previous_value: 0.0,
            constant,
        })
    }
}