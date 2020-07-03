use super::entry::{Action, LogEntry};
use actix_raft::{
    messages::{Entry, EntryNormal, EntryPayload},
    AppData,
};
use futures_util::lock::MutexGuard;
use spartan_lib::core::{
    dispatcher::{
        simple::{Delete, PositionBasedDelete},
        SimpleDispatcher, StatusAwareDispatcher,
    },
    message::Message,
    payload::Identifiable,
};
use std::collections::BTreeMap;

pub struct QueueGuard<'a, E, DB>
where
    E: AppData,
{
    name: &'a str,
    queue: MutexGuard<'a, DB>,
    log: Option<MutexGuard<'a, BTreeMap<u64, Entry<E>>>>,
}

impl<'a, E, DB> QueueGuard<'a, E, DB>
where
    E: AppData,
{
    pub fn new(
        name: &'a str,
        queue: MutexGuard<'a, DB>,
        log: Option<MutexGuard<'a, BTreeMap<u64, Entry<E>>>>,
    ) -> Self {
        QueueGuard { name, queue, log }
    }
}

impl<'a, DB> QueueGuard<'a, LogEntry, DB> {
    fn add_entry<F>(&mut self, entry: F)
    where
        F: FnOnce() -> Action,
    {
        if let Some(log) = self.log.as_mut() {
            //TODO: Remove test values from here
            log.insert(
                1,
                Entry {
                    term: 1,
                    index: 1,
                    payload: EntryPayload::Normal(EntryNormal {
                        data: LogEntry {
                            queue: self.name.to_string(),
                            action: entry(),
                        },
                    }),
                },
            );
        }
    }
}

impl<'a, DB> SimpleDispatcher<Message> for QueueGuard<'a, LogEntry, DB>
where
    DB: SimpleDispatcher<Message>,
{
    fn push(&mut self, message: Message) {
        self.add_entry(|| Action::Push(message.clone()));

        self.queue.push(message)
    }

    fn peek(&self) -> Option<&Message> {
        self.queue.peek()
    }

    fn gc(&mut self) {
        self.add_entry(|| Action::Gc);

        self.queue.gc()
    }

    fn size(&self) -> usize {
        self.queue.size()
    }

    fn clear(&mut self) {
        self.add_entry(|| Action::Clear);

        self.queue.clear()
    }
}

impl<'a, DB> StatusAwareDispatcher<Message> for QueueGuard<'a, LogEntry, DB>
where
    DB: StatusAwareDispatcher<Message>,
{
    fn pop(&mut self) -> Option<&Message> {
        self.add_entry(|| Action::Pop);

        self.queue.pop()
    }

    fn requeue(&mut self, id: <Message as Identifiable>::Id) -> Option<()> {
        self.add_entry(|| Action::Requeue(id));

        self.queue.requeue(id)
    }
}

impl<'a, DB> Delete<Message> for QueueGuard<'a, LogEntry, DB>
where
    DB: Delete<Message>,
{
    fn delete(&mut self, id: <Message as Identifiable>::Id) -> Option<Message> {
        self.add_entry(|| Action::Delete(id));

        Delete::delete(&mut *self.queue, id)
    }
}

impl<'a, DB> PositionBasedDelete<Message> for QueueGuard<'a, LogEntry, DB>
where
    DB: PositionBasedDelete<Message>,
{
    fn delete(&mut self, id: <Message as Identifiable>::Id) -> Option<Message> {
        self.add_entry(|| Action::Delete(id));

        PositionBasedDelete::delete(&mut *self.queue, id)
    }
}
