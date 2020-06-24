use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Server configuration
#[derive(Serialize, Deserialize)]
pub struct Config {
    /// Database path
    #[serde(default)]
    pub path: PathBuf,

    /// Amount of seconds between persistence jobs
    #[serde(default)]
    pub persistence_timer: u64,

    /// Amount of seconds between GC jobs
    #[serde(default)]
    pub gc_timer: u64,

    /// Array of queues
    pub queues: Vec<String>,
}

#[cfg(not(test))]
impl Default for Config {
    fn default() -> Config {
        Config {
            path: PathBuf::from("./db"),
            persistence_timer: 900,
            gc_timer: 300,
            queues: Vec::new(),
        }
    }
}

#[cfg(test)]
impl Default for Config {
    fn default() -> Config {
        Config {
            path: PathBuf::from("./db"),
            persistence_timer: 30,
            gc_timer: 10,
            queues: vec![String::from("test")],
        }
    }
}
