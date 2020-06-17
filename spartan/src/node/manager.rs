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
pub struct Manager {
    /// Server config
    pub config: Config,

    /// Node
    node: Option<Node>,
}

impl Manager {
    /// Create new manager without node
    pub fn new(config: Config) -> Manager {
        Manager { config, node: None }
    }

    /// Get node reference
    pub fn node(&self) -> &Node {
        self.node.as_ref().expect("Node not loaded")
    }

    /// Get mutable node reference
    pub fn node_mut(&mut self) -> &mut Node {
        self.node.as_mut().expect("Node not loaded")
    }

    /// Obtain queue from local node
    pub async fn queue(&self, name: &str) -> Result<MutexGuard<'_, DB>, ManagerError> {
        self.node()
            .get(name)
            .await
            .ok_or_else(|| ManagerError::QueueNotFound)
    }

    /// Load node instance from config
    pub fn load(&mut self) {
        let mut node = Node::default();
        node.load_from_config(&self.config);
        self.node = Some(node);
    }
}
