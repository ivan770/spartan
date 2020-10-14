use super::{persistence::log::Log, persistence::snapshot::Snapshot, Node, DB};
use crate::{config::Config, persistence_config::Persistence};
use actix_web::{http::StatusCode, ResponseError};
use futures_util::StreamExt;
use thiserror::Error;
use tokio::stream::iter;

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

    pub async fn load_from_fs(&mut self) {
        if let Some(persistence) = self.config.persistence.as_ref() {
            match persistence {
                Persistence::Log(config) => {
                    let driver = Log::new(config);
                }
                Persistence::Snapshot(config) => {
                    let driver = Snapshot::new(config);

                    for name in self.config.queues.iter() {
                        match driver.load_queue(&**name).await {
                            Ok(queue) => {
                                self.node.add_db(name, queue);
                            }
                            Err(e) => error!("{}", e),
                        }
                    }
                }
            }
        }
    }

    pub async fn snapshot(&self) {
        if let Some(persistence) = self.config.persistence.as_ref() {
            match persistence {
                Persistence::Snapshot(config) => {
                    let driver = &Snapshot::new(config);

                    iter(self.node.iter())
                        .for_each_concurrent(None, |(name, db)| async move {
                            match driver.persist_queue(name, db).await {
                                Err(e) => error!("{}", e),
                                _ => (),
                            }
                        })
                        .await;
                }
                _ => (),
            }
        }
    }
}
