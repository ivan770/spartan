use crate::node::DB;
use maybe_owned::MaybeOwned;
use serde::{Deserialize, Serialize};
use spartan_lib::core::{
    dispatcher::{simple::PositionBasedDelete, SimpleDispatcher, StatusAwareDispatcher},
    message::Message,
    payload::Identifiable,
};

#[derive(Serialize, Deserialize)]
pub enum Event {
    Push(Message),
    Pop,
    Requeue(<Message as Identifiable>::Id),
    Delete(<Message as Identifiable>::Id),
    Gc,
    Clear,
}

impl Event {
    fn apply_event(self, queue: &mut DB) {
        let queue = &mut **queue;

        match self {
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

    pub fn apply_events(
        queue: &mut DB,
        events: Box<[(MaybeOwned<'_, u64>, MaybeOwned<'_, Event>)]>,
    ) -> Option<()> {
        let index = *events.last()?.0;

        // into_vec allows to use owned event
        events
            .into_vec()
            .into_iter()
            .for_each(|(_, event)| match event {
                MaybeOwned::Owned(event) => event.apply_event(queue),
                MaybeOwned::Borrowed(_) => unreachable!(),
            });

        queue
            .get_storage()
            .as_mut()
            .expect("No storage provided")
            .get_replica()
            .confirm(index);

        Some(())
    }
}
