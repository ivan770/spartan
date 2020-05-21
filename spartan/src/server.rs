use async_std::fs::read;
use serde::Deserialize;
use std::{
    io::Error,
    net::SocketAddr,
    path::{Path, PathBuf},
};
use structopt::StructOpt;
use toml::from_slice;

#[derive(Deserialize)]
pub struct Config {
    pub queues: Vec<String>,
}

#[derive(StructOpt)]
pub struct Server {
    #[structopt(default_value = "127.0.0.1:5680", long)]
    host: SocketAddr,
    #[structopt(default_value = "./spartan", long)]
    db: PathBuf,
    #[structopt(default_value = "Spartan.toml", long)]
    config: PathBuf,
}

impl Server {
    pub fn host(&self) -> SocketAddr {
        self.host
    }

    pub fn path(&self) -> &Path {
        self.db.as_path()
    }

    pub async fn config(&self) -> Result<Config, Error> {
        Ok(from_slice(&read(self.config.as_path()).await?)?)
    }
}
