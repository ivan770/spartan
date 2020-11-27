/// Log-based persistence
///
/// Saves each queue action individually.
/// Provides better reliability by writing at every request.
/// This may hurt performance a bit.
pub mod log;

/// Snapshot-based persistence
///
/// Saves whole database periodically.
/// Best performance, yet worse reliability.
pub mod snapshot;

use std::{
    io::{Error as IoError, ErrorKind},
    num::TryFromIntError,
};

use actix_web::ResponseError;
use bincode::Error as BincodeError;
use thiserror::Error;

/// Errors, that may occur during persistence process
#[derive(Error, Debug)]
pub enum PersistenceError {
    #[error("File in database directory has invalid format: {0}")]
    InvalidFileFormat(BincodeError),
    #[error("Unable to serialize database: {0}")]
    SerializationError(BincodeError),
    #[error("Log entry size is too big for current platform")]
    LogEntryTooBig(TryFromIntError),
    #[error("Unable to read database file: {0}")]
    FileOpenError(IoError),
    #[error("IO error: {0}")]
    GenericIoError(IoError),
}

impl From<IoError> for PersistenceError {
    fn from(error: IoError) -> Self {
        match error.kind() {
            ErrorKind::NotFound => PersistenceError::FileOpenError(error),
            _ => PersistenceError::GenericIoError(error),
        }
    }
}

impl ResponseError for PersistenceError {}
