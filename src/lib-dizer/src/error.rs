use std::{error, fmt};

#[derive(Debug)]
pub enum DizerBuilderError {
    NoMirServer,
    NoDeviceId,
}

impl fmt::Display for DizerBuilderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DizerBuilderError::NoDeviceId => {
                write!(f, "missing the identifying key for your device")
            }
            DizerBuilderError::NoMirServer => {
                write!(f, "missing the mir server address")
            }
        }
    }
}

impl error::Error for DizerBuilderError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            DizerBuilderError::NoDeviceId => None,
            DizerBuilderError::NoMirServer => None,
        }
    }
}

#[derive(Debug)]
pub enum DizerError {
    // TODO: add address param for display
    CantConnectToMir,
    TelemetrySent,
    Unknown,
}

impl fmt::Display for DizerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DizerError::CantConnectToMir => {
                write!(f, "cant connect to mir server")
            }
            DizerError::Unknown => {
                write!(f, "unkown mir error")
            }
            DizerError::TelemetrySent => {
                write!(f, "error sending telemetry")
            }
        }
    }
}

impl error::Error for DizerError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            DizerError::CantConnectToMir => None,
            DizerError::Unknown => None,
            DizerError::TelemetrySent => None,
        }
    }
}
