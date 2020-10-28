use std::path::Path;

use bincode::{deserialize, serialize};
use cfg_if::cfg_if;
use serde::{de::DeserializeOwned, Serialize};
use tokio::fs::{create_dir, read, write};

use crate::{config::persistence::PersistenceConfig, node::Queue};

use super::PersistenceError;

const QUEUE_FILE: &str = "queue";

#[cfg(feature = "replication")]
pub const REPLICATION_FILE: &str = "replication";

/// Snapshot persistence modes
#[derive(Copy, Clone)]
pub enum PersistMode {
    /// Persist whole queue (both DB and replication log)
    Queue,

    /// Persist only replication log
    Replication
}

pub struct Snapshot<'a> {
    config: &'a PersistenceConfig<'a>,
}

impl<'a> Snapshot<'a> {
    pub fn new(config: &'a PersistenceConfig) -> Self {
        Snapshot { config }
    }

    pub async fn persist<S, P>(&self, source: &S, destination: P) -> Result<(), PersistenceError>
    where
        P: AsRef<Path>,
        S: Serialize,
    {
        let path = self.config.path.join(destination);
        write(
            path,
            serialize(source).map_err(PersistenceError::SerializationError)?,
        )
        .await
        .map_err(PersistenceError::FileWriteError)
    }

    pub async fn load<S, P>(&self, source: P) -> Result<S, PersistenceError>
    where
        P: AsRef<Path>,
        S: DeserializeOwned,
    {
        let path = self.config.path.join(source);
        deserialize(&read(path).await.map_err(PersistenceError::FileReadError)?)
            .map_err(PersistenceError::InvalidFileFormat)
    }

    pub async fn persist_queue<P, DB>(
        &self,
        name: P,
        queue: &Queue<DB>,
        mode: PersistMode
    ) -> Result<(), PersistenceError>
    where
        P: AsRef<Path>,
        DB: Serialize,
    {
        let path = self.config.path.join(&name);
        if !path.is_dir() {
            create_dir(&path)
                .await
                .map_err(PersistenceError::FileWriteError)?;
        }

        if let PersistMode::Queue = mode {
            self.persist(&*queue.database().await, name.as_ref().join(QUEUE_FILE))
                .await?;
        }

        #[cfg(feature = "replication")]
        {
            self.persist(
                &*queue.replication_storage().await,
                name.as_ref().join(REPLICATION_FILE),
            )
            .await?;
        }

        Ok(())
    }

    pub async fn load_queue<P, DB>(&self, name: P) -> Result<Queue<DB>, PersistenceError>
    where
        P: AsRef<Path>,
        DB: DeserializeOwned,
    {
        let database = self.load(name.as_ref().join(QUEUE_FILE)).await?;

        cfg_if! {
            if #[cfg(feature = "replication")] {
                let replication_storage = self.load(name.as_ref().join(REPLICATION_FILE)).await?;
                let queue = Queue::new(database, replication_storage);
            } else {
                let queue = Queue::new(database);
            }
        }

        Ok(queue)
    }
}
