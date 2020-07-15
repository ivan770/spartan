use super::{
    error::{PrimaryError, PrimaryResult},
    index::{BatchAskIndex, RecvIndex},
};
use crate::{
    config::replication::Primary,
    node::replication::{
        event::Event,
        message::{PrimaryRequest, ReplicaRequest},
    },
};
use bincode::{deserialize, serialize};
use futures_util::{stream::iter, SinkExt, StreamExt, TryStreamExt};
use maybe_owned::MaybeOwned;
use std::borrow::Cow;
use tokio::net::TcpStream;
use tokio_util::codec::{BytesCodec, Decoder, Framed};

pub struct Stream(Framed<TcpStream, BytesCodec>);

pub struct StreamPool(Box<[Stream]>);

impl<'a> Stream {
    fn serialize(message: &PrimaryRequest) -> PrimaryResult<Vec<u8>> {
        serialize(&message).map_err(PrimaryError::SerializationError)
    }

    pub async fn exchange(
        &mut self,
        message: &PrimaryRequest<'_>,
    ) -> PrimaryResult<ReplicaRequest<'_>> {
        self.0
            .send(Self::serialize(message)?.into())
            .await
            .map_err(PrimaryError::SocketError)?;

        self.0.flush().await.map_err(PrimaryError::SocketError)?;

        let buf = match self.0.next().await {
            Some(r) => r.map_err(PrimaryError::SocketError)?,
            None => return Err(PrimaryError::EmptySocket),
        };

        Ok(deserialize(&buf).map_err(PrimaryError::SerializationError)?)
    }

    async fn ping(&mut self) -> PrimaryResult<()> {
        match self.exchange(&PrimaryRequest::Ping).await? {
            ReplicaRequest::Pong => Ok(()),
            _ => Err(PrimaryError::ProtocolMismatch),
        }
    }

    async fn ask(&'a mut self) -> PrimaryResult<RecvIndex<'a>> {
        match self.exchange(&PrimaryRequest::AskIndex).await? {
            ReplicaRequest::RecvIndex(recv) => Ok(RecvIndex::new(self, recv)),
            _ => Err(PrimaryError::ProtocolMismatch),
        }
    }

    pub(super) async fn send_range(
        &mut self,
        queue: &str,
        range: Box<[(MaybeOwned<'a, u64>, MaybeOwned<'a, Event>)]>,
    ) -> PrimaryResult<()> {
        match self
            .exchange(&PrimaryRequest::SendRange(Cow::Borrowed(queue), range))
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

impl<'a> StreamPool {
    pub async fn from_config(config: &Primary) -> PrimaryResult<StreamPool> {
        let mut pool = Vec::with_capacity(config.destination.len());

        for host in &*config.destination {
            pool.push(Stream(
                BytesCodec::new().framed(
                    TcpStream::connect(host)
                        .await
                        .map_err(PrimaryError::SocketError)?,
                ),
            ));
        }

        Ok(StreamPool(pool.into_boxed_slice()))
    }

    pub async fn ping(&mut self) -> PrimaryResult<()> {
        iter(self.0.iter_mut())
            .map(Ok)
            .try_for_each_concurrent(None, |stream| async move { stream.ping().await })
            .await?;

        Ok(())
    }

    pub async fn ask(&'a mut self) -> PrimaryResult<BatchAskIndex<'a>> {
        let mut batch = BatchAskIndex::with_capacity(self.0.len());

        for host in &mut *self.0 {
            batch.push(host.ask().await?);
        }

        Ok(batch)
    }
}
