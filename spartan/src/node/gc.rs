use super::Manager;
use async_std::task::sleep;
use spartan_lib::core::dispatcher::SimpleDispatcher;
use std::time::Duration;

pub async fn spawn_gc(manager: &Manager) {
    loop {
        sleep(Duration::from_secs(manager.config.gc_timer)).await;

        for (name, db) in manager.node().db.iter() {
            let mut db = db.lock().await;

            info!("Started GC cycle on database \"{}\"", name);

            db.gc();

            info!("GC cycle on \"{}\" completed successfully", name);
        }
    }
}
