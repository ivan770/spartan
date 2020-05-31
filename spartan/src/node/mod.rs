pub mod extractor;
pub mod persistence;

pub use extractor::QueueExtractor;
pub use persistence::Persistence;

use crate::server::Config;
use async_std::sync::{Mutex, MutexGuard};
use spartan_lib::core::{db::tree::TreeDatabase, message::Message};
use std::{collections::HashMap, fmt::Display};

pub type DB = TreeDatabase<Message>;
type MutexDB = Mutex<DB>;

pub struct PersistenceConfig {
    pub timer: u64,
}

impl From<&Config> for PersistenceConfig {
    fn from(config: &Config) -> Self {
        PersistenceConfig {
            timer: config.timer,
        }
    }
}

#[derive(Default)]
pub struct Node {
    db: HashMap<String, MutexDB>,
    persistence_config: Option<PersistenceConfig>,
}

impl Node {
    pub fn queue<T>(&self, name: T) -> Option<&MutexDB>
    where
        T: Display,
    {
        self.db.get(&name.to_string())
    }

    pub async fn get<T>(&self, name: T) -> Option<MutexGuard<'_, TreeDatabase<Message>>>
    where
        T: Display,
    {
        debug!("Obtaining queue \"{}\"", name);
        Some(self.queue(name)?.lock().await)
    }

    pub fn add<T>(&mut self, name: T)
    where
        T: Display,
    {
        info!("Initializing queue \"{}\"", name);
        self.db
            .insert(name.to_string(), Mutex::new(TreeDatabase::default()));
    }

    pub fn load_from_config(&mut self, config: Config) {
        self.persistence_config = Some(PersistenceConfig::from(&config));
        config.queues.into_iter().for_each(|queue| self.add(queue));
    }
}
