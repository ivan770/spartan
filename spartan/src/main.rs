#[macro_use]
extern crate derive_new;

#[macro_use]
extern crate log;

mod actions;
mod node;
mod query;
mod routing;
mod server;

use anyhow::Error;
use node::{gc::spawn_gc, load_from_fs, spawn_persistence, Manager};
use routing::attach_routes;
use server::Server;
use structopt::StructOpt;
use tide::{with_state, JobContext, Request as TideRequest};

pub type Request = TideRequest<Manager>;

#[async_std::main]
async fn main() -> Result<(), Error> {
    pretty_env_logger::init();

    let server = Server::from_args();
    debug!("Server initialized.");

    info!("Loading persistence module.");

    let mut manager = Manager::new(server.config().await?);
    manager.load();

    load_from_fs(&mut manager).await?;

    info!("Persistence module initialized.");

    let mut tide = with_state(manager);
    debug!("Tide initialized. Loading routes.");

    attach_routes(&mut tide);

    debug!("Routes loaded.");

    debug!("Spawning GC job.");

    tide.spawn(|ctx: JobContext<Manager>| async move { spawn_gc(ctx.state()).await });

    debug!("Spawning persistence job.");

    tide.spawn(|ctx: JobContext<Manager>| async move { spawn_persistence(ctx.state()).await });

    tide.listen(server.host()).await?;
    Ok(())
}
