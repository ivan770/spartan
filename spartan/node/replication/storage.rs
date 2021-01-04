use serde::{Deserialize, Serialize};

use crate::node::replication::{
    primary::storage::PrimaryStorage, replica::storage::ReplicaStorage,
};

#[derive(Serialize, Deserialize)]
pub enum ReplicationStorage {
    Primary(PrimaryStorage),
    Replica(ReplicaStorage),
}

impl ReplicationStorage {
    pub fn get_primary(&mut self) -> &mut PrimaryStorage {
        match self {
            ReplicationStorage::Primary(storage) => storage,
            _ => panic!("Replication storage is in replica mode."),
        }
    }

    pub fn get_replica(&mut self) -> &mut ReplicaStorage {
        match self {
            ReplicationStorage::Replica(storage) => storage,
            _ => panic!("Replication storage is in primary mode."),
        }
    }

    pub fn map_primary<F>(&mut self, f: F)
    where
        F: FnOnce(&mut PrimaryStorage),
    {
        match self {
            ReplicationStorage::Primary(storage) => f(storage),
            ReplicationStorage::Replica(_) => (),
        }
    }
}
