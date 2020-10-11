use std::{io::Error as IoError, path::Path};

use cfg_if::cfg_if;
use bincode::{deserialize, serialize};
use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error as ThisError;
use tokio::fs::{create_dir, read, write};

use crate::{config::persistence::SnapshotConfig, node::Queue};

use super::PersistenceError;

const QUEUE_FILE: &str = "queue";

#[cfg(feature = "replication")]
const REPLICATION_FILE: &str = "replication";

type Error = PersistenceError<SnapshotError>;

#[derive(ThisError, Debug)]
pub enum SnapshotError {
    #[error("Unable to write serialized database to file: {0}")]
    FileWriteError(IoError),
    #[error("Unable to read serialized database from file: {0}")]
    FileReadError(IoError),
}

pub struct Snapshot<'a> {
    config: &'a SnapshotConfig,
}

impl<'a> Snapshot<'a> {
    pub fn new(config: &'a SnapshotConfig) -> Self {
        Snapshot { config }
    }

    pub async fn persist<S, P>(&self, source: &S, destination: P) -> Result<(), Error>
    where
        P: AsRef<Path>,
        S: Serialize,
    {
        let path = self.config.path.join(destination);
        write(path, serialize(source).map_err(Error::SerializationError)?)
            .await
            .map_err(|e| Error::DriverError(SnapshotError::FileWriteError(e)))
    }

    pub async fn load<S, P>(&self, source: P) -> Result<S, Error>
    where
        P: AsRef<Path>,
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

    pub async fn persist_queue<P, DB>(&self, name: P, queue: &Queue<DB>) -> Result<(), Error>
    where
        P: AsRef<Path>,
        DB: Serialize
    {
        let path = self.config.path.join(&name);
        if !path.is_dir() {
            create_dir(&path)
                .await
                .map_err(|e| PersistenceError::DriverError(SnapshotError::FileWriteError(e)))?;
        }

        self.persist(&*queue.database().await, name.as_ref().join(QUEUE_FILE)).await?;

        #[cfg(feature = "replication")]
        {
            self.persist(&*queue.replication_storage().await, name.as_ref().join(REPLICATION_FILE)).await?;
        }

        Ok(())
    }

    pub async fn load_queue<P, DB>(&self, name: P) -> Result<Queue<DB>, Error>
    where
        P: AsRef<Path>,
        DB: DeserializeOwned
    {
        let database = self.load(name.as_ref().join(QUEUE_FILE)).await?;

        cfg_if! {
            if #[cfg(feature = "replication")] {
                let replication_storage = self.load(name.as_ref().join(REPLICATION_FILE)).await?;
                let queue = Queue::new(database, replication_storage, None);
            } else {
                let queue = Queue::new(database, None);
            }
        }

        Ok(queue)
    }
}
