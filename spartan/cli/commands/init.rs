use crate::{cli::Server, config::Config};
use dialoguer::Editor;
use std::{ffi::OsString, io::Error as IoError};
use structopt::StructOpt;
use thiserror::Error;
use tokio::fs::write;
use toml::to_string_pretty;

#[derive(Error, Debug)]
pub enum InitCommandError {
    #[error("Unable to serialize config")]
    ConfigSerializationError,
    #[error("Unable to write serialized config to file: {0}")]
    ConfigWriteError(IoError),
    #[error("Missing config text. Probably file was not saved properly")]
    MissingConfigText,
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

        editor
            .require_save(true)
            .edit(
                &to_string_pretty(&config)
                    .map_err(|_| InitCommandError::ConfigSerializationError)?,
            )
            .map_err(InitCommandError::ConfigWriteError)?
            .map(|text| async move {
                write(server.config_path(), text)
                    .await
                    .map_err(InitCommandError::ConfigWriteError)
            })
            .ok_or(InitCommandError::MissingConfigText)?
            .await?;

        Ok(())
    }
}
