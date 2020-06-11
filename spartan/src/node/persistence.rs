use super::Manager;
use async_std::{
    fs::{read, write},
    sync::Mutex,
    task::sleep,
};
use bincode::{deserialize, serialize, Error as BincodeError};
use futures::stream::{iter, StreamExt};
use spartan_lib::core::{db::tree::TreeDatabase, message::Message};
use std::{io::Error, path::PathBuf, time::Duration};
use thiserror::Error as ThisError;

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

pub async fn persist_manager(manager: &Manager) {
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

pub async fn spawn_persistence(manager: &Manager) {
    let timer = Duration::from_secs(manager.config.persistence_timer);

    loop {
        sleep(timer).await;

        persist_manager(manager).await;
    }
}

pub async fn load_from_fs(manager: &mut Manager) -> PersistenceResult<()> {
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
        manager.node_mut().db.insert(name, Mutex::new(db));
    }

    Ok(())
}
