/// Queue access key
pub mod key;

/// Replication config
pub mod replication;

use key::Key;
use replication::Replication;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

/// Default database path
fn default_path() -> Box<Path> {
    PathBuf::from("./db").into_boxed_path()
}

/// Default amount of seconds between persistence jobs
const fn default_persistence_timer() -> u64 {
    900
}

/// Default amount of seconds between GC jobs
const fn default_gc_timer() -> u64 {
    300
}

/// Server configuration
#[derive(Serialize, Deserialize)]
pub struct Config {
    /// Max body size in bytes
    /// Default value is defined in Actix source code
    pub body_size: Option<usize>,

    /// Database path
    #[serde(default = "default_path")]
    pub path: Box<Path>,

    /// Amount of seconds between persistence jobs
    #[serde(default = "default_persistence_timer")]
    pub persistence_timer: u64,

    /// Amount of seconds between GC jobs
    #[serde(default = "default_gc_timer")]
    pub gc_timer: u64,

    /// Array of queues
    pub queues: Box<[Box<str>]>,

    /// Persistence encryption key
    pub encryption_key: Option<Box<str>>,

    /// Queue access keys
    pub access_keys: Option<HashSet<Key>>,

    /// Replication config
    pub replication: Option<Replication>,
}

#[cfg(not(test))]
impl Default for Config {
    fn default() -> Config {
        Config {
            body_size: None,
            path: default_path(),
            persistence_timer: default_persistence_timer(),
            gc_timer: default_gc_timer(),
            queues: Box::new([]),
            encryption_key: None,
            access_keys: None,
            replication: None,
        }
    }
}

#[cfg(test)]
impl Default for Config {
    fn default() -> Config {
        Config {
            body_size: None,
            path: default_path(),
            persistence_timer: 30,
            gc_timer: 10,
            queues: Box::new([String::from("test").into_boxed_str()]),
            encryption_key: None,
            access_keys: None,
            replication: None,
        }
    }
}
