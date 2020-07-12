use super::index::{BatchAskIndex, RecvIndex};
use crate::config::replication::Primary;
use bincode::{deserialize, serialize, ErrorKind};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, Error as IoError},
    net::TcpStream,
};

#[derive(Error, Debug)]
pub enum StreamError {
    #[error("Unable to serialize stream message: {0}")]
    SerializationError(Box<ErrorKind>),
    #[error("TCP connection error: {0}")]
    SocketError(IoError),
    #[error("Protocol mismatch")]
    ProtocolMismatch,
}

pub(super) type StreamResult<T> = Result<T, StreamError>;

#[derive(Serialize, Deserialize)]
pub enum StreamMessage {
    Ping,
    AskIndex,
    RecvIndex(Vec<(Box<str>, u64)>),
}

pub(super) struct Stream(TcpStream);

pub(super) struct StreamPool(Box<[Stream]>);

impl<'a> Stream {
    fn serialize(message: &StreamMessage) -> StreamResult<Vec<u8>> {
        serialize(&message).map_err(|e| StreamError::SerializationError(e))
    }

    pub async fn ping(&mut self) -> StreamResult<()> {
        self.0
            .write_all(&Self::serialize(&StreamMessage::Ping)?)
            .await
            .map_err(|e| StreamError::SocketError(e))
    }

    pub async fn ask(&'a mut self) -> StreamResult<RecvIndex<'a>> {
        let (mut receive, mut send) = self.0.split();

        send.write_all(&Self::serialize(&StreamMessage::AskIndex)?)
            .await
            .map_err(|e| StreamError::SocketError(e))?;

        // TODO: Optimize by using pre-allocated buffer
        let mut buf = Vec::new();

        receive
            .read_to_end(&mut buf)
            .await
            .map_err(|e| StreamError::SocketError(e))?;

        let buf: StreamMessage =
            deserialize(&buf).map_err(|e| StreamError::SerializationError(e))?;

        match buf {
            StreamMessage::RecvIndex(recv) => Ok(RecvIndex::new(self, recv.into_boxed_slice())),
            _ => Err(StreamError::ProtocolMismatch),
        }
    }
}

impl<'a> StreamPool {
    pub async fn from_config(config: &Primary) -> StreamResult<StreamPool> {
        let mut pool = Vec::with_capacity(config.destination.len());

        for host in &config.destination {
            pool.push(Stream(
                TcpStream::connect(host)
                    .await
                    .map_err(|e| StreamError::SocketError(e))?,
            ));
        }

        Ok(StreamPool(pool.into_boxed_slice()))
    }

    pub async fn ping(&mut self) -> StreamResult<()> {
        for host in &mut *self.0 {
            host.ping().await?;
        }

        Ok(())
    }

    pub async fn ask(&'a mut self) -> StreamResult<BatchAskIndex<'a>> {
        let mut batch = BatchAskIndex::with_capacity(self.0.len());

        for host in &mut *self.0 {
            batch.push(host.ask().await?);
        }

        Ok(batch)
    }
}
