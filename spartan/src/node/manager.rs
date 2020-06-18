use super::{Node, DB};
use crate::server::Config;
use actix_web::{http::StatusCode, ResponseError};
use futures_util::lock::MutexGuard;
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
    pub config: &'a Config,

    /// Node
    node: Node<'a>,
}

impl<'a> Manager<'a> {
    /// Create new manager without node
    pub fn new(config: &'a Config) -> Manager<'a> {
        let mut node = Node::default();
        node.load_from_config(config);
        Manager { config, node }
    }

    /// Get node reference
    pub fn node(&self) -> &Node {
        &self.node
    }

    /// Get mutable node reference
    pub fn node_mut(&'a mut self) -> &mut Node {
        &mut self.node
    }

    /// Obtain queue from local node
    pub async fn queue(&self, name: &str) -> Result<MutexGuard<'_, DB>, ManagerError> {
        self.node()
            .get(name)
            .await
            .ok_or_else(|| ManagerError::QueueNotFound)
    }
}
