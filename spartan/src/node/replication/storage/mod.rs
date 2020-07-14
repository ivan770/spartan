/// Storage for replication primary host
pub mod primary;

/// Storage for replica's
pub mod replica;

use crate::node::Manager;
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

    pub async fn prepare<F, R>(manager: &Manager<'_>, filter: F, replace: R)
    where
        F: Fn(&&ReplicationStorage) -> bool + Copy,
        R: Fn() -> ReplicationStorage,
    {
        for (_, db) in manager.node.db.iter() {
            let mut db = db.lock().await;

            let storage = db.get_storage().as_ref().filter(filter);

            if storage.is_none() {
                db.get_storage().replace(replace());
            }
        }
    }
}
