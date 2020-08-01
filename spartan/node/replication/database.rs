use super::storage::ReplicationStorage;
use crate::node::{Node, replication::event::Event};
use serde::{Deserialize, Serialize};
use spartan_lib::core::{
    dispatcher::{
        simple::{Delete, PositionBasedDelete},
        SimpleDispatcher, StatusAwareDispatcher,
    },
    message::Message,
    payload::Identifiable,
};
use std::ops::{Deref, DerefMut};

#[derive(Serialize, Deserialize)]
pub struct ReplicatedDatabase<DB> {
    /// Proxied database
    inner: DB,

    /// Replication storage
    /// None if replication is not enabled
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

impl<DB> Deref for ReplicatedDatabase<DB> {
    type Target = DB;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<DB> DerefMut for ReplicatedDatabase<DB> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
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
        self.call_storage(|storage| storage.map_primary(|storage| storage.push(event())));
    }

    fn gc(&mut self) {
        self.call_storage(|storage| storage.map_primary(|storage| storage.gc()));
    }

    pub fn get_storage(&mut self) -> &mut Option<ReplicationStorage> {
        &mut self.storage
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

impl Node<'_> {
    pub async fn prepare_replication<F, R>(&self, filter: F, replace: R)
    where
        F: Fn(&ReplicationStorage) -> bool + Copy,
        R: Fn() -> ReplicationStorage,
    {
        for (_, db) in self.iter() {
            let mut db = db.lock().await;

            let storage = db.get_storage().as_ref().filter(|storage| filter(*storage));

            if storage.is_none() {
                db.get_storage().replace(replace());
            }
        }
    }
}
