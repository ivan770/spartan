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

use std::io::Error as IoError;

use actix_web::ResponseError;
use bincode::Error as BincodeError;
use thiserror::Error as ThisError;

/// Errors, that may occur during persistence process
#[derive(ThisError, Debug)]
pub enum PersistenceError {
    #[error("File in database directory has invalid format: {0}")]
    InvalidFileFormat(BincodeError),
    #[error("Unable to serialize database: {0}")]
    SerializationError(BincodeError),
    #[error("Unable to read entry line from file: {0}")]
    LineReadError(IoError),
    #[error("Unable to write to file: {0}")]
    FileWriteError(IoError),
    #[error("Unable to read from file: {0}")]
    FileReadError(IoError),
    #[error("IO error: {0}")]
    GenericIoError(IoError),
}

impl ResponseError for PersistenceError {}
