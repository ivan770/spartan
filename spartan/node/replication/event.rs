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
pub enum Event {
    Push(Message),
    Pop,
    Requeue(<Message as Identifiable>::Id),
    Delete(<Message as Identifiable>::Id),
    Gc,
    Clear,
}

#[cfg(test)]
impl PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        matches!((self, other), (Event::Pop, Event::Pop))
    }
}

impl<DB> Queue<DB>
where
    DB: SimpleDispatcher<Message> + StatusAwareDispatcher<Message> + PositionBasedDelete<Message>,
{
    async fn apply_event(&self, event: Event, queue: &mut DB) {
        match event {
            Event::Push(message) => queue.push(message),
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

    pub async fn apply_events(&self, events: Box<[(MaybeOwned<'_, u64>, MaybeOwned<'_, Event>)]>) {
        let index = events.last().map(|(index, _)| **index);

        {
            let mut queue = self.database.lock().await;

            // into_vec allows to use owned event
            for (_, event) in events.into_vec().into_iter() {
                match event {
                    MaybeOwned::Owned(event) => self.apply_event(event, &mut *queue).await,
                    MaybeOwned::Borrowed(_) => unreachable!(),
                };
            }
        }

        if let Some(index) = index {
            self.replication_storage
                .lock()
                .await
                .as_mut()
                .expect("No storage provided")
                .get_replica()
                .confirm(index);
        }
    }
}
