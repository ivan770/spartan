use crate::core::payload::Dispatchable;

/// Interface for interacting with message status
pub trait Status: Dispatchable {
    /// Change message status to available
    ///
    /// ```
    /// use spartan_lib::core::message::builder::MessageBuilder;
    /// use spartan_lib::core::payload::Status;
    ///
    /// let mut message = MessageBuilder::default().body(b"Hello, world").compose().unwrap();
    ///
    /// message.requeue();
    /// ```
    fn requeue(&mut self);

    /// Change message status to "in transit"
    /// Also, default message implementation increments counter of tries
    ///
    /// ```
    /// use spartan_lib::core::message::builder::MessageBuilder;
    /// use spartan_lib::core::payload::Status;
    ///
    /// let mut message = MessageBuilder::default().body(b"Hello, world").compose().unwrap();
    ///
    /// message.reserve();
    /// ```
    fn reserve(&mut self);

    /// Check if message can be requeued
    ///
    /// ```
    /// use spartan_lib::core::message::builder::MessageBuilder;
    /// use spartan_lib::core::payload::Status;
    ///
    /// let mut message = MessageBuilder::default().body(b"Hello, world").compose().unwrap();
    ///
    /// assert!(!message.requeueable());
    /// ```
    fn requeueable(&self) -> bool;

    /// Check if message can be reserved
    ///
    /// ```
    /// use spartan_lib::core::message::builder::MessageBuilder;
    /// use spartan_lib::core::payload::Status;
    ///
    /// let mut message = MessageBuilder::default().body(b"Hello, world").compose().unwrap();
    ///
    /// assert!(message.reservable());
    /// ```
    fn reservable(&self) -> bool;
}
