use super::Manager;
use actix_rt::time::delay_for;
use bincode::{deserialize, serialize, Error as BincodeError};
use futures_util::lock::Mutex;
use futures_util::stream::{iter, StreamExt};
use spartan_lib::core::{db::tree::TreeDatabase, message::Message};
use std::{io::Error, path::PathBuf, time::Duration};
use thiserror::Error as ThisError;
use tokio::fs::{read, write};

/// Enum of errors, that may occur during persistence jobs
#[derive(ThisError, Debug)]
pub enum PersistenceError {
    #[error("Unable to open database directory: {0}")]
    DirectoryOpenError(Error),
    #[error("Unable to open file in database directory: {0}")]
    DatabaseFileOpenError(Error),
    #[error("File in database directory has invalid format: {0}")]
    InvalidFileFormat(BincodeError),
    #[error("Unable to serialize database: {0}")]
    SerializationError(BincodeError),
    #[error("Unable to write serialized database to file: {0}")]
    FileWriteError(Error),
}

type PersistenceResult<T> = Result<T, PersistenceError>;

/// Persist database to provided path
async fn persist_db(
    name: &str,
    db: &Mutex<TreeDatabase<Message>>,
    mut path: PathBuf,
) -> Result<(), PersistenceError> {
    let db = db.lock().await;

    path.push(name);

    info!("Saving database \"{}\"", name);

    write(
        path,
        serialize(&*db).map_err(PersistenceError::SerializationError)?,
    )
    .await
    .map_err(PersistenceError::FileWriteError)?;

    info!("Saved \"{}\" successfully", name);

    Ok(())
}

/// Persist all databases from manager
pub async fn persist_manager(manager: &Manager<'_>) {
    iter(manager.node().db.iter())
        .for_each_concurrent(None, |(name, db)| async move {
            let path = manager.config.path.clone();

            match persist_db(name, db, path).await {
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
    let timer = Duration::from_secs(manager.config.persistence_timer);

    loop {
        delay_for(timer).await;

        persist_manager(manager).await;
    }
}

/// Load manager from FS
pub async fn load_from_fs(manager: &mut Manager<'_>) -> PersistenceResult<()> {
    let files = manager
        .config
        .path
        .read_dir()
        .map_err(PersistenceError::DirectoryOpenError)?
        .filter_map(|file| file.ok())
        .filter_map(|file| Some((file.file_name().into_string().ok()?, file)));

    for (name, data) in files {
        info!("Loading \"{}\" from file", name);
        let file_buf = read(data.path())
            .await
            .map_err(PersistenceError::DatabaseFileOpenError)?;
        let db = deserialize(&file_buf).map_err(PersistenceError::InvalidFileFormat)?;
        // manager.node_mut().db.
    }

    Ok(())
}
