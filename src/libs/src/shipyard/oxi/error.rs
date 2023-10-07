use std::{error, fmt};

use crate::clients::amqp::AmqpError;

#[derive(Debug)]
pub enum OxiBuilderError {
    NoMirServer,
    NoDeviceId,
}

impl fmt::Display for OxiBuilderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            OxiBuilderError::NoDeviceId => {
                write!(f, "missing the identifying key for your device")
            }
            OxiBuilderError::NoMirServer => {
                write!(f, "missing the mir server address")
            }
        }
    }
}

impl error::Error for OxiBuilderError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            OxiBuilderError::NoDeviceId => None,
            OxiBuilderError::NoMirServer => None,
        }
    }
}

#[derive(Debug)]
pub enum OxiError {
    // TODO: add address param for display
    CantConnectToMir,
    TelemetrySent,
    DataSent,
    HeathbeatSent,
    ReportedSent,
    Unknown,
    CantRequestDesiredProperties(AmqpError),
}

impl fmt::Display for OxiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OxiError::CantConnectToMir => {
                write!(f, "cant connect to mir server")
            }
            OxiError::Unknown => {
                write!(f, "unkown mir error")
            }
            OxiError::TelemetrySent => {
                write!(f, "error sending telemetry")
            }
            OxiError::DataSent => {
                write!(f, "error sending data")
            }
            OxiError::HeathbeatSent => {
                write!(f, "error sending heartbeat")
            }
            OxiError::ReportedSent => {
                write!(f, "error sending reported properties")
            }
            OxiError::CantRequestDesiredProperties(x) => {
                write!(f, "error sending request for desired properties: {x}")
            }
        }
    }
}

impl error::Error for OxiError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            OxiError::CantConnectToMir => None,
            OxiError::Unknown => None,
            OxiError::TelemetrySent => None,
            OxiError::DataSent => None,
            OxiError::HeathbeatSent => None,
            OxiError::ReportedSent => None,
            OxiError::CantRequestDesiredProperties(_) => None,
        }
    }
}
