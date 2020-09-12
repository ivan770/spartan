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
            let mut queue = self.database().await;

            // into_vec allows to use owned event
            for (_, event) in events.into_vec().into_iter() {
                match event {
                    MaybeOwned::Owned(event) => self.apply_event(event, &mut *queue).await,
                    MaybeOwned::Borrowed(_) => unreachable!(),
                };
            }
        }

        if let Some(index) = index {
            self.replication_storage()
                .await
                .as_mut()
                .expect("No storage provided")
                .get_replica()
                .confirm(index);
        }
    }

    pub async fn log_event<F>(&self, f: F)
    where
        F: FnOnce() -> Event,
    {
        if let Some(storage) = self.replication_storage().await.as_mut() {
            storage.map_primary(|storage| storage.push(f()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Event;
    use crate::node::{
        replication::{
            primary::storage::PrimaryStorage, replica::storage::ReplicaStorage,
            storage::ReplicationStorage,
        },
        DB,
    };
    use maybe_owned::MaybeOwned;

    #[tokio::test]
    async fn test_apply_events() {
        let queue = DB::default();

        let events = vec![
            (MaybeOwned::Owned(1), MaybeOwned::Owned(Event::Gc)),
            (MaybeOwned::Owned(2), MaybeOwned::Owned(Event::Clear)),
        ]
        .into_boxed_slice();

        queue
            .prepare_replication(
                |_| false,
                || ReplicationStorage::Replica(ReplicaStorage::default()),
            )
            .await;

        assert_eq!(
            queue
                .replication_storage()
                .await
                .as_mut()
                .unwrap()
                .get_replica()
                .get_index(),
            1
        );

        queue.apply_events(events).await;

        assert_eq!(
            queue
                .replication_storage()
                .await
                .as_mut()
                .unwrap()
                .get_replica()
                .get_index(),
            3
        );
    }

    #[tokio::test]
    #[should_panic(expected = "Replication storage is in primary mode.")]
    async fn test_apply_events_with_invalid_storage() {
        let queue = DB::default();

        let events = vec![
            (MaybeOwned::Owned(1), MaybeOwned::Owned(Event::Gc)),
            (MaybeOwned::Owned(2), MaybeOwned::Owned(Event::Clear)),
        ]
        .into_boxed_slice();

        queue
            .prepare_replication(
                |_| false,
                || ReplicationStorage::Primary(PrimaryStorage::default()),
            )
            .await;

        queue.apply_events(events).await;
    }
}
