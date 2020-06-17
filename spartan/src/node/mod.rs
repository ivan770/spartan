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

use crate::server::Config;
use futures_util::lock::{Mutex, MutexGuard};
use spartan_lib::core::{db::tree::TreeDatabase, message::Message};
use std::{
    collections::{hash_map::RandomState, HashMap},
    fmt::Display,
};

pub type DB = TreeDatabase<Message>;
type MutexDB = Mutex<DB>;

/// Key-value node implementation
#[derive(Default)]
pub struct Node<S = RandomState> {
    /// Node database
    db: HashMap<String, MutexDB, S>,
}

impl Node {
    /// Get node queue entry
    pub fn queue<T>(&self, name: T) -> Option<&MutexDB>
    where
        T: Display,
    {
        self.db.get(&name.to_string())
    }

    /// Get locked queue instance
    pub async fn get<T>(&self, name: T) -> Option<MutexGuard<'_, TreeDatabase<Message>>>
    where
        T: Display,
    {
        debug!("Obtaining queue \"{}\"", name);
        Some(self.queue(name)?.lock().await)
    }

    /// Add queue entry to node
    pub fn add<T>(&mut self, name: T)
    where
        T: Display,
    {
        info!("Initializing queue \"{}\"", name);
        self.db
            .insert(name.to_string(), Mutex::new(TreeDatabase::default()));
    }

    /// Load queues from config
    pub fn load_from_config(&mut self, config: &Config) {
        config.queues.iter().for_each(|queue| self.add(queue));
    }
}
