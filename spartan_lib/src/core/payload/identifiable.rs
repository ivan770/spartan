/// Interface for working with identifiable messages
pub trait Identifiable {
    type Id: Copy + Eq;

    /// Get message ID
    ///
    /// ```
    /// use spartan_lib::core::message::builder::MessageBuilder;
    /// use spartan_lib::core::payload::Identifiable;
    ///
    /// let message = MessageBuilder::default().body(b"Hello, world").compose().unwrap();
    ///
    /// println!("Message ID is: {}", message.id());
    /// ```
    fn id(&self) -> Self::Id;
}
