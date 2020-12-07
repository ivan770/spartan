use actix_web::web::{self, ServiceConfig};

use crate::{config::Config, http::middleware::access::Access};

macro_rules! route {
    ($name:ident) => {
        crate::actions::$name::$name
    };
}

/// Attach routes to Actix service config
pub fn attach_routes(config: &'static Config, service_config: &mut ServiceConfig) {
    service_config.service(
        web::scope("/{queue}")
            .service(
                web::resource("")
                    .route(web::get().to(route!(pop)))
                    .route(web::post().to(route!(push)))
                    .route(web::delete().to(route!(delete))),
            )
            .route("/size", web::get().to(route!(size)))
            .route("/requeue", web::post().to(route!(requeue)))
            .route("/clear", web::post().to(route!(clear)))
            .wrap(Access::new(config)),
    );
}

#[cfg(test)]
/// Attach test routes to Actix service config
pub fn attach_test_routes(service_config: &mut ServiceConfig) {
    service_config.service(
        web::scope("/{queue}")
            .service(
                web::resource("")
                    .route(web::get().to(route!(pop)))
                    .route(web::post().to(route!(push)))
                    .route(web::delete().to(route!(delete))),
            )
            .route("/size", web::get().to(route!(size)))
            .route("/requeue", web::post().to(route!(requeue)))
            .route("/clear", web::post().to(route!(clear))),
    );
}
