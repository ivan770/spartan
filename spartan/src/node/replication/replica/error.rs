use crate::jobs::persistence::PersistenceError;
use bincode::ErrorKind;
use thiserror::Error;
use tokio::io::Error as IoError;

#[derive(Error, Debug)]
pub enum ReplicaError {
    #[error("Manager persistence error: {0}")]
    PersistenceError(PersistenceError),
    #[error("Unable to find replica node config")]
    ReplicaConfigNotFound,
    #[error("TCP socket error: {0}")]
    SocketError(IoError),
    #[error("Empty TCP socket")]
    EmptySocket,
    #[error("Packet serialization error: {0}")]
    SerializationError(Box<ErrorKind>),
    #[error("Protocol mismatch")]
    ProtocolMismatch,
}

pub type ReplicaResult<T> = Result<T, ReplicaError>;
