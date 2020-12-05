/// Replica error
pub mod error;

/// Replica node storage
pub mod storage;

use super::message::Request;
use crate::{
    config::replication::Replica,
    node::event::EventLog,
    node::{
        replication::message::{PrimaryRequest, ReplicaRequest},
        Manager,
    },
    utils::codec::BincodeCodec,
};
use actix_rt::time::delay_for;
use error::{ReplicaError, ReplicaResult};
use futures_util::{SinkExt, StreamExt};
use maybe_owned::MaybeOwned;
use std::{borrow::Cow, future::Future, time::Duration};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::codec::{Decoder, Framed};

pub struct ReplicaSocket<'m, 'c, T> {
    manager: &'m Manager<'c>,
    config: &'c Replica,
    socket: Framed<T, BincodeCodec>,
}

impl<'m, 'c, T> ReplicaSocket<'m, 'c, T>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    pub fn new(manager: &'m Manager<'c>, config: &'c Replica, socket: T) -> Self {
        ReplicaSocket {
            manager,
            config,
            socket: BincodeCodec::default().framed(socket),
        }
    }

    pub async fn exchange<F, Fut>(&mut self, f: F)
    where
        F: Fn(PrimaryRequest<'static, 'static>, &'m Manager<'c>) -> Fut + Copy,
        Fut: Future<Output = ReplicaRequest<'c>>,
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
                Ok(_) => (),
            }
        }
    }

    async fn process<F, Fut>(&mut self, f: F) -> ReplicaResult<()>
    where
        F: Fn(PrimaryRequest<'static, 'static>, &'m Manager<'c>) -> Fut,
        Fut: Future<Output = ReplicaRequest<'c>>,
    {
        let buf = match self.socket.next().await {
            Some(r) => r.map_err(ReplicaError::CodecError)?,
            None => return Err(ReplicaError::EmptySocket),
        };

        let request = f(
            buf.get_primary().ok_or(ReplicaError::ProtocolMismatch)?,
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

pub async fn accept_connection<'m, 'c>(
    request: PrimaryRequest<'static, 'static>,
    manager: &'m Manager<'c>,
) -> ReplicaRequest<'m> {
    match request {
        PrimaryRequest::Ping => ReplicaRequest::Pong,
        PrimaryRequest::AskIndex => {
            debug!("Preparing indexes for primary node.");
            let mut indexes = Vec::with_capacity(manager.config.queues.len());

            for (name, db) in manager.node.iter() {
                let index = db
                    .replication_storage()
                    .await
                    .as_mut()
                    .expect("No database present")
                    .get_replica()
                    .get_index();

                debug!("Sending {} as confirmed index of {}", index, name);

                indexes.push((Cow::Borrowed(name), index));
            }

            ReplicaRequest::RecvIndex(indexes.into_boxed_slice())
        }
        PrimaryRequest::SendRange(queue, range) => match manager.queue(&queue) {
            Ok(db) => {
                debug!("Applying event slice.");
                let range = range.into_vec();

                let index = range.last().map(|(index, _)| **index);

                db.database()
                    .await
                    .apply_log(range.into_iter().map(|(_, event)| match event {
                        MaybeOwned::Owned(event) => event,
                        MaybeOwned::Borrowed(_) => unreachable!(),
                    }));

                if let Some(index) = index {
                    debug!("Setting {} as confirmed index of {}", index, queue);

                    db.replication_storage()
                        .await
                        .as_mut()
                        .expect("No storage provided")
                        .get_replica()
                        .confirm(index);
                }

                ReplicaRequest::RecvRange
            }
            Err(_) => ReplicaRequest::QueueNotFound(queue),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{accept_connection, error::ReplicaError, storage::ReplicaStorage, ReplicaSocket};
    use crate::{
        config::replication::Replica,
        node::replication::message::Request,
        node::{
            event::Event,
            replication::{
                message::{PrimaryRequest, ReplicaRequest},
                storage::ReplicationStorage,
            },
            Manager,
        },
        utils::{codec::BincodeCodec, stream::TestStream, testing::CONFIG},
    };
    use actix_web::web::BytesMut;
    use bincode::deserialize;
    use maybe_owned::MaybeOwned;
    use std::{
        borrow::Cow,
        net::{IpAddr, Ipv4Addr, SocketAddr},
    };

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
                    Vec::from([(Cow::Borrowed("test"), 1), (Cow::Borrowed("test_2"), 1),])
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
                    Vec::from([(Cow::Borrowed("test"), 3), (Cow::Borrowed("test_2"), 1),])
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

        let request =
            PrimaryRequest::SendRange(Cow::Borrowed(QUEUE_NAME), Vec::from([]).into_boxed_slice());

        let response = accept_connection(request, &manager).await;
        assert_eq!(
            response,
            ReplicaRequest::QueueNotFound(Cow::Borrowed(QUEUE_NAME))
        );
    }

    #[tokio::test]
    async fn test_socket() {
        let manager = Manager::new(&CONFIG);
        let config = Replica {
            host: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 12345),
            try_timer: 1,
        };

        let mut buf = BytesMut::default();
        let stream =
            TestStream::from_output(Request::Primary(PrimaryRequest::Ping), &mut BincodeCodec)
                .unwrap()
                .input(&mut buf);

        {
            let mut socket = ReplicaSocket::new(&manager, &config, stream);
            socket.process(process).await.unwrap();
        }

        assert_eq!(
            deserialize::<Request>(&*buf).unwrap(),
            Request::Replica(ReplicaRequest::Pong)
        );
    }

    #[tokio::test]
    async fn test_invalid_socket() {
        let manager = Manager::new(&CONFIG);
        let config = Replica {
            host: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 12345),
            try_timer: 1,
        };

        let mut buf = BytesMut::default();
        let stream =
            TestStream::from_output(Request::Replica(ReplicaRequest::Pong), &mut BincodeCodec)
                .unwrap()
                .input(&mut buf);

        let mut socket = ReplicaSocket::new(&manager, &config, stream);
        assert_eq!(
            socket.process(process).await.unwrap_err(),
            ReplicaError::ProtocolMismatch
        );
    }

    async fn process<'a>(
        req: PrimaryRequest<'static, 'static>,
        _: &Manager<'a>,
    ) -> ReplicaRequest<'a> {
        assert_eq!(req, PrimaryRequest::Ping);
        ReplicaRequest::Pong
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
