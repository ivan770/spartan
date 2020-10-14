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

use std::error::Error;

use bincode::Error as BincodeError;
use thiserror::Error as ThisError;

/// Errors, that may occur during persistence process
#[derive(ThisError, Debug)]
pub enum PersistenceError<DE>
where
    DE: Error + 'static,
{
    #[error("File in database directory has invalid format: {0}")]
    InvalidFileFormat(BincodeError),
    #[error("Unable to serialize database: {0}")]
    SerializationError(BincodeError),
    #[error("Persistence driver error: {0}")]
    DriverError(DE),
}
