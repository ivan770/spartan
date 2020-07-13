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
use maybe_owned::MaybeOwned;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub(super) struct Stream(TcpStream);

pub(super) struct StreamPool(Box<[Stream]>);

impl<'a> Stream {
    fn serialize(message: &PrimaryRequest) -> ReplicationResult<Vec<u8>> {
        serialize(&message).map_err(|e| ReplicationError::SerializationError(e))
    }

    pub async fn exchange(
        &mut self,
        message: &PrimaryRequest<'_>,
    ) -> ReplicationResult<ReplicaRequest> {
        let (mut receive, mut send) = self.0.split();

        send.write_all(&Self::serialize(message)?)
            .await
            .map_err(|e| ReplicationError::SocketError(e))?;

        // TODO: Optimize by using pre-allocated buffer
        let mut buf = Vec::new();

        receive
            .read_to_end(&mut buf)
            .await
            .map_err(|e| ReplicationError::SocketError(e))?;

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
        range: Box<[(MaybeOwned<'a, u64>, MaybeOwned<'a, Event>)]>,
    ) -> ReplicationResult<()> {
        match self.exchange(&PrimaryRequest::SendRange(range)).await? {
            ReplicaRequest::RecvRange => Ok(()),
            _ => Err(ReplicationError::ProtocolMismatch),
        }
    }
}

impl<'a> StreamPool {
    pub async fn from_config(config: &Primary) -> ReplicationResult<StreamPool> {
        let mut pool = Vec::with_capacity(config.destination.len());

        for host in &config.destination {
            pool.push(Stream(
                TcpStream::connect(host)
                    .await
                    .map_err(|e| ReplicationError::SocketError(e))?,
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
