use super::{
    event::Event,
    persistence::log::Log,
    persistence::{
        snapshot::{PersistMode, Snapshot},
        PersistenceError,
    },
    Node, DB,
};
use crate::{config::{Config, persistence::Persistence}};
use actix_web::{http::StatusCode, ResponseError};
use futures_util::{stream::iter, StreamExt, TryStreamExt};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ManagerError {
    #[error("Queue not found")]
    QueueNotFound,
}

impl ResponseError for ManagerError {
    fn status_code(&self) -> StatusCode {
        StatusCode::NOT_FOUND
    }
}

/// Node manager
pub struct Manager<'a> {
    /// Server config
    pub config: &'a Config<'a>,

    /// Node
    pub node: Node<'a>,
}

impl<'a> Manager<'a> {
    /// Create new manager without node
    pub fn new(config: &'a Config) -> Manager<'a> {
        let mut node = Node::default();
        node.load_from_config(config);
        Manager { config, node }
    }

    /// Obtain queue from local node
    pub fn queue(&self, name: &str) -> Result<&DB, ManagerError> {
        self.node.queue(name).ok_or(ManagerError::QueueNotFound)
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
                .try_for_each_concurrent(None, |(name, db)| async move {
                    driver.persist_queue(name, db, mode).await
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
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use maybe_owned::MaybeOwned;
    use spartan_lib::core::{
        dispatcher::SimpleDispatcher, message::builder::MessageBuilder, payload::Dispatchable,
    };
    use tempfile::TempDir;

    use crate::{
        config::{Config, persistence::{Persistence, PersistenceConfig}},
        node::event::Event,
    };

    use super::Manager;

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
