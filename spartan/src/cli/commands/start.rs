use crate::{
    cli::Server,
    node::{
        gc::spawn_gc, load_from_fs, persistence::PersistenceError, spawn_ctrlc_handler,
        spawn_persistence, Manager,
    },
    routing::attach_routes,
};
use actix_rt::System;
use actix_web::{web::Data, App, HttpServer};
use std::{io::Error as IoError, net::SocketAddr};
use structopt::StructOpt;
use thiserror::Error;
use tokio::{spawn, task::LocalSet};

#[derive(Error, Debug)]
pub enum StartCommandError {
    #[error("Unable to restore DB from FS: {0}")]
    RestoreDB(PersistenceError),
    #[error("Unable to bind server to address: {0}")]
    AddressBinding(IoError),
    #[error("Internal server error: {0}")]
    ServerError(IoError),
    #[error("Unable to load configuration file")]
    ConfigFileError,
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

        HttpServer::new(move || {
            App::new()
                .app_data(manager.clone())
                .configure(attach_routes)
        })
        .bind(self.host())
        .map_err(StartCommandError::AddressBinding)?
        .run()
        .await
        .map_err(StartCommandError::ServerError)?;

        sys.await.map_err(StartCommandError::ServerError)?;

        Ok(())
    }
}
