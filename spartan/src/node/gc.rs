use super::Manager;
use actix_rt::time::delay_for;
use futures_util::stream::{iter, StreamExt};
use spartan_lib::core::dispatcher::SimpleDispatcher;
use std::time::Duration;

/// GC handler
///
/// Concurrently iterates over all databases in node, and executing GC on them.
pub async fn spawn_gc(manager: &Manager<'_>) {
    let timer = Duration::from_secs(manager.config.gc_timer);

    loop {
        delay_for(timer).await;

        iter(manager.node().db.iter())
            .for_each_concurrent(None, |(name, db)| async move {
                let mut db = db.lock().await;

                info!("Started GC cycle on database \"{}\"", name);

                db.gc();

                info!("GC cycle on \"{}\" completed successfully", name);
            })
            .await;
    }
}
