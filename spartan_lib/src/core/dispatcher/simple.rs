use crate::core::{
    db::Database,
    payload::{Dispatchable, Identifiable},
};

/// Interface for working with database as message queue
pub trait SimpleDispatcher<M>
where
    M: Dispatchable,
{
    /// Push message to queue
    ///
    /// ```
    /// use spartan_lib::core::dispatcher::SimpleDispatcher;
    /// use spartan_lib::core::db::vec::VecDatabase;
    /// use spartan_lib::core::message::builder::MessageBuilder;
    ///
    /// let mut db = VecDatabase::default();
    /// let message = MessageBuilder::default().body("Hello, world").compose().unwrap();
    ///
    /// db.push(message);
    /// ```
    fn push(&mut self, message: M);

    /// Get message from queue
    ///
    /// ```
    /// use spartan_lib::core::dispatcher::SimpleDispatcher;
    /// use spartan_lib::core::db::vec::VecDatabase;
    /// use spartan_lib::core::message::builder::MessageBuilder;
    /// use spartan_lib::core::payload::Identifiable;
    ///
    /// let mut db = VecDatabase::default();
    /// let message = MessageBuilder::default().body("Hello, world").compose().unwrap();
    ///
    /// db.push(message.clone());
    ///
    /// assert_eq!(db.peak().unwrap().id(), message.id());
    /// ```
    fn peak(&self) -> Option<&M>;

    /// Start GC cycle on queue
    ///
    /// ```
    /// use spartan_lib::core::dispatcher::SimpleDispatcher;
    /// use spartan_lib::core::db::vec::VecDatabase;
    /// use spartan_lib::core::message::builder::MessageBuilder;
    /// use spartan_lib::core::payload::{Identifiable, Status};
    /// use std::thread::sleep;
    /// use std::time::Duration;
    ///
    /// let mut db = VecDatabase::default();
    ///
    /// // We need two test messages here: one without timeout, and one with 1 second timeout
    /// // GC condition varies between messages, but here we'll use timeout as an example
    /// let message = MessageBuilder::default().body("Hello, world").compose().unwrap();
    ///
    /// // Setting timeout to 0 and waiting 1-2 seconds turns message into garbage
    /// let mut garbage_message = MessageBuilder::default().body("I will be deleted").timeout(0).compose().unwrap();
    ///
    /// // We are going to reserve the message before adding it do database
    /// garbage_message.reserve();
    ///
    /// sleep(Duration::from_secs(2));
    ///
    /// db.push(message.clone());
    /// db.push(garbage_message);
    ///
    /// assert_eq!(db.size(), 2);
    ///
    /// db.gc();
    ///
    /// assert_eq!(db.size(), 1);
    /// assert_eq!(db.peak().unwrap().id(), message.id());
    /// ```
    fn gc(&mut self);

    /// Get queue size
    ///
    /// ```
    /// use spartan_lib::core::dispatcher::SimpleDispatcher;
    /// use spartan_lib::core::db::vec::VecDatabase;
    /// use spartan_lib::core::message::builder::MessageBuilder;
    ///
    /// let mut db = VecDatabase::default();
    /// let message = MessageBuilder::default().body("Hello, world").compose().unwrap();
    ///
    /// assert_eq!(db.size(), 0);
    ///
    /// db.push(message.clone());
    ///
    /// assert_eq!(db.size(), 1);
    /// ```
    fn size(&self) -> usize;

    /// Clear all queue messages
    ///
    /// ```
    /// use spartan_lib::core::dispatcher::SimpleDispatcher;
    /// use spartan_lib::core::db::vec::VecDatabase;
    /// use spartan_lib::core::message::builder::MessageBuilder;
    ///
    /// let mut db = VecDatabase::default();
    /// let message = MessageBuilder::default().body("Hello, world").compose().unwrap();
    ///
    /// assert_eq!(db.size(), 0);
    ///
    /// db.push(message.clone());
    ///
    /// assert_eq!(db.size(), 1);
    ///
    /// db.clear();
    ///
    /// assert_eq!(db.size(), 0);
    /// ```
    fn clear(&mut self);
}

/// Interface for deleting messages from queue, where database position key is not the same, as message ID
pub trait Delete<M>: SimpleDispatcher<M>
where
    M: Dispatchable,
{
    /// Delete message from queue
    ///
    /// ```
    /// use spartan_lib::core::dispatcher::{SimpleDispatcher, simple::Delete};
    /// use spartan_lib::core::db::vec::VecDatabase;
    /// use spartan_lib::core::message::builder::MessageBuilder;
    /// use spartan_lib::core::payload::Identifiable;
    ///
    /// let mut db = VecDatabase::default();
    /// let message = MessageBuilder::default().body("Hello, world").compose().unwrap();
    ///
    /// db.push(message.clone());
    ///
    /// assert_eq!(db.size(), 1);
    ///
    /// db.delete(message.id());
    ///
    /// assert_eq!(db.size(), 0);
    /// ```
    fn delete(&mut self, id: <M as Identifiable>::Id) -> Option<()>;
}

/// Interface for deleting messages from queue, where database position key is the same, as message ID
/// It is preferable to use PositionBasedDelete instead of Delete
pub trait PositionBasedDelete<M>: SimpleDispatcher<M>
where
    M: Dispatchable,
{
    /// Delete message from queue
    ///
    /// ```
    /// use spartan_lib::core::dispatcher::{SimpleDispatcher, simple::PositionBasedDelete};
    /// use spartan_lib::core::db::tree::TreeDatabase;
    /// use spartan_lib::core::message::builder::MessageBuilder;
    /// use spartan_lib::core::payload::Identifiable;
    ///
    /// let mut db = TreeDatabase::default();
    /// let message = MessageBuilder::default().body("Hello, world").compose().unwrap();
    ///
    /// db.push(message.clone());
    ///
    /// assert_eq!(db.size(), 1);
    ///
    /// db.delete(message.id());
    ///
    /// assert_eq!(db.size(), 0);
    /// ```
    fn delete(&mut self, id: <M as Identifiable>::Id) -> Option<()>;
}

impl<T, M> SimpleDispatcher<M> for T
where
    T: Database<M>,
    M: Dispatchable,
{
    fn push(&mut self, message: M) {
        self.push_raw(message);
    }

    fn peak(&self) -> Option<&M> {
        self.get(self.position(|msg| msg.obtainable())?)
    }

    fn gc(&mut self) {
        self.retain(|msg| !msg.gc());
    }

    fn size(&self) -> usize {
        self.len()
    }

    fn clear(&mut self) {
        self.clear()
    }
}

impl<T, M> Delete<M> for T
where
    T: Database<M>,
    M: Dispatchable,
{
    fn delete(&mut self, id: <M as Identifiable>::Id) -> Option<()> {
        self.delete_pos(self.position(|msg| msg.id() == id)?)
    }
}

impl<T, M> PositionBasedDelete<M> for T
where
    T: Database<M, PositionKey = <M as Identifiable>::Id>,
    M: Dispatchable,
{
    fn delete(&mut self, id: <M as Identifiable>::Id) -> Option<()> {
        self.delete_pos(id)
    }
}
