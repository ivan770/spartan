#[macro_use]
extern crate derive_new;

#[macro_use]
extern crate log;

/// Route actions
mod actions;

/// CLI
mod cli;

/// Node
mod node;

/// HTTP requests and responses
mod query;

/// Actix routing
mod routing;

/// Configuration
mod config;

/// Utilities for easier development
pub mod utils;

use actix_rt::System;
use anyhow::Error;
use cli::Server;
use once_cell::sync::OnceCell;
use structopt::StructOpt;
use tokio::task::LocalSet;

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

    let server = SERVER.get().expect("Server is not initialized");

    match server.command() {
        cli::Command::Start(command) => command.dispatch(server).await?,
    };

    sys.await?;

    Ok(())
}
