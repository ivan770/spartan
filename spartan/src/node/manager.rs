use super::Node;
use crate::server::Config;

/// Node manager
pub struct Manager {
    /// Server config
    pub config: Config,

    /// Node
    node: Option<Node>,
}

impl Manager {
    /// Create new manager without node
    pub fn new(config: Config) -> Manager {
        Manager { config, node: None }
    }

    /// Get node reference
    pub fn node(&self) -> &Node {
        self.node.as_ref().expect("Node not loaded")
    }

    /// Get mutable node reference
    pub fn node_mut(&mut self) -> &mut Node {
        self.node.as_mut().expect("Node not loaded")
    }

    /// Load node instance from config
    pub fn load(&mut self) {
        let mut node = Node::default();
        node.load_from_config(&self.config);
        self.node = Some(node);
    }
}
