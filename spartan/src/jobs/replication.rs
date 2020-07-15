use crate::{
    config::replication::{Primary, Replication},
    node::{
        replication::{
            primary::{error::PrimaryResult, stream::StreamPool},
            storage::{primary::PrimaryStorage, ReplicationStorage},
        },
        Manager,
    },
};
use actix_rt::time::delay_for;
use std::time::Duration;
use tokio::io::Result as IoResult;

async fn replicate_manager(manager: &Manager<'_>, pool: &mut StreamPool) -> PrimaryResult<()> {
    pool.ping().await?;

    pool.ask().await?.sync(manager).await?;

    Ok(())
}

async fn start_replication(manager: &Manager<'_>, pool: &mut StreamPool, config: &Primary) {
    let timer = Duration::from_secs(config.replication_timer);

    loop {
        delay_for(timer).await;

        match replicate_manager(manager, pool).await {
            Ok(_) => info!("Database replicated successfully!"),
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

                match StreamPool::from_config(config).await {
                    Ok(mut pool) => start_replication(manager, &mut pool, config).await,
                    Err(e) => error!("Unable to open connection pool: {}", e),
                }
            }
            Replication::Replica(_) => {
                panic!("Starting replication job while in replica mode is not allowed")
            }
        };
    }

    Ok(())
}
