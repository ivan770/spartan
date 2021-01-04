use futures_util::{stream::iter, StreamExt, TryStreamExt};
use thiserror::Error;
use warp::hyper::StatusCode;

use crate::{
    actions::RespondableError,
    config::{persistence::Persistence, Config},
    node::{
        event::Event,
        persistence::{
            log::Log,
            snapshot::{PersistMode, Snapshot},
            PersistenceError,
        },
        Node, DB,
    },
};

#[derive(Error, Debug)]
pub enum ManagerError {
    #[error("Queue not found")]
    QueueNotFound,
}

impl RespondableError for ManagerError {
    fn status_code(&self) -> StatusCode {
        StatusCode::NOT_FOUND
    }
}

/// Node manager
pub struct Manager<'c> {
    /// Server config
    config: &'c Config<'c>,

    /// Node
    node: Node<'c>,
}

impl<'c> Manager<'c> {
    /// Create new manager without node
    pub fn new(config: &'c Config) -> Manager<'c> {
        let mut node = Node::default();
        node.load_from_config(config);
        Manager { config, node }
    }

    /// Obtain queue from local node
    pub fn queue(&self, name: &str) -> Result<&DB, ManagerError> {
        self.node.queue(name).ok_or(ManagerError::QueueNotFound)
    }

    pub fn config(&self) -> &'c Config<'c> {
        &self.config
    }

    pub fn node(&self) -> &Node<'_> {
        &self.node
    }

    pub async fn load_from_fs(&mut self) -> Result<(), PersistenceError> {
        if let Some(config) = self.config.persistence.as_ref() {
            match config.mode {
                Persistence::Log => {
                    let driver = Log::new(config);

                    for name in self.config.queues.iter() {
                        let queue = driver.load_queue(&**name).await?;
                        self.node.add_db(name, queue);
                    }
                }
                Persistence::Snapshot => {
                    let driver = Snapshot::new(config);

                    for name in self.config.queues.iter() {
                        let queue = driver.load_queue(&**name).await?;
                        self.node.add_db(name, queue);
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn snapshot(&self) -> Result<(), PersistenceError> {
        if let Some(config) = self.config.persistence.as_ref() {
            let mode = match config.mode {
                Persistence::Snapshot => PersistMode::Queue,
                Persistence::Log => PersistMode::Replication,
            };

            let driver = &Snapshot::new(config);

            iter(self.node.iter())
                .map(Ok)
                .try_for_each_concurrent(None, move |(name, db)| {
                    driver.persist_queue(name, db, mode)
                })
                .await
        } else {
            Ok(())
        }
    }

    pub async fn log(&self, queue: &str, event: &Event<'_>) -> Result<(), PersistenceError> {
        if let Some(config) = self
            .config
            .persistence
            .as_ref()
            .filter(|config| matches!(config.mode, Persistence::Log))
        {
            Log::new(config).persist_event(event, queue).await
        } else {
            Ok(())
        }
    }

    /// Prepare [`Manager`] for shutdown process
    ///
    /// Internally persists snapshot instance and outputs error message
    /// in case anything goes wrong
    ///
    /// Though not recommended to be done, this method can be called multiple times
    pub async fn shutdown(&self) {
        if let Err(e) = self.snapshot().await {
            error!("Error happened during shutdown: {}", e)
        }
    }
}

#[cfg(test)]
impl<'c> Manager<'c> {
    pub fn node_mut(&mut self) -> &mut Node<'c> {
        &mut self.node
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use maybe_owned::MaybeOwned;
    use spartan_lib::core::{
        dispatcher::SimpleDispatcher, message::builder::MessageBuilder, payload::Dispatchable,
    };
    use tempfile::TempDir;

    use super::Manager;
    use crate::{
        config::{
            persistence::{Persistence, PersistenceConfig},
            Config,
        },
        node::event::Event,
    };

    #[tokio::test]
    async fn test_load_snapshot() {
        let dir = TempDir::new().unwrap();

        let config = Config {
            persistence: Some(PersistenceConfig {
                mode: Persistence::Snapshot,
                path: Cow::Borrowed(&dir.path()),
                ..Default::default()
            }),
            ..Default::default()
        };

        {
            let manager = Manager::new(&config);

            manager.queue("test").unwrap().database().await.push(
                MessageBuilder::default()
                    .body("Hello, world")
                    .compose()
                    .unwrap(),
            );

            manager.snapshot().await.unwrap();
        }

        let mut manager = Manager::new(&config);
        manager.load_from_fs().await.unwrap();

        assert_eq!(
            manager
                .queue("test")
                .unwrap()
                .database()
                .await
                .peek()
                .unwrap()
                .body(),
            "Hello, world"
        );
    }

    async fn load_log(compaction: bool) {
        let dir = TempDir::new().unwrap();

        let config = Config {
            persistence: Some(PersistenceConfig {
                mode: Persistence::Log,
                path: Cow::Borrowed(&dir.path()),
                compaction,
                ..Default::default()
            }),
            queues: vec!["test".to_string().into_boxed_str()].into_boxed_slice(),
            ..Default::default()
        };

        {
            let manager = Manager::new(&config);

            manager
                .log(
                    "test",
                    &Event::Push(MaybeOwned::Owned(
                        MessageBuilder::default()
                            .body("Hello, world")
                            .compose()
                            .unwrap(),
                    )),
                )
                .await
                .unwrap();
        }

        let mut manager = Manager::new(&config);
        manager.load_from_fs().await.unwrap();

        assert_eq!(
            manager
                .queue("test")
                .unwrap()
                .database()
                .await
                .peek()
                .unwrap()
                .body(),
            "Hello, world"
        );

        if compaction {
            let mut manager = Manager::new(&config);
            manager.load_from_fs().await.unwrap();

            assert_eq!(
                manager
                    .queue("test")
                    .unwrap()
                    .database()
                    .await
                    .peek()
                    .unwrap()
                    .body(),
                "Hello, world"
            );
        }
    }

    #[tokio::test]
    async fn test_load_log() {
        load_log(false).await;
    }

    #[tokio::test]
    async fn test_load_log_compaction() {
        load_log(true).await;
    }
}
