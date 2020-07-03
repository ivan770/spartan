use super::{
    raft::{entry::LogEntry, guard::QueueGuard},
    Node, DB,
};
use crate::config::Config;
use actix_raft::{messages::Entry, AppData};
use actix_web::{http::StatusCode, ResponseError};
use futures_util::lock::Mutex;
use std::collections::BTreeMap;
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
pub struct Manager<'a, E = LogEntry>
where
    E: AppData,
{
    /// Server config
    pub config: &'a Config,

    /// Node
    pub node: Node<'a>,

    // TODO: Use concurrent BTreeMap
    /// Raft log
    replication_log: Option<Mutex<BTreeMap<u64, Entry<E>>>>,
}

impl<'a> Manager<'a> {
    /// Create new manager without node
    pub fn new(config: &'a Config) -> Manager<'a> {
        let mut node = Node::default();
        node.load_from_config(config);

        let replication_log = if config.replication {
            Some(Mutex::new(BTreeMap::new()))
        } else {
            None
        };

        Manager {
            config,
            node,
            replication_log,
        }
    }

    /// Obtain queue from local node
    pub async fn queue(
        &'a self,
        name: &'a str,
    ) -> Result<QueueGuard<'a, LogEntry, DB>, ManagerError> {
        let queue = self
            .node
            .get(name)
            .await
            .ok_or_else(|| ManagerError::QueueNotFound)?;

        if let Some(log) = self.replication_log.as_ref() {
            Ok(QueueGuard::new(name, queue, Some(log.lock().await)))
        } else {
            Ok(QueueGuard::new(name, queue, None))
        }
    }
}
