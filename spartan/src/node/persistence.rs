use super::Node;
use crate::server::Config;
use async_std::task::sleep;
use std::time::Duration;

pub async fn spawn(persistence: &Persistence) {
    // This is a test
    use spartan_lib::core::dispatcher::simple::SimpleDispatcher;

    loop {
        sleep(Duration::from_secs(3)).await;

        for (name, db) in persistence.node().db.iter() {
            println!("{}: {}", name, db.lock().await.size());
        }
    }
}

pub struct Persistence {
    config: Config,
    node: Option<Node>,
}

impl Persistence {
    pub fn new(config: Config) -> Persistence {
        Persistence { config, node: None }
    }

    pub fn node(&self) -> &Node {
        self.node.as_ref().expect("Node not loaded")
    }

    pub fn load(&mut self) {
        let mut node = Node::default();
        node.load_from_config(&self.config);
        self.node = Some(node);
    }
}
