use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;

pub mod tree;
pub mod vec;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("message not found")]
    MessageNotFound,
    #[error("not enough storage memory")]
    NotEnoughMemory,
    #[error("db connection unavailable")]
    ConnectionError,
}

pub type Result<T> = std::result::Result<T, DatabaseError>;

pub trait Database<M>: Default {
    type PositionKey;

    fn push_raw(&mut self, message: M) -> Result<()>;
    fn position<F>(&self, predicate: F) -> Result<Self::PositionKey>
    where
        F: Fn(&M) -> bool;
    fn get(&self, position: Self::PositionKey) -> Result<&M>;
    fn get_mut(&mut self, position: Self::PositionKey) -> Result<&mut M>;
    fn delete_pos(&mut self, position: Self::PositionKey) -> Result<()>;
    fn retain<F>(&mut self, predicate: F) -> Result<()>
    where
        F: Fn(&M) -> bool;
    fn len(&self) -> Result<usize>;
    fn clear(&mut self) -> Result<()>;
}
