use super::{Node, PersistenceConfig};
use async_std::task::sleep;
use std::time::Duration;

pub async fn spawn(persistence: Persistence<'_>) {
    loop {
        sleep(Duration::from_secs(persistence.persistence_config.timer)).await;
    }
}

pub struct Persistence<'a> {
    node: &'a Node,
    persistence_config: &'a PersistenceConfig,
}

impl<'a> Persistence<'a> {
    pub fn new(node: &'a Node) -> Persistence {
        Persistence {
            node,
            persistence_config: node
                .persistence_config
                .as_ref()
                .expect("Persistence config is required in Node"),
        }
    }
}
