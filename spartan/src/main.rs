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
use node::{
    persistence::{load_from_fs, spawn, PersistenceError},
    Persistence,
};
use routing::attach_routes;
use server::Server;
use structopt::StructOpt;
use tide::{with_state, JobContext, Request as TideRequest};

pub type Request = TideRequest<Persistence>;

#[async_std::main]
async fn main() -> Result<(), Error> {
    pretty_env_logger::init();

    let server = Server::from_args();
    debug!("Server initialized.");

    info!("Loading persistence module.");

    let mut persistence = Persistence::new(server.config().await?);
    persistence.load();

    load_from_fs(&mut persistence).await?;

    info!("Persistence module initialized.");

    let mut tide = with_state(persistence);
    debug!("Tide initialized. Loading routes.");

    attach_routes(&mut tide);

    debug!("Routes loaded.");

    tide.spawn(|ctx: JobContext<Persistence>| async move {
        match spawn(ctx.state()).await {
            Err(PersistenceError::SerializationError(e)) => {
                error!("Unable to serialize database: {}", e)
            }
            Err(PersistenceError::FileWriteError(e)) => {
                error!("Unable to write serialized database to file: {}", e)
            }
            _ => unreachable!(),
        }
    });

    tide.listen(server.host()).await?;
    Ok(())
}
