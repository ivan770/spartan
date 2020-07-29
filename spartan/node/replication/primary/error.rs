use bincode::ErrorKind;
use thiserror::Error;
use tokio::io::Error as IoError;

#[derive(Error, Debug)]
pub enum PrimaryError {
    #[error("Socket codec error")]
    CodecError(#[from] Box<ErrorKind>),
    #[error("TCP connection error")]
    SocketError(#[from] IoError),
    #[error("TCP socket is empty")]
    EmptySocket,
    #[error("Protocol mismatch")]
    ProtocolMismatch,
    #[error("Queue configuration mismatch")]
    QueueConfigMismatch,
}

pub type PrimaryResult<T> = Result<T, PrimaryError>;
