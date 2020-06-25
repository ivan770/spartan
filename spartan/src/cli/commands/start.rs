use crate::{
    cli::Server,
    http::server::{start_http_server, ServerError},
    node::{
        gc::spawn_gc, load_from_fs, persistence::PersistenceError, spawn_ctrlc_handler,
        spawn_persistence, Manager,
    },
};
use actix_rt::System;
use actix_web::web::Data;
use std::{io::Error as IoError, net::SocketAddr};
use structopt::StructOpt;
use thiserror::Error;
use tokio::{spawn, task::LocalSet};

#[derive(Error, Debug)]
pub enum StartCommandError {
    #[error("Unable to restore DB from FS: {0}")]
    RestoreDB(PersistenceError),
    #[error("Unable to load configuration file")]
    ConfigFileError,
    #[error("Internal runtime error")]
    RuntimeError(IoError),
    #[error("HTTP server error: {0}")]
    HttpServerError(ServerError),
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

        let mut manager = Manager::new(
            server
                .config()
                .ok_or_else(|| StartCommandError::ConfigFileError)?,
        );

        info!("Node initialized.");

        info!("Loading queues from FS.");

        load_from_fs(&mut manager)
            .await
            .map_err(StartCommandError::RestoreDB)?;

        info!("Queues loaded successfully.");

        let manager = Data::new(manager);

        debug!("Spawning GC handler.");

        let cloned_manager = manager.clone();
        spawn(async move { spawn_gc(&cloned_manager).await });

        debug!("Spawning persistence job.");

        let cloned_manager = manager.clone();
        spawn(async move { spawn_persistence(&cloned_manager).await });

        debug!("Spawning Ctrl-C handler");

        let cloned_manager = manager.clone();
        spawn(async move { spawn_ctrlc_handler(&cloned_manager).await });

        start_http_server(self.host(), manager, server)
            .await
            .map_err(StartCommandError::HttpServerError)?;

        sys.await.map_err(StartCommandError::RuntimeError)?;

        Ok(())
    }
}
