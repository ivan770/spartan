use crate::core::db::{Database, DatabaseError, Result, SerializableDatabase};
use serde::{de::DeserializeOwned, Serialize};

#[derive(Debug)]
pub struct VecDatabase<M> {
    db: Vec<M>,
}

impl<M> SerializableDatabase<M> for VecDatabase<M>
where
    M: Serialize + DeserializeOwned,
{
    type DB = Vec<M>;

    fn get_db(&self) -> &Self::DB {
        &self.db
    }

    fn set_db(&mut self, db: Self::DB) {
        self.db = db;
    }
}

impl<M> Default for VecDatabase<M> {
    fn default() -> Self {
        VecDatabase { db: Vec::new() }
    }
}

impl<M> Database<M> for VecDatabase<M>
where
    M: Serialize + DeserializeOwned,
{
    fn push_raw(&mut self, message: M) -> Result<()> {
        self.db.push(message);
        Ok(())
    }

    fn position<F>(&self, predicate: F) -> Result<usize>
    where
        F: Fn(&M) -> bool,
    {
        self.db
            .iter()
            .position(predicate)
            .ok_or(DatabaseError::MessageNotFound)
    }

    fn get(&self, position: usize) -> Result<&M> {
        self.db.get(position).ok_or(DatabaseError::MessageNotFound)
    }

    fn get_mut(&mut self, position: usize) -> Result<&mut M> {
        self.db
            .get_mut(position)
            .ok_or(DatabaseError::MessageNotFound)
    }

    fn delete_pos(&mut self, position: usize) -> Result<()> {
        if let Some(_) = self.db.get(position) {
            self.db.remove(position);
            Ok(())
        } else {
            Err(DatabaseError::MessageNotFound)
        }
    }

    fn retain<F>(&mut self, predicate: F) -> Result<()>
    where
        F: Fn(&M) -> bool,
    {
        self.db.retain(predicate);
        Ok(())
    }

    fn len(&self) -> Result<usize> {
        Ok(self.db.len())
    }

    fn clear(&mut self) -> Result<()> {
        self.db.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Database, VecDatabase};
    use crate::core::message::{builder::MessageBuilder, Message};

    fn create_message() -> Message {
        MessageBuilder::default()
            .body(b"Hello world")
            .compose()
            .unwrap()
    }

    fn create_database() -> VecDatabase<Message> {
        VecDatabase::<Message>::default()
    }

    #[test]
    fn test_push_raw() {
        let mut db = create_database();
        let message = create_message();
        db.push_raw(message).unwrap();
        assert_eq!(db.len().unwrap(), 1);
    }

    #[test]
    fn test_position() {
        let mut db = create_database();
        let message = create_message();
        db.push_raw(message.clone()).unwrap();
        assert_eq!(db.position(|msg| msg.id == message.id).unwrap(), 0);
    }

    #[test]
    fn test_get() {
        let mut db = create_database();
        let message = create_message();
        db.push_raw(message.clone()).unwrap();
        assert_eq!(db.get(0).unwrap().id, message.id);
    }

    #[test]
    fn test_get_mut() {
        let mut db = create_database();
        let message = create_message();
        db.push_raw(message.clone()).unwrap();
        assert_eq!(db.get_mut(0).unwrap().id, message.id);
    }

    #[test]
    fn test_delete() {
        let mut db = create_database();
        let message = create_message();
        db.push_raw(message.clone()).unwrap();
        db.delete_pos(0).unwrap();
        assert!(db.delete_pos(0).is_err());
    }

    #[test]
    fn test_retain() {
        let mut db = create_database();
        let message = create_message();
        let message2 = create_message();
        db.push_raw(message.clone()).unwrap();
        db.push_raw(message2).unwrap();
        assert_eq!(db.len().unwrap(), 2);
        db.retain(|msg| msg.id == message.id).unwrap();
        assert_eq!(db.len().unwrap(), 1);
    }

    #[test]
    fn test_clear() {
        let mut db = create_database();
        db.push_raw(create_message()).unwrap();
        db.push_raw(create_message()).unwrap();
        assert_eq!(db.len().unwrap(), 2);
        db.clear().unwrap();
        assert_eq!(db.len().unwrap(), 0);
    }
}
