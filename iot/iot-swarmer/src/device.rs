
use std::{fmt, collections::HashMap};
use serde::Deserialize;
use thiserror::Error as ThisError;
pub type TelemetryGeneratorType = Box<dyn TelemetryGenerator + Send + Sync>;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("unknown telemetry generator")]
    UnknownTelemetryGeneratorError,
}

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


//////////
// Telemetry Trait and Struct
////

pub trait TelemetryGenerator {
    fn next_datapoint(&mut self) -> f64;
    fn previous_datapoint(&self) -> f64;
}

#[derive(Debug, Clone)]
pub struct LinearTelemetryGenerator {
    // --------
    pub previous_value: f64,
    pub constant: f64,
}

#[derive(Debug, Clone)]
pub struct PyramidTelemetryGenerator {
    // /\/\/\/\
    pub previous_value: f64,
    pub rate: f64,
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Clone)]
pub struct WaveTelemetryGenerator {
    // ////////
    pub previous_value: f64,
    pub rate: f64,
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Clone)]
pub struct RealTelemetryGenerator {
    // ////////
    pub previous_value: f64,
    pub rate: f64,
    pub min: f64,
    pub max: f64,
}

//////////
// Implementation and Factory
////

impl TelemetryGenerator for LinearTelemetryGenerator {
    fn next_datapoint(&mut self) -> f64 {
        self.constant
    }

    fn previous_datapoint(&self) -> f64 {
        self.previous_value
    }
}

impl TelemetryGenerator for WaveTelemetryGenerator {
    fn next_datapoint(&mut self) -> f64 {
        let mut value = self.previous_value + self.rate;
        if value > self.max {
            value = self.min;
        }
        self.previous_value = value;
        value
    }

    fn previous_datapoint(&self) -> f64 {
        self.previous_value
    }
}

impl TelemetryGenerator for PyramidTelemetryGenerator {
    fn next_datapoint(&mut self) -> f64 {
        let value = self.previous_value + self.rate;
        if value >= self.max {
            self.rate = self.rate * -1.0;
        } else if value <= self.min {
            self.rate = self.rate * -1.0;
        }
        self.previous_value = value;
        value
    }

    fn previous_datapoint(&self) -> f64 {
        self.previous_value
    }
}

impl TelemetryGenerator for RealTelemetryGenerator {
    fn next_datapoint(&mut self) -> f64 {
        let value = self.previous_value + self.rate;
        if value > self.max {
            self.rate = self.rate * -1.0;
        } else if value < self.min {
            self.rate = self.rate * -1.0;
        }
        self.previous_value = value;
        value
    }

    fn previous_datapoint(&self) -> f64 {
        self.previous_value
    }
}

impl LinearTelemetryGenerator {
    pub fn new(constant: f64) -> Result<Self, Error> {
        Ok(Self {
            previous_value: constant,
            constant,
        })
    }
}

impl PyramidTelemetryGenerator {
    pub fn new(rate: f64, min: f64, max: f64) -> Result<Self, Error> {
        Ok(Self {
            previous_value: min,
            rate,
            min,
            max,
        })
    }
}

impl WaveTelemetryGenerator {
    pub fn new(rate: f64, min: f64, max: f64) -> Result<Self, Error> {
        Ok(Self {
            previous_value: min,
            rate,
            min,
            max,
        })
    }
}

impl RealTelemetryGenerator {
    pub fn new(rate: f64, min: f64, max: f64) -> Result<Self, Error> {
        Ok(Self {
            previous_value: min,
            rate,
            min,
            max,
        })
    }
}

pub fn get_telemetry_generator_factory(
    generator: &str,
    args: HashMap<String, String>,
) -> Result<TelemetryGeneratorType, Error> {
    return match generator.to_lowercase().trim() {
        "linear" => {
            let constant = args["constant"].parse::<f64>().unwrap();
            Ok(Box::new(LinearTelemetryGenerator::new(constant).unwrap()))
        }
        "pyramid" => {
            let rate = args["rate"].parse::<f64>().unwrap();
            let min = args["min"].parse::<f64>().unwrap();
            let max = args["max"].parse::<f64>().unwrap();
            Ok(Box::new(
                PyramidTelemetryGenerator::new(rate, min, max).unwrap(),
            ))
        }
        "wave" => {
            let rate = args["rate"].parse::<f64>().unwrap();
            let min = args["min"].parse::<f64>().unwrap();
            let max = args["max"].parse::<f64>().unwrap();
            Ok(Box::new(
                WaveTelemetryGenerator::new(rate, min, max).unwrap(),
            ))
        }
        "real" => {
            let rate = args["rate"].parse::<f64>().unwrap();
            let min = args["min"].parse::<f64>().unwrap();
            let max = args["max"].parse::<f64>().unwrap();
            Ok(Box::new(
                RealTelemetryGenerator::new(rate, min, max).unwrap(),
            ))
        }
        _ => Err(Error::UnknownTelemetryGeneratorError),
    };
}
