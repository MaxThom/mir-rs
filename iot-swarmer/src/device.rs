
use std::{fmt, collections::HashMap};
use thiserror::Error as ThisError;
pub type TelemetryGeneratorType = Box<dyn TelemetryGenerator + Send + Sync>;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("unknown telemetry generator")]
    UnknownTelemetryGeneratorError,
}



//////////
// Device Struct
////

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
    fn next_datapoint(&mut self) -> f32;
    fn previous_datapoint(&self) -> f32;
}

#[derive(Debug, Clone)]
pub struct LinearTelemetryGenerator {
    // --------
    pub previous_value: f32,
    pub constant: f32,
}

#[derive(Debug, Clone)]
pub struct PyramidTelemetryGenerator {
    // /\/\/\/\
    pub previous_value: f32,
    pub rate: f32,
    pub min: f32,
    pub max: f32,
}

#[derive(Debug, Clone)]
pub struct WaveTelemetryGenerator {
    // ////////
    pub previous_value: f32,
    pub rate: f32,
    pub min: f32,
    pub max: f32,
}

#[derive(Debug, Clone)]
pub struct RealTelemetryGenerator {
    // ////////
    pub previous_value: f32,
    pub rate: f32,
    pub min: f32,
    pub max: f32,
}

//////////
// Implementation and Factory
////

impl TelemetryGenerator for LinearTelemetryGenerator {
    fn next_datapoint(&mut self) -> f32 {
        self.constant
    }

    fn previous_datapoint(&self) -> f32 {
        self.previous_value
    }
}

impl TelemetryGenerator for WaveTelemetryGenerator {
    fn next_datapoint(&mut self) -> f32 {
        let mut value = self.previous_value + self.rate;
        if value > self.max {
            value = self.min;
        }
        self.previous_value = value;
        value
    }

    fn previous_datapoint(&self) -> f32 {
        self.previous_value
    }
}

impl TelemetryGenerator for PyramidTelemetryGenerator {
    fn next_datapoint(&mut self) -> f32 {
        let value = self.previous_value + self.rate;
        if value >= self.max {
            self.rate = self.rate * -1.0;
        } else if value <= self.min {
            self.rate = self.rate * -1.0;
        }
        self.previous_value = value;
        value
    }

    fn previous_datapoint(&self) -> f32 {
        self.previous_value
    }
}

impl TelemetryGenerator for RealTelemetryGenerator {
    fn next_datapoint(&mut self) -> f32 {
        let value = self.previous_value + self.rate;
        if value > self.max {
            self.rate = self.rate * -1.0;
        } else if value < self.min {
            self.rate = self.rate * -1.0;
        }
        self.previous_value = value;
        value
    }

    fn previous_datapoint(&self) -> f32 {
        self.previous_value
    }
}

impl LinearTelemetryGenerator {
    pub fn new(constant: f32) -> Result<Self, Error> {
        Ok(Self {
            previous_value: constant,
            constant,
        })
    }
}

impl PyramidTelemetryGenerator {
    pub fn new(rate: f32, min: f32, max: f32) -> Result<Self, Error> {
        Ok(Self {
            previous_value: min,
            rate,
            min,
            max,
        })
    }
}

impl WaveTelemetryGenerator {
    pub fn new(rate: f32, min: f32, max: f32) -> Result<Self, Error> {
        Ok(Self {
            previous_value: min,
            rate,
            min,
            max,
        })
    }
}

impl RealTelemetryGenerator {
    pub fn new(rate: f32, min: f32, max: f32) -> Result<Self, Error> {
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
            let constant = args["constant"].parse::<f32>().unwrap();
            Ok(Box::new(LinearTelemetryGenerator::new(constant).unwrap()))
        }
        "pyramid" => {
            let rate = args["rate"].parse::<f32>().unwrap();
            let min = args["min"].parse::<f32>().unwrap();
            let max = args["max"].parse::<f32>().unwrap();
            Ok(Box::new(
                PyramidTelemetryGenerator::new(rate, min, max).unwrap(),
            ))
        }
        "wave" => {
            let rate = args["rate"].parse::<f32>().unwrap();
            let min = args["min"].parse::<f32>().unwrap();
            let max = args["max"].parse::<f32>().unwrap();
            Ok(Box::new(
                WaveTelemetryGenerator::new(rate, min, max).unwrap(),
            ))
        }
        "real" => {
            let rate = args["rate"].parse::<f32>().unwrap();
            let min = args["min"].parse::<f32>().unwrap();
            let max = args["max"].parse::<f32>().unwrap();
            Ok(Box::new(
                RealTelemetryGenerator::new(rate, min, max).unwrap(),
            ))
        }
        _ => Err(Error::UnknownTelemetryGeneratorError),
    };
}
