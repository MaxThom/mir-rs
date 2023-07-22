use std::{error, fmt};

#[derive(Debug)]
pub enum DizerBuildError {
    NoMirServer,
    NoDeviceId,
}

impl fmt::Display for DizerBuildError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DizerBuildError::NoDeviceId => {
                write!(f, "missing the identifying key for your device")
            }
            DizerBuildError::NoMirServer => {
                write!(f, "missing the mir server address")
            }
        }
    }
}

impl error::Error for DizerBuildError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            DizerBuildError::NoDeviceId => None,
            DizerBuildError::NoMirServer => None,
        }
    }
}

#[derive(Debug)]
pub enum DizerError {
    // TODO: add address param for display
    CantConnectToMir,
}

impl fmt::Display for DizerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DizerError::CantConnectToMir => {
                write!(f, "cant connect to mir server")
            }
        }
    }
}

impl error::Error for DizerError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            DizerError::CantConnectToMir => None,
        }
    }
}
