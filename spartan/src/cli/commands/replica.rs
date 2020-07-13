use crate::{
    cli::Server,
    config::replication::Replication,
    node::{
        load_from_fs,
        persistence::PersistenceError,
        replication::message::{PrimaryRequest, ReplicaRequest},
        Manager,
    },
};
use bincode::{deserialize, serialize, ErrorKind};
use futures_util::{SinkExt, StreamExt};
use std::future::Future;
use structopt::StructOpt;
use thiserror::Error;
use tokio::{
    io::Error as IoError,
    net::{TcpListener, TcpStream},
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
    pub async fn dispatch(&self, server: &Server) -> Result<(), ReplicaCommandError> {
        let config = server.config().expect("Config not loaded");
        let mut manager = Manager::new(config);

        load_from_fs(&mut manager)
            .await
            .map_err(|e| ReplicaCommandError::PersistenceError(e))?;

        let mut socket = match config
            .replication
            .as_ref()
            .ok_or_else(|| ReplicaCommandError::ReplicaConfigNotFound)?
        {
            Replication::Replica(config) => TcpListener::bind(config.host)
                .await
                .map_err(|e| ReplicaCommandError::SocketError(e)),
            _ => Err(ReplicaCommandError::ReplicaConfigNotFound),
        }?;

        loop {
            match socket.accept().await {
                Ok((socket, _)) => {
                    ReplicaSocket::new(&manager, socket)
                        .exchange(accept_connection)
                        .await?
                }
                Err(e) => Err(ReplicaCommandError::SocketError(e))?,
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
        F: FnOnce(PrimaryRequest<'static>, &'a Manager<'a>) -> Fut,
        Fut: Future<Output = ReplicaRequest>,
    {
        let buf = match self.socket.next().await {
            Some(r) => r.map_err(|e| ReplicaCommandError::SocketError(e))?,
            None => Err(ReplicaCommandError::EmptySocket)?,
        };

        let request = f(
            deserialize(&buf).map_err(|e| ReplicaCommandError::SerializationError(e))?,
            self.manager,
        )
        .await;

        self.socket
            .send(
                serialize(&request)
                    .map_err(|e| ReplicaCommandError::SerializationError(e))?
                    .into(),
            )
            .await
            .map_err(|e| ReplicaCommandError::SocketError(e))?;

        self.socket
            .flush()
            .await
            .map_err(|e| ReplicaCommandError::SocketError(e))?;

        Ok(())
    }
}

async fn accept_connection(
    request: PrimaryRequest<'static>,
    manager: &Manager<'_>,
) -> ReplicaRequest {
    match request {
        PrimaryRequest::Ping => {
            info!("Pong!");
            ReplicaRequest::Pong
        }
        _ => unimplemented!(),
    }
}
