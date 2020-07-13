/// CLI commands
mod commands;

use crate::config::Config;
use commands::{init::InitCommand, replica::ReplicaCommand, start::StartCommand};
use std::{
    io::Error,
    path::{Path, PathBuf},
};
use structopt::StructOpt;
use tokio::fs::read;
use toml::from_slice;

#[derive(StructOpt)]
pub enum Command {
    #[structopt(about = "Start Spartan MQ server")]
    Start(StartCommand),
    #[structopt(about = "Initialize configuration file")]
    Init(InitCommand),
    #[structopt(about = "Start replication server")]
    Replica(ReplicaCommand),
}

#[derive(StructOpt)]
pub struct Server {
    /// Server configuration path
    #[structopt(default_value = "Spartan.toml", long)]
    config: PathBuf,

    /// Loaded server configuration
    #[structopt(skip = None)]
    loaded_config: Option<Config>,

    #[structopt(subcommand)]
    command: Command,
}

impl Server {
    /// Load configuration
    pub async fn load_config(mut self) -> Result<Self, Error> {
        match read(self.config.as_path()).await {
            Ok(file) => self.loaded_config = Some(from_slice(&file)?),
            Err(e) => info!("Unable to load configuration file: {}", e),
        };

        Ok(self)
    }

    pub fn config(&self) -> Option<&Config> {
        self.loaded_config.as_ref()
    }

    pub fn config_path(&self) -> &Path {
        self.config.as_path()
    }

    pub fn command(&self) -> &Command {
        &self.command
    }
}
