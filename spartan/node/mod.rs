/// Node manager
pub mod manager;

/// Queue
pub mod queue;

#[cfg(feature = "replication")]
/// Database replication
pub mod replication;

pub use manager::Manager;
pub use queue::Queue;

use crate::config::Config;
use spartan_lib::core::{db::tree::TreeDatabase, message::Message};
use std::collections::{hash_map::RandomState, HashMap};

pub type DB = Queue<TreeDatabase<Message>>;

/// Key-value node implementation
#[derive(Default)]
pub struct Node<'a, S = RandomState> {
    /// Node database
    db: HashMap<&'a str, DB, S>,
}

impl<'a> Node<'a> {
    /// Get node queue entry
    pub fn queue(&self, name: &str) -> Option<&DB> {
        self.db.get(name)
    }

    /// Add default queue entry to node
    pub fn add(&mut self, name: &'a str) {
        self.add_db(name, DB::default())
    }

    /// Add queue entry to node
    pub fn add_db(&mut self, name: &'a str, db: DB) {
        info!("Initializing queue \"{}\"", name);
        self.db.insert(name, db);
    }

    /// Iterate over queues in Node
    pub fn iter(&'a self) -> impl Iterator<Item = (&&'a str, &'a DB)> {
        self.db.iter()
    }

    /// Load queues from config
    pub fn load_from_config(&mut self, config: &'a Config) {
        config.queues.iter().for_each(|queue| self.add(queue));
    }
}
