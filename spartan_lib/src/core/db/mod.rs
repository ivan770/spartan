pub mod tree;
pub mod vec;

/// Interface for working with databases
pub trait Database<M>: Default {
    type PositionKey: Copy;

    /// Push raw message to database
    ///
    /// ```
    /// use spartan_lib::core::db::Database;
    /// use spartan_lib::core::db::vec::VecDatabase;
    /// use spartan_lib::core::message::builder::MessageBuilder;
    ///
    /// let mut db = VecDatabase::default();
    /// let message = MessageBuilder::default().body(b"Hello, world").compose().unwrap();
    ///
    /// db.push_raw(message);
    /// ```
    fn push_raw(&mut self, message: M);

    /// Get database position key of the first message, that matches predicate
    ///
    /// ```
    /// use spartan_lib::core::db::Database;
    /// use spartan_lib::core::db::vec::VecDatabase;
    /// use spartan_lib::core::message::builder::MessageBuilder;
    /// use spartan_lib::core::payload::Dispatchable;
    ///
    /// let mut db = VecDatabase::default();
    /// let message = MessageBuilder::default().body(b"Hello, world").compose().unwrap();
    ///
    /// db.push_raw(message);
    ///
    /// assert_eq!(db.position(|msg| msg.obtainable()).unwrap(), 0);
    /// ```
    fn position<F>(&self, predicate: F) -> Option<Self::PositionKey>
    where
        F: Fn(&M) -> bool;

    /// Get shared message reference by database position key
    ///
    /// ```
    /// use spartan_lib::core::db::Database;
    /// use spartan_lib::core::db::vec::VecDatabase;
    /// use spartan_lib::core::message::builder::MessageBuilder;
    /// use spartan_lib::core::payload::Dispatchable;
    ///
    /// let mut db = VecDatabase::default();
    /// let message = MessageBuilder::default().body(b"Hello, world").compose().unwrap();
    ///
    /// db.push_raw(message);
    ///
    /// let position = db.position(|msg| msg.obtainable()).unwrap();
    ///
    /// assert!(db.get(position).unwrap().obtainable());
    /// ```
    fn get(&self, position: Self::PositionKey) -> Option<&M>;

    /// Get mutable message reference by database position key
    ///
    /// ```
    /// use spartan_lib::core::db::Database;
    /// use spartan_lib::core::db::vec::VecDatabase;
    /// use spartan_lib::core::message::builder::MessageBuilder;
    /// use spartan_lib::core::payload::{Dispatchable, Status};
    ///
    /// let mut db = VecDatabase::default();
    /// let message = MessageBuilder::default().body(b"Hello, world").compose().unwrap();
    ///
    /// db.push_raw(message);
    ///
    /// let position = db.position(|msg| msg.obtainable()).unwrap();
    ///
    /// db.get_mut(position).unwrap().reserve();
    /// ```
    fn get_mut(&mut self, position: Self::PositionKey) -> Option<&mut M>;

    /// Delete message by database position key
    ///
    /// ```
    /// use spartan_lib::core::db::Database;
    /// use spartan_lib::core::db::vec::VecDatabase;
    /// use spartan_lib::core::message::builder::MessageBuilder;
    /// use spartan_lib::core::payload::Dispatchable;
    ///
    /// let mut db = VecDatabase::default();
    /// let message = MessageBuilder::default().body(b"Hello, world").compose().unwrap();
    ///
    /// db.push_raw(message);
    ///
    /// let position = db.position(|msg| msg.obtainable()).unwrap();
    ///
    /// db.delete_pos(position).unwrap();
    /// ```
    fn delete_pos(&mut self, position: Self::PositionKey) -> Option<()>;

