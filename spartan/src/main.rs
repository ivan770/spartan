#[macro_use]
extern crate derive_new;

#[macro_use]
extern crate log;

mod actions;
mod node;
mod query;
mod routing;
mod server;

use node::{persistence::spawn, Node, Persistence};
use routing::attach_routes;
use server::Server;
use structopt::StructOpt;
use tide::{with_state, JobContext, Request as TideRequest};

pub type Request = TideRequest<Node>;

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    pretty_env_logger::init();

    let server = Server::from_args();
    debug!("Server initialized.");
    let mut node = Node::default();
    info!("Node initialized. Starting config loading.");
    node.load_from_config(server.config().await?);
    debug!("Config loaded.");

    let mut tide = with_state(node);
    debug!("Tide initialized. Loading routes.");

    attach_routes(&mut tide);

    debug!("Routes loaded.");

    tide.spawn(|ctx: JobContext<Node>| async move {
        spawn(Persistence::new(ctx.state())).await;
    });

    tide.listen(server.host()).await?;
    Ok(())
}
