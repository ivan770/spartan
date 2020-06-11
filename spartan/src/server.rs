use async_std::fs::read;
use serde::Deserialize;
use std::{io::Error, net::SocketAddr, path::PathBuf};
use structopt::StructOpt;
use toml::from_slice;

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

#[derive(StructOpt)]
pub struct Server {
    /// Server host
    #[structopt(default_value = "127.0.0.1:5680", long)]
    host: SocketAddr,

    /// Server configuration path
    #[structopt(default_value = "Spartan.toml", long)]
    config: PathBuf,
}

impl Server {
    /// Get server host
    pub fn host(&self) -> SocketAddr {
        self.host
    }

    /// Load configuration
    pub async fn config(&self) -> Result<Config, Error> {
        Ok(from_slice(&read(self.config.as_path()).await?)?)
    }
}
