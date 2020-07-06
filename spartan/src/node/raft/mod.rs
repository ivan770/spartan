use actix_raft::AppDataResponse;
use serde::{Deserialize, Serialize};

/// Log entry and actions
pub mod entry;

/// Replication error
pub mod error;

/// Replication storage
pub mod storage;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReplicationResponse();

impl AppDataResponse for ReplicationResponse {}
