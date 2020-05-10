/// Interface for working with sortable messages
pub trait Sortable {
    type Sort: Ord;

    /// Get message sort key. It's used for message prioritization in queue.
    ///
    /// ```
    /// use spartan_lib::core::message::builder::MessageBuilder;
    /// use spartan_lib::core::payload::Sortable;
    ///
    /// let message = MessageBuilder::default().body(b"Hello, world").compose().unwrap();
    ///
    /// dbg!(message.sort());
    /// ```
    fn sort(&self) -> Self::Sort;
}
