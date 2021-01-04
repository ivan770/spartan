use crate::core::payload::Identifiable;

/// Interface for working with dispatchable messages
pub trait Dispatchable: Identifiable {
    /// Body type of dispatchable message
    type Body: ?Sized;

    /// Check if current message is obtainable
    ///
    /// ```
    /// use spartan_lib::core::message::builder::MessageBuilder;
    /// use spartan_lib::core::payload::Dispatchable;
    /// use spartan_lib::chrono::{Utc, Duration};
    ///
    /// let message = MessageBuilder::default().body("Hello, world").compose().unwrap();
    /// let delayed_message = MessageBuilder::default()
    ///     .body("Hello, world")
    ///     .delay(600)
    ///     .compose()
    ///     .unwrap();
    ///
    /// assert!(message.obtainable());
    /// assert!(!delayed_message.obtainable());
    /// ```
    fn obtainable(&self) -> bool;

    /// Get message body
    ///
    /// ```
    /// use spartan_lib::core::message::builder::MessageBuilder;
    /// use spartan_lib::core::payload::Dispatchable;
    ///
    /// let message = MessageBuilder::default().body("Hello, world").compose().unwrap();
    ///
    /// assert_eq!(message.body(), "Hello, world");
    /// ```
    fn body(&self) -> &Self::Body;

    /// Check if current message is garbage
    ///
    /// ```
    /// use spartan_lib::core::message::builder::MessageBuilder;
    /// use spartan_lib::core::payload::{Dispatchable, Status};
    /// use std::thread::sleep;
    /// use std::time::Duration;
    ///
    /// let mut message = MessageBuilder::default()
    ///     .body("Hello, world")
    ///     .timeout(0)
    ///     .compose()
    ///     .unwrap();
    ///
    /// message.reserve();
    ///
    /// sleep(Duration::from_secs(2));
    ///
    /// assert!(message.gc());
    /// ```
    fn gc(&self) -> bool;
}
