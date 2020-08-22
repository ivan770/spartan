use super::Node;
use crate::node::replication::storage::ReplicationStorage;
use tokio::sync::Mutex;

pub struct Queue<DB> {
    /// Proxied database
    pub database: Mutex<DB>,

    #[cfg(feature = "replication")]
    /// Replication storage
    /// None if replication is not enabled
    pub replication_storage: Mutex<Option<ReplicationStorage>>,
}

impl<DB> Default for Queue<DB>
where
    DB: Default,
{
    fn default() -> Self {
        Queue {
            database: Mutex::new(DB::default()),
            #[cfg(feature = "replication")]
            replication_storage: Mutex::new(None),
        }
    }
}

#[cfg(feature = "replication")]
impl<DB> Queue<DB> {
    pub fn new(database: DB, replication_storage: Option<ReplicationStorage>) -> Queue<DB> {
        Queue {
            database: Mutex::new(database),
            replication_storage: Mutex::new(replication_storage),
        }
    }

    pub async fn prepare_replication<F, R>(&self, filter: F, replace: R)
    where
        F: Fn(&ReplicationStorage) -> bool + Copy,
        R: Fn() -> ReplicationStorage,
    {
        let mut replication_storage = self.replication_storage.lock().await;

        if replication_storage
            .as_ref()
            .filter(|storage| filter(*storage))
            .is_none()
        {
            replication_storage.replace(replace());
        }
    }
}

#[cfg(feature = "replication")]
impl Node<'_> {
    pub async fn prepare_replication<F, R>(&self, filter: F, replace: R)
    where
        F: Fn(&ReplicationStorage) -> bool + Copy,
        R: Fn() -> ReplicationStorage + Copy,
    {
        //TODO: Concurrency
        for (_, queue) in self.iter() {
            queue.prepare_replication(filter, replace).await;
        }
    }
}
