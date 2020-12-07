use std::process::exit;

use actix_rt::signal::ctrl_c;

use crate::node::Manager;

/// Ctrl-C handler
///
/// Listens to Ctrl-C signal, and after receiving one starts persisting database.
pub async fn spawn_ctrlc_handler(manager: &Manager<'_>) {
    debug!("Spawning Ctrl-C handler");

    ctrl_c().await.expect("Unable to listen to Ctrl-C signal.");

    if let Err(e) = manager.snapshot().await {
        error!("Error happened during shutdown: {}", e)
    }

    exit(0);
}
