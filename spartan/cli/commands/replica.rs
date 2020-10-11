use crate::{
    cli::Server,
    config::replication::Replication,
    jobs::{
        exit::spawn_ctrlc_handler,
        persistence::spawn_persistence,
    },
    node::{
        replication::{
            replica::{
                accept_connection,
                error::{ReplicaError, ReplicaResult},
                storage::ReplicaStorage,
                ReplicaSocket,
            },
            storage::ReplicationStorage,
        },
        Manager,
    },
};
use std::sync::Arc;
use structopt::StructOpt;
use tokio::{net::TcpListener, spawn};

#[derive(StructOpt)]
pub struct ReplicaCommand {}

impl ReplicaCommand {
    pub async fn dispatch(&self, server: &'static Server) -> ReplicaResult<()> {
        let config = server.config().expect("Config not loaded");
        let mut manager = Manager::new(config);

        manager
            .load_from_fs()
            .await;

        let manager = Arc::new(manager);

        let cloned_manager = manager.clone();
        spawn(async move { spawn_ctrlc_handler(&cloned_manager).await });

        let cloned_manager = manager.clone();
        spawn(async move { spawn_persistence(&cloned_manager).await });

        manager
            .node
            .prepare_replication(
                |storage| matches!(storage, ReplicationStorage::Replica(_)),
                || ReplicationStorage::Replica(ReplicaStorage::default()),
            )
            .await;

        let config = match config
            .replication
            .as_ref()
            .ok_or(ReplicaError::ReplicaConfigNotFound)?
        {
            Replication::Replica(config) => Ok(config),
            _ => Err(ReplicaError::ReplicaConfigNotFound),
        }?;

        let mut socket = TcpListener::bind(config.host)
            .await
            .map_err(ReplicaError::SocketError)?;

        loop {
            match socket.accept().await {
                Ok((socket, _)) => {
                    ReplicaSocket::new(&manager, config, socket)
                        .exchange(accept_connection)
                        .await
                }
                Err(e) => error!("Unable to accept TCP connection: {}", e),
            }
        }
    }
}
