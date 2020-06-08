use async_std::fs::read;
use serde::Deserialize;
use std::{io::Error, net::SocketAddr, path::PathBuf};
use structopt::StructOpt;
use toml::from_slice;

fn default_path() -> PathBuf {
    PathBuf::from("./spartan")
}

const fn default_persistence_timer() -> u64 {
    900
}

const fn default_gc_timer() -> u64 {
    300
}

#[derive(Deserialize)]
pub struct Config {
    pub queues: Vec<String>,
    #[serde(default = "default_path")]
    pub path: PathBuf,
    #[serde(default = "default_persistence_timer")]
    pub persistence_timer: u64,
    #[serde(default = "default_gc_timer")]
    pub gc_timer: u64,
}

#[derive(StructOpt)]
pub struct Server {
    #[structopt(default_value = "127.0.0.1:5680", long)]
    host: SocketAddr,
    #[structopt(default_value = "Spartan.toml", long)]
    config: PathBuf,
}

impl Server {
    pub fn host(&self) -> SocketAddr {
        self.host
    }

    pub async fn config(&self) -> Result<Config, Error> {
        Ok(from_slice(&read(self.config.as_path()).await?)?)
    }
}
