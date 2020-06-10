use super::{persist_manager, Manager};
use async_ctrlc::CtrlC;
use std::process::exit;

pub async fn spawn_ctrlc_handler(manager: &Manager) {
    CtrlC::new().expect("Cannot create Ctrl-C handler").await;

    persist_manager(manager).await;

    exit(0);
}
