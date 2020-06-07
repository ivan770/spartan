#[macro_use]
extern crate derive_new;

#[macro_use]
extern crate log;

mod actions;
mod node;
mod query;
mod routing;
mod server;

use node::{Node, Persistence, persistence::spawn};
use routing::attach_routes;
use server::Server;
use structopt::StructOpt;
use tide::{with_state, JobContext, Request as TideRequest};

pub type Request = TideRequest<Persistence>;

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    pretty_env_logger::init();

    let server = Server::from_args();
    debug!("Server initialized.");
    // let mut node = Node::default();
    info!("Node initialized. Starting config loading.");
    let config = server.config().await?;
    
    let mut persistence = Persistence::new(config);
    persistence.load();

    debug!("Config loaded.");

    let mut tide = with_state(persistence);
    debug!("Tide initialized. Loading routes.");

    attach_routes(&mut tide);

    debug!("Routes loaded.");

    tide.spawn(|ctx: JobContext<Persistence>| async move {
        spawn(ctx.state()).await
    });

    tide.listen(server.host()).await?;
    Ok(())
}
