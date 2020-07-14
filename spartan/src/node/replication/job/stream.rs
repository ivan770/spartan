use super::{
    error::{ReplicationError, ReplicationResult},
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
use futures_util::{SinkExt, StreamExt};
use maybe_owned::MaybeOwned;
use std::borrow::Cow;
use tokio::net::TcpStream;
use tokio_util::codec::{BytesCodec, Decoder, Framed};

pub(super) struct Stream(Framed<TcpStream, BytesCodec>);

pub(super) struct StreamPool(Box<[Stream]>);

impl<'a> Stream {
    fn serialize(message: &PrimaryRequest) -> ReplicationResult<Vec<u8>> {
        serialize(&message).map_err(|e| ReplicationError::SerializationError(e))
    }

    pub async fn exchange(
        &mut self,
        message: &PrimaryRequest<'_>,
    ) -> ReplicationResult<ReplicaRequest> {
        self.0
            .send(Self::serialize(message)?.into())
            .await
            .map_err(|e| ReplicationError::SocketError(e))?;

        self.0
            .flush()
            .await
            .map_err(|e| ReplicationError::SocketError(e))?;

        let buf = match self.0.next().await {
            Some(r) => r.map_err(|e| ReplicationError::SocketError(e))?,
            None => Err(ReplicationError::EmptySocket)?,
        };

        Ok(deserialize(&buf).map_err(|e| ReplicationError::SerializationError(e))?)
    }

    pub async fn ping(&mut self) -> ReplicationResult<()> {
        match self.exchange(&PrimaryRequest::Ping).await? {
            ReplicaRequest::Pong => Ok(()),
            _ => Err(ReplicationError::ProtocolMismatch),
        }
    }

    pub async fn ask(&'a mut self) -> ReplicationResult<RecvIndex<'a>> {
        match self.exchange(&PrimaryRequest::AskIndex).await? {
            ReplicaRequest::RecvIndex(recv) => Ok(RecvIndex::new(self, recv)),
            _ => Err(ReplicationError::ProtocolMismatch),
        }
    }

    pub async fn send_range(
        &mut self,
        queue: &str,
        range: Box<[(MaybeOwned<'a, u64>, MaybeOwned<'a, Event>)]>,
    ) -> ReplicationResult<()> {
        match self
            .exchange(&PrimaryRequest::SendRange(Cow::Borrowed(queue), range))
            .await?
        {
            ReplicaRequest::RecvRange => Ok(()),
            _ => Err(ReplicationError::ProtocolMismatch),
        }
    }
}

impl<'a> StreamPool {
    pub async fn from_config(config: &Primary) -> ReplicationResult<StreamPool> {
        let mut pool = Vec::with_capacity(config.destination.len());

        for host in &*config.destination {
            pool.push(Stream(
                BytesCodec::new().framed(
                    TcpStream::connect(host)
                        .await
                        .map_err(|e| ReplicationError::SocketError(e))?,
                ),
            ));
        }

        Ok(StreamPool(pool.into_boxed_slice()))
    }

    pub async fn ping(&mut self) -> ReplicationResult<()> {
        for host in &mut *self.0 {
            host.ping().await?;
        }

        Ok(())
    }

    pub async fn ask(&'a mut self) -> ReplicationResult<BatchAskIndex<'a>> {
        let mut batch = BatchAskIndex::with_capacity(self.0.len());

        for host in &mut *self.0 {
            batch.push(host.ask().await?);
        }

        Ok(batch)
    }
}
