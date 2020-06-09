use super::Manager;
use async_std::task::sleep;
use futures::stream::{iter, StreamExt};
use spartan_lib::core::dispatcher::SimpleDispatcher;
use std::time::Duration;

pub async fn spawn_gc(manager: &Manager) {
    let timer = Duration::from_secs(manager.config.gc_timer);

    loop {
        sleep(timer).await;

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
