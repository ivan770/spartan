use serde::Deserialize;
use std::path::PathBuf;

/// Default database path
fn default_path() -> PathBuf {
    PathBuf::from("./db")
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
#[derive(Deserialize)]
pub struct Config {
    /// Array of queues
    pub queues: Vec<String>,

    /// Database path
    #[serde(default = "default_path")]
    pub path: PathBuf,

    /// Amount of seconds between persistence jobs
    #[serde(default = "default_persistence_timer")]
    pub persistence_timer: u64,

    /// Amount of seconds between GC jobs
    #[serde(default = "default_gc_timer")]
    pub gc_timer: u64,
}

#[cfg(test)]
impl Default for Config {
    fn default() -> Config {
        Config {
            gc_timer: 10,
            persistence_timer: 30,
            path: PathBuf::from("./db"),
            queues: vec![String::from("test")],
        }
    }
}
