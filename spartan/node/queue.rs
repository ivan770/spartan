use tokio::sync::{Mutex, MutexGuard};

#[cfg(feature = "replication")]
use crate::node::replication::storage::ReplicationStorage;

use super::{event::Event, persistence::PersistenceError, Manager};

pub struct Queue<DB> {
    /// Inner database
    database: Mutex<DB>,

    #[cfg(feature = "replication")]
    /// Replication storage
    /// None if replication is not enabled
    replication_storage: Mutex<Option<ReplicationStorage>>,
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

impl<DB> Queue<DB> {
    #[cfg(feature = "replication")]
    pub fn new(database: DB, replication_storage: Option<ReplicationStorage>) -> Queue<DB> {
        Queue {
            database: Mutex::new(database),
            replication_storage: Mutex::new(replication_storage),
        }
    }

    #[cfg(not(feature = "replication"))]
    pub fn new(database: DB) -> Queue<DB> {
        Queue {
            database: Mutex::new(database),
        }
    }

    pub async fn database(&self) -> MutexGuard<'_, DB> {
        self.database.lock().await
    }

    #[cfg(feature = "replication")]
    pub async fn replication_storage(&self) -> MutexGuard<'_, Option<ReplicationStorage>> {
        self.replication_storage.lock().await
    }

    #[cfg(feature = "replication")]
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

    pub async fn log_event(
        &self,
        name: &str,
        manager: &Manager<'_>,
        event: Event<'_>,
    ) -> Result<(), PersistenceError> {
        manager.log(name, &event).await?;

        #[cfg(feature = "replication")]
        if let Some(storage) = self.replication_storage().await.as_mut() {
            storage.map_primary(|storage| storage.push(event.into_owned()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::node::replication::{
        primary::storage::PrimaryStorage, replica::storage::ReplicaStorage,
        storage::ReplicationStorage,
    };
    use crate::node::DB;

    #[tokio::test]
    async fn test_prepare_replication_empty() {
        let queue = DB::default();

        assert!(queue.replication_storage.lock().await.is_none());

        queue
            .prepare_replication(
                |_| false,
                || ReplicationStorage::Primary(PrimaryStorage::default()),
            )
            .await;

        assert!(matches!(
            queue.replication_storage.lock().await.as_ref().unwrap(),
            &ReplicationStorage::Primary(_)
        ));
    }

    #[tokio::test]
    async fn test_prepare_replication() {
        let queue = DB::default();

        queue
            .prepare_replication(
                |_| false,
                || ReplicationStorage::Primary(PrimaryStorage::default()),
            )
            .await;

        assert!(matches!(
            queue.replication_storage.lock().await.as_ref().unwrap(),
            &ReplicationStorage::Primary(_)
        ));

        queue
            .prepare_replication(
                |storage| matches!(storage, ReplicationStorage::Replica(_)),
                || ReplicationStorage::Replica(ReplicaStorage::default()),
            )
            .await;

        assert!(matches!(
            queue.replication_storage.lock().await.as_ref().unwrap(),
            &ReplicationStorage::Replica(_)
        ));
    }
}
