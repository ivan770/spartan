use crate::{
    cli::Server,
    http::server::{start_http_server, ServerError},
    jobs::{exit::spawn_ctrlc_handler, gc::spawn_gc, persistence::spawn_persistence},
    node::persistence::PersistenceError,
    node::Manager,
};
use actix_rt::System;
use actix_web::web::Data;
use std::{io::Error as IoError, net::SocketAddr};
use structopt::StructOpt;
use thiserror::Error;
use tokio::{spawn, task::LocalSet};

#[cfg(feature = "replication")]
use crate::jobs::replication::spawn_replication;

#[derive(Error, Debug)]
pub enum StartCommandError {
    #[error("Unable to load configuration file")]
    ConfigFileError,
    #[error("Internal runtime error")]
    RuntimeError(IoError),
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
        debug!("Initializing runtime.");

        let local_set = LocalSet::new();
        let sys = System::run_in_tokio("server", &local_set);

        debug!("Runtime initialized.");

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

        let manager = Data::new(manager);

        let cloned_manager = manager.clone();
        spawn(async move { spawn_gc(&cloned_manager).await });

        let cloned_manager = manager.clone();
        spawn(async move { spawn_persistence(&cloned_manager).await });

        #[cfg(feature = "replication")]
        {
            let cloned_manager = manager.clone();
            spawn(async move { spawn_replication(&cloned_manager).await });
        }

        let cloned_manager = manager.clone();
        spawn(async move { spawn_ctrlc_handler(&cloned_manager).await });

        start_http_server(self.host(), manager)
            .await
            .map_err(StartCommandError::HttpServerError)?;

        sys.await.map_err(StartCommandError::RuntimeError)?;

        Ok(())
    }
}
