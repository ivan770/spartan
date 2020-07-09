use super::storage::{primary::PrimaryStorage, ReplicationStorage};
use crate::{config::replication::Replication, node::Manager};

async fn prepare_storage(manager: &Manager<'_>) {
    for (_, db) in manager.node.db.iter() {
        let mut db = db.lock().await;

        let storage = db
            .get_storage()
            .as_ref()
            .filter(|storage| matches!(storage, ReplicationStorage::Primary(_)));

        if storage.is_none() {
            db.get_storage()
                .replace(ReplicationStorage::Primary(PrimaryStorage::default()));
        }
    }
}

async fn replicate_manager(manager: &Manager<'_>) {
    prepare_storage(manager).await;
}

pub async fn spawn_replication(manager: &Manager<'_>) {
    manager
        .config
        .replication
        .as_ref()
        .map(|config| async move {
            match config {
                Replication::Primary(_) => replicate_manager(manager).await,
                Replication::Replica(_) => (),
            };
        });
}
