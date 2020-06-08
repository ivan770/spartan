use super::StatusAwareDatabase;
use crate::core::db::Database;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct VecDatabase<M> {
    db: Vec<M>,
}

impl<M> Default for VecDatabase<M> {
    fn default() -> Self {
        VecDatabase { db: Vec::new() }
    }
}

impl<M> Database<M> for VecDatabase<M> {
    type PositionKey = usize;

    fn push_raw(&mut self, message: M) {
        self.db.push(message);
    }

    fn position<F>(&self, predicate: F) -> Option<Self::PositionKey>
    where
        F: Fn(&M) -> bool,
    {
        Some(self.db.iter().position(predicate)?)
    }

    fn get(&self, position: Self::PositionKey) -> Option<&M> {
        Some(self.db.get(position)?)
    }

    fn get_mut(&mut self, position: Self::PositionKey) -> Option<&mut M> {
        Some(self.db.get_mut(position)?)
    }

    fn delete_pos(&mut self, position: Self::PositionKey) -> Option<()> {
        if self.db.get(position).is_some() {
            self.db.remove(position);
            Some(())
        } else {
            None
        }
    }

    fn retain<F>(&mut self, predicate: F)
    where
        F: Fn(&M) -> bool,
    {
        self.db.retain(predicate);
    }

    fn len(&self) -> usize {
        self.db.len()
    }

    fn is_empty(&self) -> bool {
        self.db.is_empty()
    }

    fn clear(&mut self) {
        self.db.clear();
        self.db.shrink_to_fit();
    }
}

impl<M> StatusAwareDatabase<M> for VecDatabase<M> {
    type RequeueKey = usize;

    fn reserve(&mut self, position: Self::PositionKey) -> Option<&mut M> {
        Some(self.db.get_mut(position)?)
    }

    fn requeue<F>(&mut self, position: Self::RequeueKey, predicate: F) -> Option<&mut M>
    where
        F: Fn(&M) -> bool,
    {
        let message = self.reserve(position)?;

        if predicate(message) {
            Some(message)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Database, VecDatabase};
    use crate::core::message::{builder::MessageBuilder, Message};

    fn create_message() -> Message {
        MessageBuilder::default()
            .body("Hello world")
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
        db.push_raw(message);
        assert_eq!(db.len(), 1);
    }

    #[test]
    fn test_position() {
        let mut db = create_database();
        let message = create_message();
        db.push_raw(message.clone());
        assert_eq!(db.position(|msg| msg.id == message.id).unwrap(), 0);
    }

    #[test]
    fn test_get() {
        let mut db = create_database();
        let message = create_message();
        db.push_raw(message.clone());
        assert_eq!(db.get(0).unwrap().id, message.id);
    }

    #[test]
    fn test_get_mut() {
        let mut db = create_database();
        let message = create_message();
        db.push_raw(message.clone());
        assert_eq!(db.get_mut(0).unwrap().id, message.id);
    }

    #[test]
    fn test_delete() {
        let mut db = create_database();
        let message = create_message();
        db.push_raw(message.clone());
        db.delete_pos(0).unwrap();
        assert!(db.delete_pos(0).is_none());
    }

    #[test]
    fn test_retain() {
        let mut db = create_database();
        let message = create_message();
        let message2 = create_message();
        db.push_raw(message.clone());
        db.push_raw(message2);
        assert_eq!(db.len(), 2);
        db.retain(|msg| msg.id == message.id);
        assert_eq!(db.len(), 1);
    }

    #[test]
    fn test_clear() {
        let mut db = create_database();
        db.push_raw(create_message());
        db.push_raw(create_message());
        assert_eq!(db.len(), 2);
        db.clear();
        assert_eq!(db.len(), 0);
    }

    #[test]
    fn test_is_empty() {
        let mut db = create_database();
        assert!(db.is_empty());
        db.push_raw(create_message());
        assert!(!db.is_empty());
    }
}

#[cfg(test)]
mod dispatcher_tests {
    use super::VecDatabase;
    use crate::core::dispatcher::simple::Delete;

    crate::test_dispatcher!(VecDatabase);
}
