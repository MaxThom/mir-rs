//////////
// Telemetry Trait and Struct
////

use std::collections::HashMap;
use thiserror::Error as ThisError;

pub type TelemetryGeneratorType = Box<dyn TelemetryGenerator + Send + Sync>;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("unknown telemetry generator")]
    UnknownTelemetryGeneratorError,
}

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
        if value <= self.min || value >= self.max {
            self.rate *= -1.0;
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
        if value < self.min || value > self.max {
            self.rate *= -1.0;
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
