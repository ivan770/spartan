/// Storage for replication primary host
pub mod primary;

/// Storage for replica's
pub mod replica;

use primary::PrimaryStorage;
use replica::ReplicaStorage;
use serde::{Deserialize, Serialize};

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
