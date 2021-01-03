use std::{net::SocketAddr, sync::Arc};

use structopt::StructOpt;
use thiserror::Error;

#[cfg(feature = "replication")]
use crate::jobs::replication::spawn_replication;
use crate::{
    cli::Server,
    dispatch_jobs,
    http::server::{start_http_server, ServerError},
    jobs::{gc::spawn_gc, persistence::spawn_persistence},
    node::{persistence::PersistenceError, Manager},
};

#[derive(Error, Debug)]
pub enum StartCommandError {
    #[error("Unable to load configuration file")]
    ConfigFileError,
    #[error("HTTP server error: {0}")]
    HttpServerError(ServerError),
    #[error("Persistence error: {0}")]
    PersistenceError(PersistenceError),
}

#[derive(StructOpt)]
pub struct StartCommand {
    /// Server host
    #[structopt(default_value = "127.0.0.1:5680", long)]
    host: SocketAddr,
}

impl StartCommand {
    pub fn host(&self) -> SocketAddr {
        self.host
    }

    pub async fn dispatch(&self, server: &'static Server) -> Result<(), StartCommandError> {
        info!("Initializing node.");

        let config = server.config().ok_or(StartCommandError::ConfigFileError)?;
        let mut manager = Manager::new(config);

        info!("Node initialized.");

        info!("Loading queues from FS.");

        match manager.load_from_fs().await {
            Err(PersistenceError::FileOpenError(e)) => error!("Unable to load database: {}", e),
            Err(e) => Err(e).map_err(StartCommandError::PersistenceError)?,
            Ok(_) => (),
        };

        let manager = Arc::new(manager);

        dispatch_jobs!(manager, spawn_gc, spawn_persistence);

        #[cfg(feature = "replication")]
        dispatch_jobs!(manager, spawn_replication);

        start_http_server(self.host(), manager)
            .await
            .map_err(StartCommandError::HttpServerError)?;

        Ok(())
    }
}
