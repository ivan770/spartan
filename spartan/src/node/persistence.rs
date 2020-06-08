use super::Manager;
use async_std::{
    fs::{read, write},
    sync::Mutex,
    task::sleep,
};
use bincode::{deserialize, serialize, Error as BincodeError};
use std::{io::Error, time::Duration};
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum PersistenceError {
    #[error("Unable to open database directory")]
    DirectoryOpenError(Error),
    #[error("Unable to open file in database directory")]
    DatabaseFileOpenError(Error),
    #[error("File in database directory has invalid format")]
    InvalidFileFormat(BincodeError),
    #[error("Unable to serialize database")]
    SerializationError(BincodeError),
    #[error("Unable to write serialized database to file")]
    FileWriteError(Error),
}

type PersistenceResult<T> = Result<T, PersistenceError>;

pub async fn spawn_persistence(manager: &Manager) -> Result<(), PersistenceError> {
    loop {
        sleep(Duration::from_secs(manager.config.persistence_timer)).await;

        for (name, db) in manager.node().db.iter() {
            let db = db.lock().await;

            let mut path = manager.config.path.clone();
            path.push(name);

            info!("Saving database \"{}\"", name);
            write(
                path,
                serialize(&*db).map_err(PersistenceError::SerializationError)?,
            )
            .await
            .map_err(PersistenceError::FileWriteError)?;

            info!("Saved \"{}\" successfully", name);
        }
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
