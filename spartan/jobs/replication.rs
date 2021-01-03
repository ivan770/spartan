use std::{collections::hash_map::DefaultHasher, time::Duration};

use tokio::{net::TcpStream, time::delay_for};

use crate::{
    config::replication::{Primary, Replication},
    node::{
        replication::{
            primary::{
                error::{PrimaryError, PrimaryResult},
                storage::PrimaryStorage,
                stream::StreamPool,
            },
            storage::ReplicationStorage,
        },
        Manager,
    },
};

async fn replicate_manager(
    manager: &Manager<'_>,
    pool: &mut StreamPool<TcpStream>,
) -> PrimaryResult<()> {
    pool.ping().await?;

    pool.ask()
        .await?
        .sync(manager)
        .await?
        .set_gc::<DefaultHasher>(manager)
        .await;

    Ok(())
}

async fn start_replication(
    manager: &Manager<'_>,
    pool: &mut StreamPool<TcpStream>,
    config: &Primary,
) {
    let timer = Duration::from_secs(config.replication_timer);

    loop {
        delay_for(timer).await;

        info!("Starting database replication.");

        match replicate_manager(manager, pool).await {
            Ok(_) => info!("Database replicated successfully!"),
            Err(PrimaryError::EmptySocket) => {
                error!("Empty TCP socket");
                return;
            }
            Err(PrimaryError::SocketError(e)) => {
                error!("TCP socket error: {}", e);
                return;
            }
            Err(PrimaryError::CodecError(e)) => {
                error!("Codec error: {}", e);
                return;
            }
            Err(e) => error!("Error happened during replication attempt: {}", e),
        }
    }
}

/// Spawn replication job
pub async fn spawn_replication(manager: &Manager<'_>) {
    debug!("Spawning replication job.");

    if let Some(config) = manager.config().replication.as_ref() {
        match config.mode {
            Replication::Primary if config.primary.is_some() => {
                let config = config.primary.as_ref().unwrap();

                manager
                    .node()
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
            Replication::Primary => {
                warn!("Primary node started without primary configuration!");
                warn!("Event log will be disabled for this session.");
            }
            Replication::Replica => {
                warn!("Primary node started with replica configuration!");
                warn!("Event log will be disabled for this session.");
            }
        };
    }
}
