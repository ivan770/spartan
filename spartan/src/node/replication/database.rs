use super::storage::ReplicationStorage;
use crate::node::replication::event::Event;
use serde::{Deserialize, Serialize};
use spartan_lib::core::{
    dispatcher::{
        simple::{Delete, PositionBasedDelete},
        SimpleDispatcher, StatusAwareDispatcher,
    },
    message::Message,
    payload::Identifiable,
};

#[derive(Serialize, Deserialize)]
pub struct ReplicatedDatabase<DB> {
    inner: DB,
    storage: Option<ReplicationStorage>,
}

impl<DB> Default for ReplicatedDatabase<DB>
where
    DB: Default,
{
    fn default() -> Self {
        ReplicatedDatabase {
            inner: DB::default(),
            storage: None,
        }
    }
}

impl<DB> ReplicatedDatabase<DB> {
    fn call_storage<F>(&mut self, f: F)
    where
        F: FnOnce(&mut ReplicationStorage),
    {
        self.storage.as_mut().map(f);
    }

    fn push_event<F>(&mut self, event: F)
    where
        F: FnOnce() -> Event,
    {
        self.call_storage(|storage| storage.get_primary().push(event()));
    }

    fn gc(&mut self) {
        self.call_storage(|storage| storage.get_primary().gc());
    }
}

impl<DB> SimpleDispatcher<Message> for ReplicatedDatabase<DB>
where
    DB: SimpleDispatcher<Message>,
{
    fn push(&mut self, message: Message) {
        self.push_event(|| Event::Push(message.clone()));

        self.inner.push(message)
    }

    fn peek(&self) -> Option<&Message> {
        self.inner.peek()
    }

    fn gc(&mut self) {
        self.push_event(|| Event::Gc);

        ReplicatedDatabase::gc(self);
        self.inner.gc()
    }

    fn size(&self) -> usize {
        self.inner.size()
    }

    fn clear(&mut self) {
        self.push_event(|| Event::Clear);

        self.inner.clear()
    }
}

impl<DB> StatusAwareDispatcher<Message> for ReplicatedDatabase<DB>
where
    DB: StatusAwareDispatcher<Message>,
{
    fn pop(&mut self) -> Option<&Message> {
        self.push_event(|| Event::Pop);

        self.inner.pop()
    }

    fn requeue(&mut self, id: <Message as Identifiable>::Id) -> Option<()> {
        self.push_event(|| Event::Requeue(id));

        self.inner.requeue(id)
    }
}

impl<DB> Delete<Message> for ReplicatedDatabase<DB>
where
    DB: Delete<Message>,
{
    fn delete(&mut self, id: <Message as Identifiable>::Id) -> Option<Message> {
        self.push_event(|| Event::Delete(id));

        Delete::delete(&mut self.inner, id)
    }
}

impl<DB> PositionBasedDelete<Message> for ReplicatedDatabase<DB>
where
    DB: PositionBasedDelete<Message>,
{
    fn delete(&mut self, id: <Message as Identifiable>::Id) -> Option<Message> {
        self.push_event(|| Event::Delete(id));

        PositionBasedDelete::delete(&mut self.inner, id)
    }
}
