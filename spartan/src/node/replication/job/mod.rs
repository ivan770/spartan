/// Tokio TCP stream abstraction
pub mod stream;

/// Replication index exchange
pub mod index;

/// Replication error
pub mod error;

use super::storage::{primary::PrimaryStorage, ReplicationStorage};
use crate::{
    config::replication::{Primary, Replication},
    node::Manager,
};
use actix_rt::time::delay_for;
use error::ReplicationResult;
use std::time::Duration;
use stream::StreamPool;
use tokio::io::Result as IoResult;

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

async fn replicate_manager(manager: &Manager<'_>, pool: &mut StreamPool) -> ReplicationResult<()> {
    pool.ping().await?;

    pool.ask().await?.sync(manager).await?;

    Ok(())
}

async fn start_replication(manager: &Manager<'_>, pool: &mut StreamPool, config: &Primary) {
    let timer = Duration::from_secs(config.replication_timer);

    loop {
        delay_for(timer).await;

        match replicate_manager(manager, pool).await {
            Err(e) => error!("Error happened during replication attempt: {}", e),
            Ok(_) => info!("Database replicated successfully!"),
        }
    }
}

pub async fn spawn_replication(manager: &Manager<'_>) -> IoResult<()> {
    manager
        .config
        .replication
        .as_ref()
        .map(|config| async move {
            match config {
                Replication::Primary(config) => {
                    prepare_storage(manager).await;

                    match StreamPool::from_config(config).await {
                        Ok(mut pool) => start_replication(manager, &mut pool, config).await,
                        Err(e) => error!("Unable to open connection pool: {}", e),
                    }
                }
                Replication::Replica(_) => {
                    panic!("Starting replication job while in replica mode is not allowed")
                }
            };
        });

    Ok(())
}
