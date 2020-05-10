use crate::core::{
    db::StatusAwareDatabase,
    dispatcher::simple::SimpleDispatcher,
    payload::{Identifiable, Status},
};

/// Interface for working with databases that support statuses
pub trait StatusAwareDispatcher<M>: SimpleDispatcher<M>
where
    M: Status,
{
    /// Pop message from queue
    /// Behaves like "peak", but with "obtainable" message check, message and database reservation
    ///
    /// ```
    /// use spartan_lib::core::dispatcher::{SimpleDispatcher, StatusAwareDispatcher};
    /// use spartan_lib::core::db::tree::TreeDatabase;
    /// use spartan_lib::core::message::builder::MessageBuilder;
    /// use spartan_lib::core::payload::Identifiable;
    ///
    /// let mut db = TreeDatabase::default();
    /// let message = MessageBuilder::default().body(b"Hello, world").compose().unwrap();
    ///
    /// db.push(message.clone());
    ///
    /// assert_eq!(db.pop().unwrap().id(), message.id());
    /// ```
    fn pop(&mut self) -> Option<&M>;

    /// Requeue message in queue
    /// Returns None, if message was not found, or message cannot be requeued
    ///
    /// ```
    /// use spartan_lib::core::dispatcher::{SimpleDispatcher, StatusAwareDispatcher};
    /// use spartan_lib::core::db::tree::TreeDatabase;
    /// use spartan_lib::core::message::builder::MessageBuilder;
    /// use spartan_lib::core::payload::Identifiable;
    ///
    /// let mut db = TreeDatabase::default();
    /// let message = MessageBuilder::default().body(b"Hello, world").compose().unwrap();
    ///
    /// db.push(message);
    ///
    /// let id = db.pop().unwrap().id();
    /// db.requeue(id).unwrap();
    /// ```
    fn requeue(&mut self, id: <M as Identifiable>::Id) -> Option<()>;
}

impl<T, M> StatusAwareDispatcher<M> for T
where
    T: SimpleDispatcher<M> + StatusAwareDatabase<M, RequeueKey = <M as Identifiable>::Id>,
    M: Status,
{
    fn pop(&mut self) -> Option<&M> {
        let position = self.position(|msg| msg.reservable() && msg.obtainable())?;
        let message = self.reserve(position).unwrap();
        message.reserve();
        Some(message)
    }

    fn requeue(&mut self, key: <M as Identifiable>::Id) -> Option<()> {
        let message = self.requeue(key, |msg| msg.requeueable() && msg.obtainable())?;
        message.requeue();
        Some(())
    }
}
