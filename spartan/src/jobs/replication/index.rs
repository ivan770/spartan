use super::{
    error::{ReplicationError, ReplicationResult},
    stream::Stream,
};
use crate::node::Manager;
use futures_util::{stream::iter, StreamExt, TryStreamExt};

pub(super) struct RecvIndex<'a> {
    stream: &'a mut Stream,
    indexes: Box<[(Box<str>, u64)]>,
}

impl<'a> RecvIndex<'a> {
    pub fn new(stream: &'a mut Stream, indexes: Box<[(Box<str>, u64)]>) -> Self {
        RecvIndex { stream, indexes }
    }

    pub async fn sync(&mut self, manager: &Manager<'_>) -> ReplicationResult<()> {
        for (name, start) in self.indexes.iter() {
            self.stream
                .send_range(
                    name,
                    manager
                        .queue(name)
                        .await
                        .map_err(|_| ReplicationError::QueueConfigMismatch)?
                        .get_storage()
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

pub(super) struct BatchAskIndex<'a> {
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

    pub async fn sync(&mut self, manager: &Manager<'_>) -> ReplicationResult<()> {
        iter(self.batch.iter_mut())
            .map(Ok)
            .try_for_each_concurrent(None, |host| async move { host.sync(manager).await })
            .await?;

        Ok(())
    }
}
