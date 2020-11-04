// Replication GC
// https://github.com/rust-lang/rust/issues/70530
#![feature(btree_drain_filter)]

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

/// Configuration
pub mod config;

/// Background jobs
mod jobs;

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
        #[cfg(feature = "init")]
        Init(command) => command.dispatch(server).await?,
        #[cfg(feature = "replication")]
        Replica(command) => command.dispatch(server).await?,
    };

    Ok(())
}
