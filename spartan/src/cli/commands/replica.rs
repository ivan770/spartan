use crate::{
    cli::Server,
    config::replication::Replication,
    jobs::{persistence::{load_from_fs, PersistenceError, spawn_persistence}, exit::spawn_ctrlc_handler},
    node::{
        replication::{
            event::ApplyEvent,
            message::{PrimaryRequest, ReplicaRequest},
            storage::{replica::ReplicaStorage, ReplicationStorage},
        },
        Manager,
    },
};
use bincode::{deserialize, serialize, ErrorKind};
use futures_util::{SinkExt, StreamExt};
use std::{sync::Arc, future::Future};
use structopt::StructOpt;
use thiserror::Error;
use tokio::{
    io::Error as IoError,
    net::{TcpListener, TcpStream}, spawn,
};
use tokio_util::codec::{BytesCodec, Decoder, Framed};

#[derive(Error, Debug)]
pub enum ReplicaCommandError {
    #[error("Manager persistence error: {0}")]
    PersistenceError(PersistenceError),
    #[error("Unable to find replica node config")]
    ReplicaConfigNotFound,
    #[error("TCP socket error: {0}")]
    SocketError(IoError),
    #[error("Empty TCP socket")]
    EmptySocket,
    #[error("Packet serialization error: {0}")]
    SerializationError(Box<ErrorKind>),
}

#[derive(StructOpt)]
pub struct ReplicaCommand {}

impl ReplicaCommand {
    pub async fn dispatch(&self, server: &'static Server) -> Result<(), ReplicaCommandError> {
        let config = server.config().expect("Config not loaded");
        let mut manager = Manager::new(config);

        load_from_fs(&mut manager)
            .await
            .map_err(ReplicaCommandError::PersistenceError)?;

        let manager = Arc::new(manager);

        let cloned_manager = manager.clone();
        spawn(async move { spawn_ctrlc_handler(&cloned_manager).await });

        let cloned_manager = manager.clone();
        spawn(async move { spawn_persistence(&cloned_manager).await });

        ReplicationStorage::prepare(
            &manager,
            |storage| matches!(storage, ReplicationStorage::Replica(_)),
            || ReplicationStorage::Replica(ReplicaStorage::default()),
        )
        .await;

        let mut socket = match config
            .replication
            .as_ref()
            .ok_or_else(|| ReplicaCommandError::ReplicaConfigNotFound)?
        {
            Replication::Replica(config) => TcpListener::bind(config.host)
                .await
                .map_err(ReplicaCommandError::SocketError),
            _ => Err(ReplicaCommandError::ReplicaConfigNotFound),
        }?;

        loop {
            match socket.accept().await {
                Ok((socket, _)) => {
                    ReplicaSocket::new(&manager, socket)
                        .exchange(accept_connection)
                        .await?
                }
                Err(e) => return Err(ReplicaCommandError::SocketError(e)),
            }
        }
    }
}

struct ReplicaSocket<'a> {
    manager: &'a Manager<'a>,
    socket: Framed<TcpStream, BytesCodec>,
}

impl<'a> ReplicaSocket<'a> {
    fn new(manager: &'a Manager<'a>, socket: TcpStream) -> Self {
        ReplicaSocket {
            manager,
            socket: BytesCodec::new().framed(socket),
        }
    }

    async fn exchange<F, Fut>(&mut self, f: F) -> Result<(), ReplicaCommandError>
    where
        F: Fn(PrimaryRequest<'static>, &'a Manager<'a>) -> Fut,
        Fut: Future<Output = ReplicaRequest<'a>>,
    {
        loop {
            let buf = match self.socket.next().await {
                Some(r) => r.map_err(ReplicaCommandError::SocketError)?,
                None => return Err(ReplicaCommandError::EmptySocket),
            };

            let request = f(
                deserialize(&buf).map_err(ReplicaCommandError::SerializationError)?,
                self.manager,
            )
            .await;

            self.socket
                .send(
                    serialize(&request)
                        .map_err(ReplicaCommandError::SerializationError)?
                        .into(),
                )
                .await
                .map_err(ReplicaCommandError::SocketError)?;

            self.socket
                .flush()
                .await
                .map_err(ReplicaCommandError::SocketError)?;
        }
    }
}

async fn accept_connection<'a>(
    request: PrimaryRequest<'static>,
    manager: &Manager<'a>,
) -> ReplicaRequest<'a> {
    match request {
        PrimaryRequest::Ping => ReplicaRequest::Pong,
        PrimaryRequest::AskIndex => {
            let mut indexes = Vec::with_capacity(manager.config.queues.len());

            for (name, db) in manager.node.iter() {
                let index = db
                    .lock()
                    .await
                    .get_storage()
                    .as_mut()
                    .expect("No database present")
                    .get_replica()
                    .get_index();

                indexes.push((name.to_string().into_boxed_str(), index));
            }

            ReplicaRequest::RecvIndex(indexes.into_boxed_slice())
        }
        PrimaryRequest::SendRange(queue, range) => match manager.queue(&queue).await {
            Ok(mut db) => {
                db.apply_events(range);
                ReplicaRequest::RecvRange
            }
            Err(_) => ReplicaRequest::QueueNotFound(queue),
        },
    }
}
