use std::num::ParseIntError;
use serde::{Serialize, Deserialize};
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum SerializationError {
    #[error("parse int error: {0}")]
    ParseIntError(#[from] ParseIntError),
    #[error("serialization kind unkown")]
    Unkown(),
}

pub enum SerializationKind {
    Json,
    MsgPack,
    Yaml,
}

impl SerializationKind {
    pub fn from_str(kind: &str) -> Result<Self, SerializationError> {
        match kind {
            "json" => Ok(Self::Json),
            "msgpack" => Ok(Self::MsgPack),
            "yaml" => Ok(Self::Yaml),
            _ => Err(SerializationError::Unkown()),
        }
    }

    pub fn to_vec<T: Serialize>(&self, payload: &T) -> Result<Vec<u8>, SerializationError> {
        match self {
            Self::Json => {
                Ok(serde_json::to_vec(payload).unwrap())
            },
            Self::MsgPack => todo!(),
            Self::Yaml => todo!(),
        }
    }
    pub fn from_vec<'a, T: Deserialize<'a>>(&self, payload: &'a Vec<u8>) -> Result<T, SerializationError> {
        match self {
            Self::Json => {
                Ok(serde_json::from_slice(&payload).unwrap())
            },
            Self::MsgPack => todo!(),
            Self::Yaml => todo!(),
        }
    }
}

