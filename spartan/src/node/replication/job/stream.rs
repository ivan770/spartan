use crate::config::replication::Primary;
use bincode::{serialize, ErrorKind};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::{
    io::{AsyncWriteExt, Error as IoError},
    net::TcpStream,
};

#[derive(Error, Debug)]
pub enum StreamError {
    #[error("Unable to serialize stream message: {0}")]
    SerializationError(Box<ErrorKind>),
    #[error("TCP connection error: {0}")]
    SocketError(IoError),
}

pub(super) type StreamResult<T> = Result<T, StreamError>;

#[derive(Serialize, Deserialize)]
pub enum StreamMessage {
    Ping,
}

pub(super) struct Stream(TcpStream);

pub(super) struct StreamPool(Vec<Stream>);

impl Stream {
    fn serialize(message: &StreamMessage) -> StreamResult<Vec<u8>> {
        serialize(&message).map_err(|e| StreamError::SerializationError(e))
    }

    pub async fn ping(&mut self) -> Result<(), StreamError> {
        self.0
            .write_all(&Self::serialize(&StreamMessage::Ping)?)
            .await
            .map_err(|e| StreamError::SocketError(e))
    }
}

impl StreamPool {
    pub async fn from_config(config: &Primary) -> StreamResult<StreamPool> {
        let mut pool = Vec::with_capacity(config.destination.len());

        for host in &config.destination {
            pool.push(Stream(
                TcpStream::connect(host)
                    .await
                    .map_err(|e| StreamError::SocketError(e))?,
            ));
        }

        Ok(StreamPool(pool))
    }

    pub async fn ping(&mut self) -> StreamResult<()> {
        for host in &mut self.0 {
            host.ping().await?;
        }

        Ok(())
    }
}
