use crate::node::{event::Event, persistence::PersistenceError, Manager};
use actix_rt::time::delay_for;
use futures_util::{
    stream::{iter, StreamExt},
    TryStreamExt,
};
use spartan_lib::core::dispatcher::SimpleDispatcher;
use std::time::Duration;

#[cfg(feature = "replication")]
use crate::node::replication::primary::storage::PrimaryStorage;

/// Concurrently iterates over all databases in node, and executes GC on them.
async fn execute_gc(manager: &Manager<'_>) -> Result<(), PersistenceError> {
    iter(manager.node().iter())
        .map(Ok)
        .try_for_each_concurrent(None, |(name, queue)| async move {
            info!("Started GC cycle on database \"{}\"", name);

            queue.log_event(name, manager, Event::Gc).await?;

            queue.database().await.gc();

            #[cfg(feature = "replication")]
            if let Some(storage) = queue.replication_storage().await.as_mut() {
                storage.map_primary(PrimaryStorage::gc);
            }

            info!("GC cycle on \"{}\" completed successfully", name);

            Ok(())
        })
        .await
}

/// GC job spawner
///
/// Periodically iterates over all databases in node, and executes GC on them.
pub async fn spawn_gc(manager: &Manager<'_>) {
    debug!("Spawning GC handler.");

    let timer = Duration::from_secs(manager.config().gc_timer);

    loop {
        delay_for(timer).await;

        if let Err(e) = execute_gc(manager).await {
            error!("{}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::execute_gc;
    use crate::{node::Manager, utils::testing::CONFIG};
    use spartan_lib::core::{
        dispatcher::SimpleDispatcher, message::builder::MessageBuilder, payload::Status,
    };

    #[tokio::test]
    async fn test_gc() {
        let mut manager = Manager::new(&CONFIG);

        manager.node_mut().add("first");

        let mut message = MessageBuilder::default()
            .body("Hello, world")
            .max_tries(1)
            .compose()
            .unwrap();

        message.reserve();
        message.requeue();
        manager
            .queue("first")
            .unwrap()
            .database()
            .await
            .push(message);

        assert_eq!(manager.queue("first").unwrap().database().await.size(), 1);

        execute_gc(&manager).await.unwrap();

        assert_eq!(manager.queue("first").unwrap().database().await.size(), 0);
    }
}
