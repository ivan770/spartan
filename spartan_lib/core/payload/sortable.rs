/// Interface for working with sortable messages
pub trait Sortable {
    type Sort: Ord;

    /// Get message sort key. It's used for message prioritization in queue.
    ///
    /// While [`Sortable`] itself doesn't have any contract, it's usage in [`TreeDatabase`](crate::core::db::tree::TreeDatabase)
    /// allows to ignore all messages except the first one.
    ///
    /// ```
    /// use spartan_lib::core::message::builder::MessageBuilder;
    /// use spartan_lib::core::payload::Sortable;
    ///
    /// let message = MessageBuilder::default().body("Hello, world").compose().unwrap();
    ///
    /// dbg!(message.sort());
    /// ```
    fn sort(&self) -> Self::Sort;
}
