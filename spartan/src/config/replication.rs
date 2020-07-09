use crate::node::replication::storage::ReplicationStorage;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Serialize, Deserialize)]
pub struct Primary {
    destination: Vec<SocketAddr>,
}

#[derive(Serialize, Deserialize)]
pub struct Replica {
    host: SocketAddr,
}

#[derive(Serialize, Deserialize)]
pub enum Replication {
    Primary(Primary),
    Replica(Replica),
}

impl PartialEq<ReplicationStorage> for Replication {
    fn eq(&self, other: &ReplicationStorage) -> bool {
        match *self {
            Replication::Primary(_) => matches!(*other, ReplicationStorage::Primary(_)),
            Replication::Replica(_) => matches!(*other, ReplicationStorage::Replica(_)),
        }
    }
}
