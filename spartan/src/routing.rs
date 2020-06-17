use actix_web::web::{self, ServiceConfig};

macro_rules! route {
    ($name:ident) => {
        crate::actions::$name::$name
    };
}

/// Attach routes to Actix service config
pub fn attach_routes(config: &mut ServiceConfig) {
    config.service(
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
