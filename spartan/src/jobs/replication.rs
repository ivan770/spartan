use crate::{
    config::replication::{Primary, Replication},
    node::{
        replication::{
            primary::{error::{PrimaryError, PrimaryResult}, stream::StreamPool},
            storage::{primary::PrimaryStorage, ReplicationStorage},
        },
        Manager,
    },
};
use actix_rt::time::delay_for;
use std::time::Duration;
use tokio::io::Result as IoResult;

async fn replicate_manager(manager: &Manager<'_>, pool: &mut StreamPool) -> PrimaryResult<()> {
    debug!("Pinging stream pool.");
    pool.ping().await?;

    debug!("Asking stream pool for indexes.");
    let mut batch = pool.ask().await?;

    debug!("Starting event slice sync.");
    batch.sync(manager).await?;

    debug!("Setting GC threshold.");
    batch.set_gc(manager).await;

    Ok(())
}

async fn start_replication(manager: &Manager<'_>, pool: &mut StreamPool, config: &Primary) {
    let timer = Duration::from_secs(config.replication_timer);

    loop {
        delay_for(timer).await;

        info!("Starting database replication.");

        match replicate_manager(manager, pool).await {
            Ok(_) => info!("Database replicated successfully!"),
            Err(PrimaryError::EmptySocket) => {
                error!("Empty TCP socket");
                return;
            },
            Err(PrimaryError::SocketError(e)) => {
                error!("TCP socket error: {}", e);
                return;
            }
            Err(e) => error!("Error happened during replication attempt: {}", e),
        }
    }
}

pub async fn spawn_replication(manager: &Manager<'_>) -> IoResult<()> {
    debug!("Spawning replication job.");

    if let Some(config) = manager.config.replication.as_ref() {
        match config {
            Replication::Primary(config) => {
                manager
                    .node
                    .prepare_replication(
                        |storage| matches!(storage, ReplicationStorage::Primary(_)),
                        || ReplicationStorage::Primary(PrimaryStorage::default()),
                    )
                    .await;

                let timer = Duration::from_secs(config.try_timer);
                
                loop {
                    delay_for(timer).await;

                    match StreamPool::from_config(config).await {
                        Ok(mut pool) => start_replication(manager, &mut pool, config).await,
                        Err(e) => error!("Unable to open connection pool: {}", e),
                    }
                }
            }
            Replication::Replica(_) => {
                panic!("Starting replication job while in replica mode is not allowed")
            }
        };
    }

    Ok(())
}
