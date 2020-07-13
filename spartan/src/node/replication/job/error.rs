use bincode::ErrorKind;
use thiserror::Error;
use tokio::io::Error as IoError;

#[derive(Error, Debug)]
pub(super) enum ReplicationError {
    #[error("Unable to serialize stream message: {0}")]
    SerializationError(Box<ErrorKind>),
    #[error("TCP connection error: {0}")]
    SocketError(IoError),
    #[error("TCP socket is empty")]
    EmptySocket,
    #[error("Protocol mismatch")]
    ProtocolMismatch,
    #[error("Queue configuration mismatch")]
    QueueConfigMismatch,
}

pub(super) type ReplicationResult<T> = Result<T, ReplicationError>;
