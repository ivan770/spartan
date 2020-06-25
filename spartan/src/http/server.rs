use crate::{cli::Server, node::Manager, routing::attach_routes};
use actix_web::{web::Data, App, HttpServer};
use std::{io::Error as IoError, net::SocketAddr};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("Unable to bind server to address: {0}")]
    AddressBinding(IoError),
    #[error("Internal server error: {0}")]
    ServerError(IoError),
}

pub async fn start_http_server(
    host: SocketAddr,
    manager: Data<Manager<'static>>,
    server: &'static Server,
) -> Result<(), ServerError> {
    HttpServer::new(move || {
        let app = App::new()
            .app_data(manager.clone())
            .configure(attach_routes);

        if server
            .config()
            .expect("Config not loaded")
            .access_keys
            .is_some()
        {}

        app
    })
    .bind(host)
    .map_err(ServerError::AddressBinding)?
    .run()
    .await
    .map_err(ServerError::ServerError)?;

    Ok(())
}
