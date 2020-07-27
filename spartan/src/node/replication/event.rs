use crate::node::DB;
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
        match (self, other) {
            (Event::Pop, Event::Pop) => true,
            _ => false
        }
    }
}

pub trait ApplyEvent {
    /// Apply single event to database
    fn apply_event(&mut self, event: Event);

    /// Apply slice of events to database
    fn apply_events(&mut self, events: Box<[(MaybeOwned<'_, u64>, MaybeOwned<'_, Event>)]>);
}

impl ApplyEvent for DB {
    fn apply_event(&mut self, event: Event) {
        let queue = &mut **self;

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

    fn apply_events(&mut self, events: Box<[(MaybeOwned<'_, u64>, MaybeOwned<'_, Event>)]>) {
        let index = events.last().map(|(index, _)| **index);

        // into_vec allows to use owned event
        events
            .into_vec()
            .into_iter()
            .for_each(|(_, event)| match event {
                MaybeOwned::Owned(event) => self.apply_event(event),
                MaybeOwned::Borrowed(_) => unreachable!(),
            });

        if let Some(index) = index {
            self.get_storage()
                .as_mut()
                .expect("No storage provided")
                .get_replica()
                .confirm(index);
        }
    }
}
