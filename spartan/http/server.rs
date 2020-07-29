use crate::{config::Config, node::Manager, routing::attach_routes};
use actix_web::{
    web::{Data, JsonConfig},
    App, HttpServer,
};
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
