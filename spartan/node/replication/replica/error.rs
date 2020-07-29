use crate::jobs::persistence::PersistenceError;
use bincode::ErrorKind;
use thiserror::Error;
use tokio::io::Error as IoError;

#[derive(Error, Debug)]
pub enum ReplicaError {
    #[error("Manager persistence error")]
    PersistenceError(#[from] PersistenceError),
    #[error("Unable to find replica node config")]
    ReplicaConfigNotFound,
    #[error("TCP socket error")]
    SocketError(#[from] IoError),
    #[error("Empty TCP socket")]
    EmptySocket,
    #[error("Socket codec error")]
    CodecError(#[from] Box<ErrorKind>),
    #[error("Protocol mismatch")]
    ProtocolMismatch,
}

pub type ReplicaResult<T> = Result<T, ReplicaError>;
