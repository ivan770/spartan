use std::path::Path;

use bincode::{deserialize, serialize};
use cfg_if::cfg_if;
use serde::{de::DeserializeOwned, Serialize};
use tokio::fs::{create_dir, read, write};

use crate::{config::persistence::PersistenceConfig, node::Queue};

use super::PersistenceError;

const QUEUE_FILE: &str = "queue";

#[cfg(feature = "replication")]
pub(crate) const REPLICATION_FILE: &str = "replication";

/// Snapshot persistence modes
#[derive(Copy, Clone)]
pub enum PersistMode {
    /// Persist whole queue (both DB and replication log)
    Queue,

    /// Persist only replication log
    Replication,
}

pub struct Snapshot<'c> {
    /// Persistence config
    config: &'c PersistenceConfig<'c>,
}

impl<'c> Snapshot<'c> {
    pub fn new(config: &'c PersistenceConfig) -> Self {
        Snapshot { config }
    }

    /// Serialize `source` into `destination`.
    pub(crate) async fn persist<S, P>(
        &self,
        source: &S,
        destination: P,
    ) -> Result<(), PersistenceError>
    where
        S: Serialize,
        P: AsRef<Path>,
    {
        let path = self.config.path.join(destination);

        debug!("Writing to {}", path.display());

        if let Some(parent) = path.parent() {
            if !parent.is_dir() {
                create_dir(parent).await.map_err(PersistenceError::from)?;
            }
        }

        write(
            path,
            serialize(source).map_err(PersistenceError::SerializationError)?,
        )
        .await
        .map_err(PersistenceError::from)
    }

    /// Load serialized database from `source`
    pub(crate) async fn load<S, P>(&self, source: P) -> Result<S, PersistenceError>
    where
        P: AsRef<Path>,
        S: DeserializeOwned,
    {
        let path = self.config.path.join(source);

        debug!("Loading from {}", path.display());

        deserialize(&read(path).await.map_err(PersistenceError::from)?)
            .map_err(PersistenceError::InvalidFileFormat)
    }

    /// Persist queue with provided [`PersistMode`]
    ///
    /// Usually, when using this driver you may prefer [`PersistMode::Queue`],
    /// but if your driver doesn't support replication storage serialization,
    /// then pair it with [`Snapshot`] and choose [`PersistMode::Replication`] mode
    pub async fn persist_queue<P, DB>(
        &self,
        name: P,
        queue: &Queue<DB>,
        mode: PersistMode,
    ) -> Result<(), PersistenceError>
    where
        P: AsRef<Path>,
        DB: Serialize,
    {
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

    /// Deserialize queue from file
    pub async fn load_queue<P, DB>(&self, name: P) -> Result<Queue<DB>, PersistenceError>
    where
        P: AsRef<Path>,
        DB: DeserializeOwned,
    {
        let database = self.load(name.as_ref().join(QUEUE_FILE)).await?;

        cfg_if! {
            if #[cfg(feature = "replication")] {
                let replication_storage = match self.load(name.as_ref().join(REPLICATION_FILE)).await {
                    Ok(storage) => storage,
                    Err(PersistenceError::FileOpenError(e)) => {
                        error!("{}", e);
                        None
                    },
                    Err(e) => return Err(e)
                };

                let queue = Queue::new(database, replication_storage);
            } else {
                let queue = Queue::new(database);
            }
        }

        Ok(queue)
    }
}
