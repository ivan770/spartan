use crate::{cli::Server, config::Config};
use std::io::Error as IoError;
use structopt::StructOpt;
use thiserror::Error;
use tokio::fs::write;
use toml::to_vec;

#[derive(Error, Debug)]
pub enum InitCommandError {
    #[error("Unable to serialize config")]
    ConfigSerializationError,
    #[error("Unable to write serialized config to file: {0}")]
    ConfigWriteError(IoError),
}

#[derive(StructOpt)]
pub struct InitCommand {}

impl InitCommand {
    pub async fn dispatch(&self, server: &Server) -> Result<(), InitCommandError> {
        let config = Config::default();

        write(
            server.config_path(),
            to_vec(&config).map_err(|_| InitCommandError::ConfigSerializationError)?,
        )
        .await
        .map_err(InitCommandError::ConfigWriteError)?;

        Ok(())
    }
}
