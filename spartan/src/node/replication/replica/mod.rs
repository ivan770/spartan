/// Replica error
pub mod error;

use crate::node::{
    replication::{
        event::ApplyEvent,
        message::{PrimaryRequest, ReplicaRequest},
    },
    Manager,
};
use bincode::{deserialize, serialize};
use error::{ReplicaError, ReplicaResult};
use futures_util::{SinkExt, StreamExt};
use std::future::Future;
use tokio::net::TcpStream;
use tokio_util::codec::{BytesCodec, Decoder, Framed};

pub struct ReplicaSocket<'a> {
    manager: &'a Manager<'a>,
    socket: Framed<TcpStream, BytesCodec>,
}

impl<'a> ReplicaSocket<'a> {
    pub fn new(manager: &'a Manager<'a>, socket: TcpStream) -> Self {
        ReplicaSocket {
            manager,
            socket: BytesCodec::new().framed(socket),
        }
    }

    pub async fn exchange<F, Fut>(&mut self, f: F) -> ReplicaResult<()>
    where
        F: Fn(PrimaryRequest<'static>, &'a Manager<'a>) -> Fut,
        Fut: Future<Output = ReplicaRequest<'a>>,
    {
        loop {
            let buf = match self.socket.next().await {
                Some(r) => r.map_err(ReplicaError::SocketError)?,
                None => return Err(ReplicaError::EmptySocket),
            };

            let request = f(
                deserialize(&buf).map_err(ReplicaError::SerializationError)?,
                self.manager,
            )
            .await;

            self.socket
                .send(
                    serialize(&request)
                        .map_err(ReplicaError::SerializationError)?
                        .into(),
                )
                .await
                .map_err(ReplicaError::SocketError)?;

            self.socket
                .flush()
                .await
                .map_err(ReplicaError::SocketError)?;
        }
    }
}

pub async fn accept_connection<'a>(
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
