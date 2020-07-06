use super::{
    raft::entry::LogEntry,
    Node, DB,
};
use crate::config::Config;
use actix_web::{http::StatusCode, ResponseError};
use futures_util::lock::{MutexGuard, Mutex};
use thiserror::Error;
use std::collections::BTreeMap;
use actix_raft::{AppData, messages::Entry};

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
    E: AppData
{
    /// Server config
    pub config: &'a Config,

    /// Node
    pub node: Node<'a>,

    // TODO: Use concurrent BTreeMap
    /// Raft log
    pub replication_log: BTreeMap<u64, Entry<E>>,
}

impl<'a> Manager<'a> {
    /// Create new manager without node
    pub fn new(config: &'a Config) -> Manager<'a> {
        let mut node = Node::default();
        node.load_from_config(config);

        let replication_log = BTreeMap::new();

        Manager {
            config,
            node,
            replication_log
        }
    }

    /// Obtain queue from local node
    pub async fn queue(
        &'a self,
        name: &'a str,
    ) -> Result<MutexGuard<'a, DB>, ManagerError> {
        Ok(self
            .node
            .get(name)
            .await
            .ok_or_else(|| ManagerError::QueueNotFound)?)
    }
}
