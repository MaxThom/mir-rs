use std::num::ParseIntError;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum NetworkError {
    #[error("parse int error: {0}")]
    ParseIntError(#[from] ParseIntError),
}

pub fn parse_host_port(host: &str) -> Result<(String, u16), NetworkError> {
    let mut parts = host.split(':');
    let host = parts.next().unwrap_or("");
    let port = parts.next().unwrap_or("");
    let port = port.parse::<u16>()?;
    Ok((host.to_string(), port))
}