use super::{
    error::{PrimaryError, PrimaryResult},
    stream::Stream,
};
use crate::node::Manager;
use futures_util::{stream::iter, StreamExt, TryStreamExt};
use itertools::Itertools;
use std::{
    borrow::Cow,
    hash::{Hash, Hasher},
};
use tokio::io::{AsyncRead, AsyncWrite};

pub struct RecvIndex<'s, T> {
    stream: &'s mut Stream<T>,
    indexes: Box<[(Cow<'static, str>, u64)]>,
}

impl<'s, T> RecvIndex<'s, T>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    pub fn new(stream: &'s mut Stream<T>, indexes: Box<[(Cow<'static, str>, u64)]>) -> Self {
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
                        .replication_storage()
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

pub struct BatchAskIndex<'s, T> {
    batch: Vec<RecvIndex<'s, T>>,
}

impl<'s, T> BatchAskIndex<'s, T>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    pub fn with_capacity(capacity: usize) -> Self {
        BatchAskIndex {
            batch: Vec::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, index: RecvIndex<'s, T>) {
        self.batch.push(index);
    }

    pub async fn sync(mut self, manager: &Manager<'_>) -> PrimaryResult<Sync<'s, T>> {
        debug!("Starting event slice sync.");

        iter(self.batch.iter_mut())
            .map(Ok)
            .try_for_each_concurrent(None, |host| async move { host.sync(manager).await })
            .await?;

        Ok(Sync::new(self))
    }
}

pub struct Sync<'s, T> {
    batch_ask_index: BatchAskIndex<'s, T>,
}

impl<'s, T> Sync<'s, T> {
    fn new(batch_ask_index: BatchAskIndex<'s, T>) -> Self {
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
    pub async fn set_gc<H>(&self, manager: &Manager<'_>)
    where
        H: Hasher + Default,
    {
        debug!("Setting GC threshold.");

        let iter = self
            .batch_ask_index
            .batch
            .iter()
            .flat_map(|index| index.indexes.iter())
            .sorted_by(Ord::cmp)
            .unique_by(|(name, _)| {
                let mut hasher = H::default();
                name.hash(&mut hasher);
                hasher.finish()
            });

        for (queue, index) in iter {
            debug!("Updating {} GC threshold to {}", queue, index);

            manager
                .queue(&queue)
                .as_mut()
                .expect("set_gc called without sync before")
                .replication_storage()
                .await
                .as_mut()
                .unwrap()
                .get_primary()
                .set_gc_threshold(*index);
        }
    }
}
