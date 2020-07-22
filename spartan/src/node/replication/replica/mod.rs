/// Replica error
pub mod error;

/// Replica node storage
pub mod storage;

use crate::{
    config::replication::Replica,
    node::{
        replication::{
            event::ApplyEvent,
            message::{PrimaryRequest, ReplicaRequest},
        },
        Manager,
    },
};
use actix_rt::time::delay_for;
use bincode::{deserialize, serialize};
use error::{ReplicaError, ReplicaResult};
use futures_util::{SinkExt, StreamExt};
use std::{future::Future, time::Duration};
use tokio::net::TcpStream;
use tokio_util::codec::{BytesCodec, Decoder, Framed};

pub struct ReplicaSocket<'a> {
    manager: &'a Manager<'a>,
    config: &'a Replica,
    socket: Framed<TcpStream, BytesCodec>,
}

impl<'a> ReplicaSocket<'a> {
    pub fn new(manager: &'a Manager<'a>, config: &'a Replica, socket: TcpStream) -> Self {
        ReplicaSocket {
            manager,
            config,
            socket: BytesCodec::new().framed(socket),
        }
    }

    pub async fn exchange<F, Fut>(&mut self, f: F)
    where
        F: Fn(PrimaryRequest<'static>, &'a Manager<'a>) -> Fut + Copy,
        Fut: Future<Output = ReplicaRequest<'a>>,
    {
        let timer = Duration::from_secs(self.config.try_timer);

        loop {
            delay_for(timer).await;

            match self.process(f).await {
                Err(ReplicaError::EmptySocket) => {
                    error!("Empty TCP socket");
                    return;
                }
                Err(e) => error!("Error occured during replication process: {}", e),
                _ => (),
            }
        }
    }

    async fn process<F, Fut>(&mut self, f: F) -> ReplicaResult<()>
    where
        F: Fn(PrimaryRequest<'static>, &'a Manager<'a>) -> Fut,
        Fut: Future<Output = ReplicaRequest<'a>>,
    {
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

        Ok(())
    }
}

pub async fn accept_connection<'a>(
    request: PrimaryRequest<'static>,
    manager: &Manager<'a>,
) -> ReplicaRequest<'a> {
    match request {
        PrimaryRequest::Ping => ReplicaRequest::Pong,
        PrimaryRequest::AskIndex => {
            debug!("Preparing indexes for primary node.");
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
                debug!("Applying event slice.");
                db.apply_events(range);
                ReplicaRequest::RecvRange
            }
            Err(_) => ReplicaRequest::QueueNotFound(queue),
        },
    }
}
