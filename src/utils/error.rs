use std::{env::VarError, string::FromUtf8Error};

use crate::migration::DbErr;
use base64::DecodeError;
use log::SetLoggerError;
use log4rs::config::runtime::ConfigErrors;
use ring::error::{KeyRejected, Unspecified};
use tokio::task::JoinError;

#[derive(Debug)]
pub enum Error {
    Database(DbErr),
    File(std::io::Error),
    Generic(Box<dyn std::error::Error + Send + Sync>),
    Reqwest(reqwest::Error),
    Config(ConfigErrors),
    Var(VarError),
    Logger(SetLoggerError),
    TokioJoinError(JoinError),
    UnspecifiedRingError(Unspecified),
    KeyRejectedError(KeyRejected),
    FromUtf8(FromUtf8Error),
    BaseDecode(DecodeError),
    SerdeJson(serde_json::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Database(ref err) => {
                write!(f, "Database error: {}", err)
            }
            Self::File(ref err) => {
                write!(f, "File error: {}", err)
            }
            Self::Generic(ref err) => {
                write!(f, "Other error: {}", err)
            }
            Self::Reqwest(ref err) => {
                write!(f, "Reqwest error: {}", err)
            }
            Self::Config(ref err) => {
                write!(f, "Config error: {}", err)
            }
            Self::Var(ref err) => {
                write!(f, "Variable error: {}", err)
            }
            Self::Logger(ref err) => {
                write!(f, "Logger error: {}", err)
            }
            Self::TokioJoinError(ref err) => {
                write!(f, "Tokio JoinError: {}", err)
            }
            Self::UnspecifiedRingError(ref err) => {
                write!(f, "Ring error: {}", err)
            }
            Self::KeyRejectedError(ref err) => {
                write!(f, "Key rejected: {}", err)
            }
            Self::FromUtf8(ref err) => {
                write!(f, "Error parsing utf-8: {}", err)
            }
            Self::BaseDecode(ref err) => {
                write!(f, "Error decoding bytes to base64: {}", err)
            }
            Self::SerdeJson(ref err) => {
                write!(f, "Error decoding json: {}", err)
            }
        }
    }
}

impl From<DbErr> for Error {
    fn from(err: DbErr) -> Self {
        Self::Database(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::File(err)
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for Error {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self::Generic(err)
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Self::Reqwest(err)
    }
}

impl From<ConfigErrors> for Error {
    fn from(err: ConfigErrors) -> Self {
        Self::Config(err)
    }
}

impl From<VarError> for Error {
    fn from(err: VarError) -> Self {
        Self::Var(err)
    }
}

impl From<SetLoggerError> for Error {
    fn from(err: SetLoggerError) -> Self {
        Self::Logger(err)
    }
}

impl From<JoinError> for Error {
    fn from(err: JoinError) -> Self {
        Self::TokioJoinError(err)
    }
}

impl From<Unspecified> for Error {
    fn from(err: Unspecified) -> Self {
        Self::UnspecifiedRingError(err)
    }
}

impl From<KeyRejected> for Error {
    fn from(err: KeyRejected) -> Self {
        Self::KeyRejectedError(err)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(err: FromUtf8Error) -> Self {
        Self::FromUtf8(err)
    }
}

impl From<DecodeError> for Error {
    fn from(err: DecodeError) -> Self {
        Self::BaseDecode(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::SerdeJson(err)
    }
}
