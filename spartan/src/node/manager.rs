use super::Node;
use crate::server::Config;

pub struct Manager {
    pub config: Config,
    node: Option<Node>,
}

impl Manager {
    pub fn new(config: Config) -> Manager {
        Manager { config, node: None }
    }

    pub fn node(&self) -> &Node {
        self.node.as_ref().expect("Node not loaded")
    }

    pub fn node_mut(&mut self) -> &mut Node {
        self.node.as_mut().expect("Node not loaded")
    }

    pub fn load(&mut self) {
        let mut node = Node::default();
        node.load_from_config(&self.config);
        self.node = Some(node);
    }
}
