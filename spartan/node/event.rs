use crate::node::Queue;
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

#[cfg(test)]
impl PartialEq for Event<'_> {
    fn eq(&self, other: &Self) -> bool {
        matches!((self, other), (Event::Pop, Event::Pop))
    }
}

impl<DB> Queue<DB>
where
    DB: SimpleDispatcher<Message> + StatusAwareDispatcher<Message> + PositionBasedDelete<Message>,
{
    async fn apply_event(&self, event: Event<'_>, queue: &mut DB) {
        match event {
            Event::Push(message) => match message {
                MaybeOwned::Owned(message) => queue.push(message),
                _ => panic!("Applying push event with borrowed message is not allowed."),
            },
            Event::Pop => {
                queue.pop();
            }
            Event::Requeue(id) => {
                queue.requeue(id);
            }
            Event::Delete(id) => {
                queue.delete(id);
            }
            Event::Gc => {
                queue.gc();
            }
            Event::Clear => {
                queue.clear();
            }
        }
    }

    pub async fn apply_events<I>(&self, events: I)
    where
        I: IntoIterator<Item = MaybeOwned<'static, Event<'static>>>,
    {
        let mut queue = self.database().await;

        for event in events.into_iter() {
            match event {
                MaybeOwned::Owned(event) => self.apply_event(event, &mut *queue).await,
                MaybeOwned::Borrowed(_) => unreachable!(),
            };
        }
    }

    pub async fn log_event<F>(&self, f: F)
    where
        F: FnOnce() -> Event<'static>,
    {
        if let Some(storage) = self.replication_storage().await.as_mut() {
            storage.map_primary(|storage| storage.push(f()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Event;
    use crate::node::DB;
    use maybe_owned::MaybeOwned;
    use spartan_lib::core::{dispatcher::StatusAwareDispatcher, message::builder::MessageBuilder};

    #[tokio::test]
    async fn test_apply_events() {
        let queue = DB::default();

        let message = MessageBuilder::default().body("test").compose().unwrap();

        let events = vec![MaybeOwned::Owned(Event::Push(MaybeOwned::Owned(
            message.clone(),
        )))];

        queue.apply_events(events).await;

        assert_eq!(queue.database().await.pop().unwrap().id, message.id);
    }
}
