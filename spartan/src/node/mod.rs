/// Ctrl-C handler
pub mod exit;

/// GC handler
pub mod gc;

/// Node manager
pub mod manager;

/// Persistence handler
pub mod persistence;

pub use exit::spawn_ctrlc_handler;
pub use manager::Manager;
pub use persistence::{load_from_fs, persist_manager, spawn_persistence};

use crate::config::Config;
use futures_util::lock::{Mutex, MutexGuard};
use spartan_lib::core::{db::tree::TreeDatabase, message::Message};
use std::collections::{hash_map::RandomState, HashMap};

pub type DB = TreeDatabase<Message>;
type MutexDB = Mutex<DB>;

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
    pub async fn get(&self, name: &str) -> Option<MutexGuard<'_, TreeDatabase<Message>>> {
        debug!("Obtaining queue \"{}\"", name);
        Some(self.queue(name)?.lock().await)
    }

    /// Add queue entry to node
    pub fn add(&mut self, name: &'a str) {
        info!("Initializing queue \"{}\"", name);
        self.db.insert(name, Mutex::new(TreeDatabase::default()));
    }

    /// Load queues from config
    pub fn load_from_config(&mut self, config: &'a Config) {
        config.queues.iter().for_each(|queue| self.add(queue));
    }
}
