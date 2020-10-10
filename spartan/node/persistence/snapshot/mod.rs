use std::{io::Error as IoError, path::Path};

use bincode::{deserialize, serialize};
use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error as ThisError;
use tokio::fs::{read, write};

use super::PersistenceError;

type Error = PersistenceError<SnapshotError>;

#[derive(ThisError, Debug)]
pub enum SnapshotError {
    #[error("Unable to write serialized database to file: {0}")]
    FileWriteError(IoError),
    #[error("Unable to read serialized database from file: {0}")]
    FileReadError(IoError),
}

pub struct SnapshotConfig<'a> {
    path: &'a Path,
}

pub struct Snapshot<'a> {
    config: SnapshotConfig<'a>,
}

impl<'a> Snapshot<'a> {
    pub fn new(config: SnapshotConfig<'a>) -> Self {
        Snapshot { config }
    }

    pub async fn persist<S>(&self, source: &S, destination: &Path) -> Result<(), Error>
    where
        S: Serialize,
    {
        let path = self.config.path.join(destination);
        write(path, serialize(source).map_err(Error::SerializationError)?)
            .await
            .map_err(|e| Error::DriverError(SnapshotError::FileWriteError(e)))
    }

    pub async fn load<S>(&self, source: &Path) -> Result<S, Error>
    where
        S: DeserializeOwned,
    {
        let path = self.config.path.join(source);
        deserialize(
            &read(path)
                .await
                .map_err(|e| Error::DriverError(SnapshotError::FileReadError(e)))?,
        )
        .map_err(Error::InvalidFileFormat)
    }
}
