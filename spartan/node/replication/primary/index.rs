use super::{
    error::{PrimaryError, PrimaryResult},
    stream::Stream,
};
use crate::node::Manager;
use futures_util::{stream::iter, StreamExt, TryStreamExt};
use itertools::Itertools;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

pub struct RecvIndex<'a> {
    stream: &'a mut Stream,
    indexes: Box<[(Box<str>, u64)]>,
}

impl<'a> RecvIndex<'a> {
    pub fn new(stream: &'a mut Stream, indexes: Box<[(Box<str>, u64)]>) -> Self {
        RecvIndex { stream, indexes }
    }

    pub async fn sync(&mut self, manager: &Manager<'_>) -> PrimaryResult<()> {
        for (name, start) in self.indexes.iter() {
            self.stream
                .send_range(
                    name,
                    manager
                        .queue(name)
                        .map_err(|_| PrimaryError::QueueConfigMismatch)?
                        .replication_storage
                        .lock()
                        .await
                        .as_mut()
                        .expect("Replication storage is uninitialized")
                        .get_primary()
                        .slice(*start),
                )
                .await?;
        }

        Ok(())
    }
}

pub struct BatchAskIndex<'a> {
    batch: Vec<RecvIndex<'a>>,
}

impl<'a> BatchAskIndex<'a> {
    pub fn with_capacity(capacity: usize) -> Self {
        BatchAskIndex {
            batch: Vec::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, index: RecvIndex<'a>) {
        self.batch.push(index);
    }

    pub async fn sync(mut self, manager: &Manager<'_>) -> PrimaryResult<Sync<'a>> {
        iter(self.batch.iter_mut())
            .map(Ok)
            .try_for_each_concurrent(None, |host| async move { host.sync(manager).await })
            .await?;

        Ok(Sync::new(self))
    }
}

pub struct Sync<'a> {
    batch_ask_index: BatchAskIndex<'a>,
}

impl<'a> Sync<'a> {
    fn new(batch_ask_index: BatchAskIndex<'a>) -> Self {
        Sync { batch_ask_index }
    }

    /// Set GC threshold of each queue to minimal index of all replica's
    ///
    /// Example:
    ///
    /// ```no_run
    /// First replica: [("TestQueue", 2), ("NextQueue", 3)]
    /// Second replica: [("TestQueue", 1), ("AnotherQueue", 4)]
    ///
    /// Result: [("AnotherQueue", 4), ("NextQueue", 3), ("TestQueue", 1)]
    /// ```
    pub async fn set_gc(&self, manager: &Manager<'_>) {
        let iter = self
            .batch_ask_index
            .batch
            .iter()
            .map(|index| index.indexes.iter())
            .flatten()
            .sorted_by(|a, b| Ord::cmp(a, b))
            .unique_by(|(name, _)| {
                let mut hasher = DefaultHasher::new();
                name.hash(&mut hasher);
                hasher.finish()
            });

        for (queue, index) in iter {
            manager
                .queue(&queue)
                .as_mut()
                .expect("set_gc called without sync before")
                .replication_storage
                .lock()
                .await
                .as_mut()
                .unwrap()
                .get_primary()
                .set_gc_threshold(*index);
        }
    }
}
