use std::sync::Arc;

use structopt::StructOpt;
use tokio::net::TcpListener;

use crate::{
    cli::Server,
    dispatch_jobs,
    jobs::{exit::spawn_ctrlc_handler, persistence::spawn_persistence},
    node::{
        persistence::PersistenceError,
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

#[derive(StructOpt)]
pub struct ReplicaCommand {}

impl ReplicaCommand {
    pub async fn dispatch(&self, server: &'static Server) -> ReplicaResult<()> {
        let config = server.config().ok_or(ReplicaError::ReplicaConfigNotFound)?;
        let mut manager = Manager::new(config);

        match manager.load_from_fs().await {
            Err(PersistenceError::FileOpenError(e)) => error!("Unable to load database: {}", e),
            Err(e) => Err(e).map_err(ReplicaError::PersistenceError)?,
            Ok(_) => (),
        };

        let manager = Arc::new(manager);

        dispatch_jobs!(manager, spawn_ctrlc_handler, spawn_persistence);

        manager
            .node()
            .prepare_replication(
                |storage| matches!(storage, ReplicationStorage::Replica(_)),
                || ReplicationStorage::Replica(ReplicaStorage::default()),
            )
            .await;

        let config = config
            .replication
            .as_ref()
            .ok_or(ReplicaError::ReplicaConfigNotFound)?
            .replica
            .as_ref()
            .ok_or(ReplicaError::ReplicaConfigNotFound)?;

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
