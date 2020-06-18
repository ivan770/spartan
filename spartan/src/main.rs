#[macro_use]
extern crate derive_new;

#[macro_use]
extern crate log;

/// Route actions
mod actions;

/// Node
mod node;

/// HTTP requests and responses
mod query;

/// Actix routing
mod routing;

/// Server and configuration
mod server;

use actix_rt::System;
use actix_web::{web::Data, App, HttpServer};
use anyhow::Error;
use node::{gc::spawn_gc, load_from_fs, spawn_ctrlc_handler, spawn_persistence, Manager};
use once_cell::sync::OnceCell;
use routing::attach_routes;
use server::Server;
use structopt::StructOpt;
use tokio::{spawn, task::LocalSet};

static SERVER: OnceCell<Server> = OnceCell::new();

#[tokio::main]
async fn main() -> Result<(), Error> {
    pretty_env_logger::init_custom_env("LOG_LEVEL");

    debug!("Initializing runtime.");

    SERVER
        .set(Server::from_args().load_config().await?)
        .ok()
        .expect("Server was already initialized");

    let local_set = LocalSet::new();
    let sys = System::run_in_tokio("server", &local_set);

    debug!("Runtime initialized.");

    info!("Initializing node.");

    let mut manager = Manager::new(SERVER.get().expect("Server is uninitialized").config());

    info!("Node initialized.");

    info!("Loading queues from FS.");

    load_from_fs(&mut manager).await?;

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
    .bind(SERVER.get().expect("Server is uninitialized").host())?
    .run()
    .await?;

    sys.await?;

    Ok(())
}
