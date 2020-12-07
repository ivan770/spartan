use std::{io::Error as IoError, net::SocketAddr};

use actix_web::{
    web::{Data, JsonConfig},
    App, HttpServer,
};
use thiserror::Error;

use super::routing::attach_routes;
use crate::node::Manager;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("Unable to bind server to address: {0}")]
    AddressBinding(IoError),
    #[error("Internal server error: {0}")]
    ServerError(IoError),
}

/// Start HTTP server with shared manager
pub async fn start_http_server(
    host: SocketAddr,
    manager: Data<Manager<'static>>,
) -> Result<(), ServerError> {
    HttpServer::new(move || {
        let config = manager.config();

        let mut app = App::new()
            .app_data(manager.clone())
            .configure(move |service_config| {
                attach_routes(config, service_config);
            });

        if let Some(bytes) = config.body_size {
            app = app.app_data(JsonConfig::default().limit(bytes));
        }

        app
    })
    .bind(host)
    .map_err(ServerError::AddressBinding)?
    .run()
    .await
    .map_err(ServerError::ServerError)?;

    Ok(())
}