    /// Retain only messages, that match predicate
    ///
    /// ```
    /// use spartan_lib::core::db::Database;
    /// use spartan_lib::core::db::vec::VecDatabase;
    ///
    /// let mut db = VecDatabase::default();
    ///
    /// db.push_raw(1);
    /// db.push_raw(2);
    /// db.push_raw(3);
    ///
    /// db.retain(|msg| *msg == 1 || *msg == 2);
    ///
    /// assert_eq!(db.position(|msg| *msg == 2).unwrap(), 1);
    /// ```
    fn retain<F>(&mut self, predicate: F)
    where
        F: Fn(&M) -> bool;

    /// Get database size
    ///
    /// ```
    /// use spartan_lib::core::db::Database;
    /// use spartan_lib::core::db::vec::VecDatabase;
    ///
    /// let mut db = VecDatabase::default();
    ///
    /// db.push_raw(1);
    /// db.push_raw(2);
    /// db.push_raw(3);
    ///
    /// assert_eq!(db.len(), 3);
    /// ```
    fn len(&self) -> usize;

    /// Check if database is empty
    ///
    /// ```
    /// use spartan_lib::core::db::Database;
    /// use spartan_lib::core::db::vec::VecDatabase;
    ///
    /// let mut db = VecDatabase::default();
    ///
    /// db.push_raw(1);
    ///
    /// assert!(!db.is_empty());
    /// ```
    fn is_empty(&self) -> bool;

    /// Remove all database messages
    ///
    /// ```
    /// use spartan_lib::core::db::Database;
    /// use spartan_lib::core::db::vec::VecDatabase;
    ///
    /// let mut db = VecDatabase::default();
    ///
    /// db.push_raw(1);
    ///
    /// db.clear();
    ///
    /// assert!(db.is_empty());
    /// ```
    fn clear(&mut self);
}

/// Interface for working with databases, that support status interaction
pub trait StatusAwareDatabase<M>: Database<M> {
    type RequeueKey: Copy;

    /// Reserve message in database
    ///
    /// Removes message from tree in TreeDatabase, does nothing in VecDatabase
    ///
    /// ```
    /// use spartan_lib::core::db::{Database, StatusAwareDatabase};
    /// use spartan_lib::core::db::tree::TreeDatabase;
    /// use spartan_lib::core::message::builder::MessageBuilder;
    /// use spartan_lib::core::payload::{Dispatchable, Status};
    ///
    /// let mut db = TreeDatabase::default();
    /// let message = MessageBuilder::default().body(b"Hello, world").compose().unwrap();
    ///
    /// db.push_raw(message);
    ///
    /// let position = db.position(|msg| msg.reservable()).unwrap();
    ///
    /// // reserve returns mutable reference to message, so we can call reserve on message too.
    /// let message = db.reserve(position).unwrap();
    /// message.reserve();
    /// ```
    fn reserve(&mut self, position: Self::PositionKey) -> Option<&mut M>;

    /// Requeue message back to database
    ///
    /// Returns message back to tree in TreeDatabase, does nothing in VecDatabase
    ///
    /// ```
    /// use spartan_lib::core::db::{Database, StatusAwareDatabase};
    /// use spartan_lib::core::db::tree::TreeDatabase;
    /// use spartan_lib::core::message::builder::MessageBuilder;
    /// use spartan_lib::core::payload::{Dispatchable, Status, Identifiable};
    ///
    /// let mut db = TreeDatabase::default();
    /// let message = MessageBuilder::default().body(b"Hello, world").compose().unwrap();
    ///
    /// db.push_raw(message);
    ///
    /// let position = db.position(|msg| msg.reservable()).unwrap();
    ///
    /// let message = db.reserve(position).unwrap();
    /// message.reserve();
    ///
    /// let id = message.id;
    ///
    /// // requeue tries to find a message with provided id, and checks it for predicate
    /// let message = db.requeue(id, |msg| msg.requeueable()).unwrap();
    /// message.requeue();
    /// ```
    fn requeue<F>(&mut self, position: Self::RequeueKey, predicate: F) -> Option<&mut M>
    where
        F: Fn(&M) -> bool;
}
