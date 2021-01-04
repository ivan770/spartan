use std::borrow::Cow;

use futures_util::{stream::iter, SinkExt, StreamExt, TryStreamExt};
use maybe_owned::MaybeOwned;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpStream,
};
use tokio_util::codec::{Decoder, Framed};

use crate::{
    config::replication::Primary,
    node::{
        event::Event,
        replication::{
            message::{PrimaryRequest, ReplicaRequest, Request},
            primary::{
                error::{PrimaryError, PrimaryResult},
                index::{BatchAskIndex, RecvIndex},
            },
        },
    },
    utils::codec::BincodeCodec,
};

pub struct Stream<T>(Framed<T, BincodeCodec>);

pub struct StreamPool<T>(Box<[Stream<T>]>);

impl<T> Stream<T>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    pub async fn exchange(
        &mut self,
        message: PrimaryRequest<'_, '_>,
    ) -> PrimaryResult<ReplicaRequest<'static>> {
        self.0
            .send(Request::Primary(message))
            .await
            .map_err(PrimaryError::CodecError)?;

        SinkExt::<Request>::flush(&mut self.0)
            .await
            .map_err(PrimaryError::CodecError)?;

        let buf = match self.0.next().await {
            Some(r) => r.map_err(PrimaryError::CodecError)?,
            None => return Err(PrimaryError::EmptySocket),
        };

        buf.get_replica().ok_or(PrimaryError::ProtocolMismatch)
    }

    async fn ping(&mut self) -> PrimaryResult<()> {
        match self.exchange(PrimaryRequest::Ping).await? {
            ReplicaRequest::Pong(version) => {
                if version == crate::VERSION {
                    Ok(())
                } else {
                    Err(PrimaryError::VersionMismatch(version))
                }
            }
            _ => Err(PrimaryError::ProtocolMismatch),
        }
    }

    async fn ask(&mut self) -> PrimaryResult<RecvIndex<'_, T>> {
        match self.exchange(PrimaryRequest::AskIndex).await? {
            ReplicaRequest::RecvIndex(recv) => Ok(RecvIndex::new(self, recv)),
            _ => Err(PrimaryError::ProtocolMismatch),
        }
    }

    pub(super) async fn send_range<'r>(
        &mut self,
        queue: &str,
        range: Box<[(MaybeOwned<'r, u64>, MaybeOwned<'r, Event<'r>>)]>,
    ) -> PrimaryResult<()> {
        match self
            .exchange(PrimaryRequest::SendRange(Cow::Borrowed(queue), range))
            .await?
        {
            ReplicaRequest::RecvRange => Ok(()),
            ReplicaRequest::QueueNotFound(queue) => {
                warn!("Queue {} not found on replica", queue);
                Ok(())
            }
            _ => Err(PrimaryError::ProtocolMismatch),
        }
    }
}

impl StreamPool<TcpStream> {
    pub async fn from_config(config: &Primary) -> PrimaryResult<Self> {
        let mut pool = Vec::with_capacity(config.destination.len());

        for host in &*config.destination {
            pool.push(
                TcpStream::connect(host)
                    .await
                    .map_err(PrimaryError::SocketError)?,
            );
        }

        Ok(StreamPool::new(pool).await)
    }
}

impl<T> StreamPool<T>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    pub async fn new<P>(pool: P) -> Self
    where
        P: IntoIterator<Item = T>,
    {
        let pool = pool
            .into_iter()
            .map(|stream| Stream(BincodeCodec::default().framed(stream)))
            .collect::<Box<[_]>>();

        StreamPool(pool)
    }

    pub async fn ping(&mut self) -> PrimaryResult<()> {
        debug!("Pinging stream pool.");

        iter(self.0.iter_mut())
            .map(Ok)
            .try_for_each_concurrent(None, Stream::ping)
            .await?;

        Ok(())
    }

    pub async fn ask(&mut self) -> PrimaryResult<BatchAskIndex<'_, T>> {
        let len = self.0.len();

        debug!("Asking stream pool of {} nodes for indexes.", len);

        let mut batch = BatchAskIndex::with_capacity(len);

        for host in &mut *self.0 {
            batch.push(host.ask().await?);
        }

        Ok(batch)
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use bincode::deserialize;
    use bytes::BytesMut;

    use super::StreamPool;
    use crate::{
        node::replication::{
            message::{PrimaryRequest, ReplicaRequest, Request},
            primary::error::PrimaryError,
        },
        utils::{codec::BincodeCodec, stream::TestStream},
    };

    #[tokio::test]
    async fn test_ping() {
        let mut buf = BytesMut::default();
        let stream = vec![TestStream::from_output(
            Request::Replica(ReplicaRequest::Pong(Cow::Borrowed(crate::VERSION))),
            &mut BincodeCodec,
        )
        .unwrap()
        .input(&mut buf)];

        StreamPool::new(stream).await.ping().await.unwrap();
        assert_eq!(
            deserialize::<Request>(&*buf).unwrap(),
            Request::Primary(PrimaryRequest::Ping)
        );
    }

    #[tokio::test]
    async fn test_ping_invalid_version() {
        let mut buf = BytesMut::default();
        let stream = vec![TestStream::from_output(
            Request::Replica(ReplicaRequest::Pong(Cow::Borrowed("0.0.0"))),
            &mut BincodeCodec,
        )
        .unwrap()
        .input(&mut buf)];

        assert!(matches!(
            StreamPool::new(stream).await.ping().await.unwrap_err(),
            PrimaryError::VersionMismatch(_)
        ));
    }

    #[tokio::test]
    async fn test_ask() {
        let mut buf = BytesMut::default();
        let stream = vec![TestStream::from_output(
            Request::Replica(ReplicaRequest::RecvIndex(
                vec![(Cow::Borrowed("test"), 123)].into_boxed_slice(),
            )),
            &mut BincodeCodec,
        )
        .unwrap()
        .input(&mut buf)];

        StreamPool::new(stream).await.ask().await.unwrap();
        assert_eq!(
            deserialize::<Request>(&*buf).unwrap(),
            Request::Primary(PrimaryRequest::AskIndex)
        );
    }

    // TODO: TestStream with multiple input and output buffers
    // #[tokio::test]
    // async fn test_send_range() {
    //     let mut buf = BytesMut::default();
    //     let stream = vec![TestStream::from_output(
    //         Request::Replica(ReplicaRequest::RecvIndex(
    //             vec![(String::from("test").into_boxed_str(), 0)].into_boxed_slice(),
    //         )),
    //         &mut BincodeCodec,
    //     )
    //     .unwrap()
    //     .input(&mut buf)];

    //     let manager = Manager::new(&CONFIG);
    //     manager
    //         .queue("test")
    //         .unwrap()
    //         .prepare_replication(
    //             |_| false,
    //             || ReplicationStorage::Primary(PrimaryStorage::default()),
    //         )
    //         .await;
    //     manager.queue("test").unwrap().database().await.gc();

    //     StreamPool::new(stream)
    //         .await
    //         .ask()
    //         .await
    //         .unwrap()
    //         .sync(&manager)
    //         .await
    //         .unwrap();
    // }
}
