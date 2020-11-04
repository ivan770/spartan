use crate::{cli::Server, config::Config};
use dialoguer::Editor;
use std::{ffi::OsString, io::Error as IoError};
use structopt::StructOpt;
use thiserror::Error;
use tokio::fs::{create_dir, write};
use toml::{
    de::{from_str, Error},
    to_string_pretty,
};

#[derive(Error, Debug)]
pub enum InitCommandError {
    #[error("Unable to serialize config")]
    ConfigSerializationError,
    #[error("Unable to write serialized config to file: {0}")]
    ConfigWriteError(IoError),
    #[error("Missing config text. Probably file was not saved properly")]
    MissingConfigText,
    #[error("Provided config is incorrect. Check config key rules in README file: {0}")]
    IncorrectConfig(Error),
    #[error("Unable to create directory by path, provided in database path: {0}")]
    DirectoryCreateError(IoError),
}

#[derive(StructOpt)]
pub struct InitCommand {
    // Path to editor executable
    #[structopt(long)]
    editor: Option<OsString>,
}

impl InitCommand {
    pub async fn dispatch(&self, server: &Server) -> Result<(), InitCommandError> {
        let config = Config::default();
        let mut editor = Editor::default();

        self.editor.as_ref().map(|name| editor.executable(name));

        let text = editor
            .require_save(true)
            .edit(
                &to_string_pretty(&config)
                    .map_err(|_| InitCommandError::ConfigSerializationError)?,
            )
            .map_err(InitCommandError::ConfigWriteError)?
            .ok_or(InitCommandError::MissingConfigText)?;

        let config: Config = from_str(&text).map_err(InitCommandError::IncorrectConfig)?;

        if let Some(persistence) = config.persistence {
            if !persistence.path.is_dir() {
                create_dir(&persistence.path)
                    .await
                    .map_err(InitCommandError::DirectoryCreateError)?;
            }
        }

        write(server.config_path(), text)
            .await
            .map_err(InitCommandError::ConfigWriteError)?;

        Ok(())
    }
}
