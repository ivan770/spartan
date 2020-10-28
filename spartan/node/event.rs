use maybe_owned::MaybeOwned;
use serde::{Deserialize, Serialize};
use spartan_lib::core::{
    dispatcher::{simple::PositionBasedDelete, SimpleDispatcher, StatusAwareDispatcher},
    message::Message,
    payload::Identifiable,
};

/// Database event
/// Only events that mutate database are present here
#[derive(Serialize, Deserialize)]
#[cfg_attr(test, derive(Debug))]
pub enum Event<'a> {
    Push(MaybeOwned<'a, Message>),
    Pop,
    Requeue(<Message as Identifiable>::Id),
    Delete(<Message as Identifiable>::Id),
    Gc,
    Clear,
}

#[cfg(feature = "replication")]
impl<'a> Event<'a> {
    pub(super) fn into_owned(self) -> Event<'static> {
        match self {
            Event::Push(message) => Event::Push(MaybeOwned::Owned(message.into_owned())),
            // These variants are needed to appease compiler
            // since it doesn't know that all other variants are 'static
            Event::Pop => Event::Pop,
            Event::Requeue(id) => Event::Requeue(id),
            Event::Delete(id) => Event::Delete(id),
            Event::Gc => Event::Gc,
            Event::Clear => Event::Clear,
        }
    }
}

#[cfg(test)]
impl PartialEq for Event<'_> {
    fn eq(&self, other: &Self) -> bool {
        matches!((self, other), (Event::Pop, Event::Pop))
    }
}

pub trait EventLog<L>: Default {
    fn from_log(log: L) -> Self {
        let mut database = Self::default();
        database.apply_log(log);
        database
    }

    fn apply_log(&mut self, log: L);
}

impl<L, DB> EventLog<L> for DB
where
    L: IntoIterator<Item = Event<'static>>,
    DB: SimpleDispatcher<Message>
        + StatusAwareDispatcher<Message>
        + PositionBasedDelete<Message>
        + Default,
{
    fn apply_log(&mut self, log: L) {
        for event in log {
            match event {
                Event::Push(message) => match message {
                    MaybeOwned::Owned(message) => self.push(message),
                    _ => panic!("Applying push event with borrowed message is not allowed."),
                },
                Event::Pop => {
                    self.pop();
                }
                Event::Requeue(id) => {
                    self.requeue(id);
                }
                Event::Delete(id) => {
                    self.delete(id);
                }
                Event::Gc => {
                    self.gc();
                }
                Event::Clear => {
                    self.clear();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Event, EventLog};
    use crate::node::DB;
    use maybe_owned::MaybeOwned;
    use spartan_lib::core::{dispatcher::StatusAwareDispatcher, message::builder::MessageBuilder};

    #[tokio::test]
    async fn test_apply_events() {
        let queue = DB::default();

        let message = MessageBuilder::default().body("test").compose().unwrap();

        let events = vec![Event::Push(MaybeOwned::Owned(message.clone()))];

        queue.database().await.apply_log(events);

        assert_eq!(queue.database().await.pop().unwrap().id, message.id);
    }
}
