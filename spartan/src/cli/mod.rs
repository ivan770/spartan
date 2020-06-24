/// CLI commands
mod commands;

use crate::config::Config;
use commands::start::StartCommand;
use std::{io::Error, path::PathBuf};
use structopt::StructOpt;
use tokio::fs::read;
use toml::from_slice;

#[derive(StructOpt)]
pub enum Command {
    #[structopt(about = "Start Spartan MQ server")]
    Start(StartCommand),
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
        self.loaded_config = Some(from_slice(&read(self.config.as_path()).await?)?);
        Ok(self)
    }

    pub fn config(&self) -> &Config {
        self.loaded_config.as_ref().expect("Config not loaded")
    }

    pub fn command(&self) -> &Command {
        &self.command
    }
}
