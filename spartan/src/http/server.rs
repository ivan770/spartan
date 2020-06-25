use crate::{config::Config, node::Manager, routing::attach_routes};
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
    config: &'static Config,
) -> Result<(), ServerError> {
    HttpServer::new(move || {
        App::new()
            .app_data(manager.clone())
            .configure(move |service_config| {
                attach_routes(config, service_config);
            })
    })
    .bind(host)
    .map_err(ServerError::AddressBinding)?
    .run()
    .await
    .map_err(ServerError::ServerError)?;

    Ok(())
}
