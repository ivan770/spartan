pub mod extractor;

pub use extractor::QueueExtractor;

use crate::server::Config;
use async_std::sync::{Mutex, MutexGuard};
use spartan_lib::core::{db::tree::TreeDatabase, message::Message};
use std::{collections::HashMap, fmt::Display};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NodeError {
    #[error("Filesystem error")]
    FsError,
    #[error("Serializer error")]
    BincodeError,
    #[error("Unable to open directory")]
    DirectoryOpenError,
}

pub type DB = TreeDatabase<Message>;
type MutexDB = Mutex<DB>;

#[derive(Default)]
pub struct Node {
    db: HashMap<String, MutexDB>,
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
        config.queues.into_iter().for_each(|queue| self.add(queue));
    }
}
