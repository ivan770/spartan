use super::Manager;
use actix::{AsyncContext, Handler, Message as ActixMessage, fut::wrap_future, ActorFuture, ResponseActFuture, AtomicResponse};
use actix_rt::time::delay_for;
use bincode::{deserialize, serialize, Error as BincodeError};
use futures_util::lock::Mutex;
use futures_util::{stream::{iter, StreamExt}};
use spartan_lib::core::{db::tree::TreeDatabase, message::Message};
use std::{io::Error, path::Path, time::Duration, pin::Pin, future::Future};
use thiserror::Error as ThisError;
use tokio::fs::{read, write};

/// Enum of errors, that may occur during persistence jobs
#[derive(ThisError, Debug)]
pub enum PersistenceError {
    #[error("File in database directory has invalid format: {0}")]
    InvalidFileFormat(BincodeError),
    #[error("Unable to serialize database: {0}")]
    SerializationError(BincodeError),
    #[error("Unable to write serialized database to file: {0}")]
    FileWriteError(Error),
}

type PersistenceResult<T> = Result<T, PersistenceError>;

pub struct LoadFromFs {}

impl LoadFromFs {
    pub fn new() -> Self {
        LoadFromFs {}
    }

    pub fn test<T>(&self, f: T)
    where
        T: Future<Output = ()> + 'static
    {
        
    }
}

impl ActixMessage for LoadFromFs {
    type Result = ();
}

impl Handler<LoadFromFs> for Manager<'static> {
    type Result = ();

    fn handle(&mut self, msg: LoadFromFs, ctx: &mut Self::Context) -> Self::Result {
        let f = wrap_future::<_, Self>(persist_manager(self));
        ctx.wait(f);
    }
}

/// Persist database to provided path
async fn persist_db(
    name: &str,
    db: &Mutex<TreeDatabase<Message>>,
    path: &Path,
) -> Result<(), PersistenceError> {
    let db = db.lock().await;

    info!("Saving database \"{}\"", name);

    write(
        path.join(name),
        serialize(&*db).map_err(PersistenceError::SerializationError)?,
    )
    .await
    .map_err(PersistenceError::FileWriteError)?;

    info!("Saved \"{}\" successfully", name);

    Ok(())
}

/// Persist all databases from manager
pub async fn persist_manager(manager: &Manager<'_>) {
    iter(manager.node.db.iter())
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
    let timer = Duration::from_secs(manager.config.persistence_timer);

    loop {
        delay_for(timer).await;

        persist_manager(manager).await;
    }
}

/// Load manager from FS
pub async fn load_from_fs(manager: &mut Manager<'_>) -> PersistenceResult<()> {
    for queue in manager.config.queues.iter() {
        match read(manager.config.path.join(queue)).await {
            Ok(file_buf) => {
                let db = deserialize(&file_buf).map_err(PersistenceError::InvalidFileFormat)?;
                manager.node.db.insert(queue, Mutex::new(db));
            }
            Err(e) => warn!("Unable to load database {}: {}", queue, e),
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
            path: tempdir.path().to_owned(),
            queues: vec![String::from("test"), String::from("test2")],
            ..Default::default()
        };

        {
            let manager = Manager::new(&config);

            let message = MessageBuilder::default()
                .body("Hello, world")
                .compose()
                .unwrap();
            manager.queue("test").await.unwrap().push(message);

            persist_manager(&manager).await;
        }

        let mut manager = Manager::new(&config);
        load_from_fs(&mut manager).await.unwrap();

        manager.queue("test2").await.unwrap();
        assert_eq!(
            manager.queue("test").await.unwrap().pop().unwrap().body(),
            "Hello, world"
        );
    }
}
