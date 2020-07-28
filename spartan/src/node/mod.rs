/// Node manager
pub mod manager;

#[cfg(feature = "replication")]
/// Database replication
pub mod replication;

pub use manager::Manager;

use crate::config::Config;
use futures_util::lock::{Mutex, MutexGuard};
use spartan_lib::core::{db::tree::TreeDatabase, message::Message};
use std::collections::{hash_map::RandomState, HashMap};

#[cfg(feature = "replication")]
use replication::database::ReplicatedDatabase;

/// Conjucted type of replicated database based on tree database
#[cfg(feature = "replication")]
pub type DB = ReplicatedDatabase<TreeDatabase<Message>>;
#[cfg(not(feature = "replication"))]
pub type DB = TreeDatabase<Message>;

/// Mutexed database
pub type MutexDB = Mutex<DB>;

/// Key-value node implementation
#[derive(Default)]
pub struct Node<'a, S = RandomState> {
    /// Node database
    db: HashMap<&'a str, MutexDB, S>,
}

impl<'a> Node<'a> {
    /// Get node queue entry
    pub fn queue(&self, name: &str) -> Option<&MutexDB> {
        self.db.get(name)
    }

    /// Get locked queue instance
    pub async fn get(&self, name: &str) -> Option<MutexGuard<'_, DB>> {
        debug!("Obtaining queue \"{}\"", name);
        Some(self.queue(name)?.lock().await)
    }

    /// Add queue entry to node
    pub fn add(&mut self, name: &'a str) {
        self.add_db(name, DB::default())
    }

    pub fn add_db(&mut self, name: &'a str, db: DB) {
        info!("Initializing queue \"{}\"", name);
        self.db.insert(name, Mutex::new(db));
    }

    pub fn iter(&'a self) -> impl Iterator<Item = (&&'a str, &'a MutexDB)> {
        self.db.iter()
    }

    /// Load queues from config
    pub fn load_from_config(&mut self, config: &'a Config) {
        config.queues.iter().for_each(|queue| self.add(queue));
    }
}
