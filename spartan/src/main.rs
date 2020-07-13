#![feature(btree_drain_filter)]

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

/// HTTP server and query structs
mod http;

/// Actix routing
mod routing;

/// Configuration
mod config;

/// Utilities for easier development
pub mod utils;

use anyhow::Error;
use cli::{Command::*, Server};
use once_cell::sync::OnceCell;
use structopt::StructOpt;

static SERVER: OnceCell<Server> = OnceCell::new();

#[tokio::main]
async fn main() -> Result<(), Error> {
    pretty_env_logger::init_custom_env("LOG_LEVEL");

    SERVER
        .set(Server::from_args().load_config().await?)
        .ok()
        .expect("Server was already initialized");

    let server = SERVER.get().expect("Server is not initialized");

    match server.command() {
        Start(command) => command.dispatch(server).await?,
        Init(command) => command.dispatch(server).await?,
        Replica(command) => command.dispatch(server).await?,
    };

    Ok(())
}
