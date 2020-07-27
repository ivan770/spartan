/// Replica error
pub mod error;

/// Replica node storage
pub mod storage;

use super::message::Request;
use crate::{
    config::replication::Replica,
    node::{
        replication::{
            event::ApplyEvent,
            message::{PrimaryRequest, ReplicaRequest},
        },
        Manager,
    },
    utils::codec::BincodeCodec,
};
use actix_rt::time::delay_for;
use error::{ReplicaError, ReplicaResult};
use futures_util::{SinkExt, StreamExt};
use std::{future::Future, time::Duration};
use tokio::net::TcpStream;
use tokio_util::codec::{Decoder, Framed};

pub struct ReplicaSocket<'a> {
    manager: &'a Manager<'a>,
    config: &'a Replica,
    socket: Framed<TcpStream, BincodeCodec>,
}

impl<'a> ReplicaSocket<'a> {
    pub fn new(manager: &'a Manager<'a>, config: &'a Replica, socket: TcpStream) -> Self {
        ReplicaSocket {
            manager,
            config,
            socket: BincodeCodec::default().framed(socket),
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
            Some(r) => r.map_err(ReplicaError::CodecError)?,
            None => return Err(ReplicaError::EmptySocket),
        };

        let request = f(
            buf.get_primary()
                .ok_or_else(|| ReplicaError::ProtocolMismatch)?,
            self.manager,
        )
        .await;

        self.socket
            .send(Request::Replica(request))
            .await
            .map_err(ReplicaError::CodecError)?;

        SinkExt::<Request>::flush(&mut self.socket)
            .await
            .map_err(ReplicaError::CodecError)?;

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

#[cfg(test)]
mod tests {
    use super::{accept_connection, storage::ReplicaStorage};
    use crate::{
        node::{
            replication::{
                event::Event,
                message::{PrimaryRequest, ReplicaRequest},
                storage::ReplicationStorage,
            },
            Manager,
        },
        utils::testing::CONFIG,
    };
    use maybe_owned::MaybeOwned;
    use std::borrow::Cow;

    #[tokio::test]
    async fn test_accept_ping() {
        let manager = Manager::new(&CONFIG);
        let response = accept_connection(PrimaryRequest::Ping, &manager).await;
        assert_eq!(response, ReplicaRequest::Pong);
    }

    #[tokio::test]
    async fn test_accept_ask() {
        let mut manager = Manager::new(&CONFIG);
        prepare_manager(&mut manager).await;

        let response = accept_connection(PrimaryRequest::AskIndex, &manager).await;

        match response {
            ReplicaRequest::RecvIndex(index) => {
                let mut index = index.into_vec();
                index.sort();
                assert_eq!(
                    index,
                    Vec::from([
                        (String::from("test").into_boxed_str(), 1),
                        (String::from("test_2").into_boxed_str(), 1),
                    ])
                );
            }
            _ => panic!("Invalid response"),
        }
    }

    #[tokio::test]
    async fn test_accept_send() {
        let mut manager = Manager::new(&CONFIG);
        prepare_manager(&mut manager).await;

        let request = PrimaryRequest::SendRange(
            Cow::Borrowed("test"),
            Vec::from([
                (MaybeOwned::Owned(1), MaybeOwned::Owned(Event::Clear)),
                (MaybeOwned::Owned(2), MaybeOwned::Owned(Event::Clear)),
            ])
            .into_boxed_slice(),
        );

        let response = accept_connection(request, &manager).await;
        assert_eq!(response, ReplicaRequest::RecvRange);

        let response = accept_connection(PrimaryRequest::AskIndex, &manager).await;
        match response {
            ReplicaRequest::RecvIndex(index) => {
                let mut index = index.into_vec();
                index.sort();
                assert_eq!(
                    index,
                    Vec::from([
                        (String::from("test").into_boxed_str(), 3),
                        (String::from("test_2").into_boxed_str(), 1),
                    ])
                );
            }
            _ => panic!("Invalid response"),
        }
    }

    #[tokio::test]
    async fn test_accept_invalid_send() {
        const QUEUE_NAME: &str = "this_queue_doesnt_exist";

        let mut manager = Manager::new(&CONFIG);
        prepare_manager(&mut manager).await;

        let request = PrimaryRequest::SendRange(
            Cow::Borrowed(QUEUE_NAME),
            Vec::from([]).into_boxed_slice(),
        );

        let response = accept_connection(request, &manager).await;
        assert_eq!(response, ReplicaRequest::QueueNotFound(Cow::Borrowed(QUEUE_NAME)));
    }

    async fn prepare_manager(manager: &mut Manager<'_>) {
        manager
            .node
            .prepare_replication(
                |storage| matches!(storage, ReplicationStorage::Replica(_)),
                || ReplicationStorage::Replica(ReplicaStorage::default()),
            )
            .await;
    }
}
