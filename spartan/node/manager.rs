use super::{Node, DB};
use crate::config::Config;
use actix_web::{http::StatusCode, ResponseError};
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
        self.node
            .queue(name)
            .ok_or_else(|| ManagerError::QueueNotFound)
    }
}
