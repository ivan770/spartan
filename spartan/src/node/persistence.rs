use super::Node;
use crate::server::Config;
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

pub async fn spawn(persistence: &Persistence) -> Result<(), PersistenceError> {
    loop {
        sleep(Duration::from_secs(persistence.config.timer)).await;

        for (name, db) in persistence.node().db.iter() {
            let db = db.lock().await;

            let mut path = persistence.config.path.clone();
            path.push(name);

            info!("Saving database \"{}\"", name);
            write(
                path,
                serialize(&*db).map_err(|e| PersistenceError::SerializationError(e))?,
            )
            .await
            .map_err(|e| PersistenceError::FileWriteError(e))?;

            info!("Saved \"{}\" successfully", name);
        }
    }
}

pub async fn load_from_fs(persistence: &mut Persistence) -> PersistenceResult<()> {
    let files = persistence
        .config
        .path
        .read_dir()
        .map_err(|e| PersistenceError::DirectoryOpenError(e))?
        .into_iter()
        .filter_map(|file| file.ok())
        .filter_map(|file| Some((file.file_name().into_string().ok()?, file)));

    for (name, data) in files {
        info!("Loading \"{}\" from file", name);
        let file_buf = read(data.path())
            .await
            .map_err(|e| PersistenceError::DatabaseFileOpenError(e))?;
        let db = deserialize(&file_buf).map_err(|e| PersistenceError::InvalidFileFormat(e))?;
        persistence.node_mut().db.insert(name, Mutex::new(db));
    }

    Ok(())
}

pub struct Persistence {
    config: Config,
    node: Option<Node>,
}

impl Persistence {
    pub fn new(config: Config) -> Persistence {
        Persistence { config, node: None }
    }

    pub fn node(&self) -> &Node {
        self.node.as_ref().expect("Node not loaded")
    }

    pub fn node_mut(&mut self) -> &mut Node {
        self.node.as_mut().expect("Node not loaded")
    }

    pub fn load(&mut self) {
        let mut node = Node::default();
        node.load_from_config(&self.config);
        self.node = Some(node);
    }
}
