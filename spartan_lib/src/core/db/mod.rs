use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;

pub mod vec;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("message not found")]
    MessageNotFound,
    #[error("not enough storage memory")]
    NotEnoughMemory,
    #[error("db connection unavailable")]
    ConnectionError
}

pub type Result<T> = std::result::Result<T, DatabaseError>;

pub trait Database<M>: Default {
    fn push_raw(&mut self, message: M) -> Result<()>;
    fn position<F>(&self, predicate: F) -> Result<usize>
    where
        F: Fn(&M) -> bool;
    fn get(&self, position: usize) -> Result<&M>;
    fn get_mut(&mut self, position: usize) -> Result<&mut M>;
    fn delete_pos(&mut self, position: usize) -> Result<()>;
    fn retain<F>(&mut self, predicate: F) -> Result<()>
    where
        F: Fn(&M) -> bool;
    fn len(&self) -> Result<usize>;
    fn clear(&mut self) -> Result<()>;
}

pub trait SerializableDatabase<M>: Database<M>
where
    M: Serialize + DeserializeOwned,
{
    type DB: Serialize + DeserializeOwned;

    fn get_db(&self) -> &Self::DB;
    fn set_db(&mut self, db: Self::DB);
}
