use actix_raft::AppError;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Serialize, Deserialize, Error, Debug)]
pub enum ReplicationError {
    #[error("Missing replication log. This is probably due to disabled replication")]
    MissingReplicationLog,
    #[error("Arc strong references already exist")]
    ExistingEntryReferences,
}

impl AppError for ReplicationError {}
