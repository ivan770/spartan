use crate::node::{Manager, Queue, DB};
use actix_rt::time::delay_for;
use bincode::{deserialize, serialize, Error as BincodeError};
use cfg_if::cfg_if;
use futures_util::stream::{iter, StreamExt};
use serde::de::DeserializeOwned;
use std::{io::Error, path::Path, time::Duration};
use thiserror::Error as ThisError;
use tokio::fs::{create_dir, read, write};

const QUEUE_FILE: &str = "queue";
#[cfg(feature = "replication")]
const REPLICATION_FILE: &str = "replication";

/// Enum of errors, that may occur during persistence jobs
#[derive(ThisError, Debug)]
pub enum PersistenceError {
    #[error("File in database directory has invalid format: {0}")]
    InvalidFileFormat(BincodeError),
    #[error("Unable to serialize database: {0}")]
    SerializationError(BincodeError),
    #[error("Unable to write serialized database to file: {0}")]
    FileWriteError(Error),
    #[error("Unable to read serialized database from file: {0}")]
    FileReadError(Error),
}

type PersistenceResult<T> = Result<T, PersistenceError>;

/// Persist database to provided path
async fn persist_db(name: &str, db: &DB, path: &Path) -> Result<(), PersistenceError> {
    info!("Saving database \"{}\"", name);

    // Save queue and replication storage separatly
    let path = path.join(name);
    if !path.is_dir() {
        create_dir(&path)
            .await
            .map_err(PersistenceError::FileWriteError)?;
    }

    write(
        path.join(QUEUE_FILE),
        serialize(&*db.database.lock().await).map_err(PersistenceError::SerializationError)?,
    )
    .await
    .map_err(PersistenceError::FileWriteError)?;

    #[cfg(feature = "replication")]
    {
        write(
            path.join(REPLICATION_FILE),
            serialize(&*db.replication_storage.lock().await)
                .map_err(PersistenceError::SerializationError)?,
        )
        .await
        .map_err(PersistenceError::FileWriteError)?;
    }

    info!("Saved \"{}\" successfully", name);

    Ok(())
}

/// Persist all databases from manager
pub async fn persist_manager(manager: &Manager<'_>) {
    iter(manager.node.iter())
        .for_each_concurrent(None, |(name, db)| async move {
            match persist_db(name, db, &manager.config.path).await {
                Err(PersistenceError::SerializationError(e)) => {
                    error!("Unable to serialize database: {}", e)
                }
                Err(PersistenceError::FileWriteError(e)) => {
                    error!("Unable to write serialized database to file: {}", e)
                }
                _ => (),
            }
        })
        .await;
}

/// Persistence job handler, that persists all databases from manager
pub async fn spawn_persistence(manager: &Manager<'_>) {
    debug!("Spawning persistence job.");

    let timer = Duration::from_secs(manager.config.persistence_timer);

    loop {
        delay_for(timer).await;
        persist_manager(manager).await;
    }
}

async fn read_struct<T>(path: &Path) -> PersistenceResult<T>
where
    T: DeserializeOwned,
{
    let buf = read(path).await.map_err(PersistenceError::FileReadError)?;
    deserialize(&buf).map_err(PersistenceError::InvalidFileFormat)
}

async fn load<'a>(manager: &mut Manager<'a>, name: &'a str) -> PersistenceResult<()> {
    let path = manager.config.path.join(name);
    let database = read_struct(&path.join(QUEUE_FILE)).await?;
    cfg_if! {
        if #[cfg(feature = "replication")] {
            let replication_storage = read_struct(&path.join(REPLICATION_FILE)).await?;
            let queue = Queue::new(database, replication_storage);
        } else {
            let queue = Queue::new(database);
        }
    }
    manager.node.add_db(name, queue);
    Ok(())
}

/// Load manager from FS
pub async fn load_from_fs(manager: &mut Manager<'_>) -> PersistenceResult<()> {
    for queue in manager.config.queues.iter() {
        match load(manager, queue).await {
            Err(PersistenceError::FileReadError(e)) => {
                warn!("Unable to load database {}: {}", queue, e)
            }
            Err(e) => return Err(e),
            _ => (),
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{load_from_fs, persist_manager};
    use crate::{config::Config, node::Manager};
    use spartan_lib::core::{
        dispatcher::{SimpleDispatcher, StatusAwareDispatcher},
        message::builder::MessageBuilder,
        payload::Dispatchable,
    };
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_persistence() {
        let tempdir = TempDir::new().expect("Unable to create temporary test directory");

        let config = Config {
            path: tempdir.path().to_owned().into_boxed_path(),
            queues: Box::new([
                String::from("test").into_boxed_str(),
                String::from("test2").into_boxed_str(),
            ]),
            ..Default::default()
        };

        {
            let manager = Manager::new(&config);

            let message = MessageBuilder::default()
                .body("Hello, world")
                .compose()
                .unwrap();
            manager
                .queue("test")
                .unwrap()
                .database
                .lock()
                .await
                .push(message);

            persist_manager(&manager).await;
        }

        let mut manager = Manager::new(&config);
        load_from_fs(&mut manager).await.unwrap();

        manager.queue("test2").unwrap();
        assert_eq!(
            manager
                .queue("test")
                .unwrap()
                .database
                .lock()
                .await
                .pop()
                .unwrap()
                .body(),
            "Hello, world"
        );
    }
}
