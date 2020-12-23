use std::borrow::Cow;

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
    #[error(
        "Replica version mismatch. Primary version: {}, replica version: {0}",
        crate::VERSION
    )]
    VersionMismatch(Cow<'static, str>),
    #[error("Replica node requested index below GC threshold")]
    IndexMismatch,
    #[error("Queue configuration mismatch")]
    QueueConfigMismatch,
}

pub type PrimaryResult<T> = Result<T, PrimaryError>;
