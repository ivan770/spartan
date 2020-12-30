use std::{net::SocketAddr, sync::Arc};

use thiserror::Error;
use tokio::signal::ctrl_c;
use warp::{serve, Error};

use super::routing::attach_routes;
use crate::node::Manager;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("Internal server error: {0}")]
    Internal(#[from] Error),
}

/// Start HTTP server with shared manager
///
/// Server starts graceful shutdown process on CTRL-C signal
pub async fn start_http_server(
    host: SocketAddr,
    manager: Arc<Manager<'static>>,
) -> Result<(), ServerError> {
    serve(attach_routes(manager.clone()))
        .try_bind_with_graceful_shutdown(host, async {
            ctrl_c().await.ok();
        })
        .map_err(ServerError::Internal)?
        .1
        .await;

    manager.shutdown().await;

    Ok(())
}
